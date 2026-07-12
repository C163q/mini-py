use std::{
    num::NonZero,
    sync::{Mutex, MutexGuard},
};

use crate::{
    error::InterpreterError,
    lexer::indent::{Indent, OwnedLineIndent},
};

/// The type of an open bracket being tracked for implicit line continuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BracketType {
    Parenthesis,
    Square,
    Curly,
}

/// Mutable state carried across successive raw input chunks while assembling a logical [`Line`].
#[derive(Debug, Clone)]
pub struct LineContext {
    /// The leading indentation accumulated for the line currently being built.
    /// [`None`] means this is the first line of a logical line, and the indentation has not been
    /// determined yet.
    pub indents: Option<OwnedLineIndent>,
    /// Open brackets seen so far; a non-empty stack means line continuation is still in progress.
    pub bracket_stack: Vec<BracketType>,
    /// Buffers partial content until the logical line is complete.
    pub concatenator: LineConcatenator,
}

impl Default for LineContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LineContext {
    pub fn new() -> Self {
        Self {
            indents: None,
            bracket_stack: Vec::new(),
            concatenator: LineConcatenator::new(),
        }
    }
}

/// Buffers partial line content across multiple input chunks until a logical line is complete.
///
/// Chunks are joined with a single space to preserve token boundaries across physical newlines.
#[derive(Debug, Clone)]
pub struct LineConcatenator {
    line: String,
}

impl Default for LineConcatenator {
    fn default() -> Self {
        Self::new()
    }
}

impl LineConcatenator {
    /// Creates an empty `LineConcatenator`.
    pub fn new() -> Self {
        Self {
            line: String::new(),
        }
    }

    /// Discards all buffered content.
    pub fn clear(&mut self) {
        self.line.clear();
    }

    /// Appends `chunk` to the buffer.
    pub fn append(&mut self, command: &str) {
        self.line.push_str(command);
    }

    /// Returns the accumulated content so far.
    pub fn get(&self) -> &str {
        &self.line
    }
}

/// A single logical line together with its leading indentation.
///
/// A "logical line" may span multiple physical lines: implicit continuation inside open
/// brackets and explicit continuation with a trailing `\` are both merged into one `Line`.
/// For example:
///
/// ```text
/// x = (1 +
///      2)
/// ```
///
/// produces a single `Line` whose `content` is `"x = (1 + 2)"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub indent: OwnedLineIndent,
    pub content: String,
}

impl Line {
    /// Creates a `Line` from a pre-parsed indentation sequence and content string.
    pub fn new(indent: OwnedLineIndent, content: String) -> Self {
        Self { indent, content }
    }
}

macro_rules! get_line_branch {
    ($current:tt, $other:tt, $last_indent:ident, $indents:ident) => {
        if let Some(indent) = $last_indent.as_mut() {
            match indent {
                Indent::$current(count) => {
                    *count = NonZero::new(count.get() + 1).ok_or_else(|| {
                        InterpreterError::new_lexical_error(String::from("Indentation too large"))
                    })?
                }
                Indent::$other(_) => {
                    $indents.0.push(*indent);
                    $last_indent = Some(Indent::$current(NonZero::new(1).unwrap()));
                }
            }
        } else {
            $last_indent = Some(Indent::$current(NonZero::new(1).unwrap()));
        }
    };
}

/// Attempts to assemble a complete [`Line`] from a raw input string, updating `last` in place.
///
/// Returns `Ok(Some(Line))` when a logical line is complete, `Ok(None)` if more input is
/// needed (open brackets or trailing `\`), or an error on malformed input.
pub fn get_line(last: &Mutex<LineContext>, line: &str) -> Result<Option<Line>, InterpreterError> {
    if last.lock().unwrap().concatenator.get().is_empty() {
        let line = line.trim_end();
        if line.is_empty() {
            return Ok(None);
        }

        // None means this is the first line of a logical line, so we need to parse the indentation.
        if last.lock().unwrap().indents.is_none() {
            let mut last_indent: Option<Indent> = None;
            let mut indents = OwnedLineIndent::new();
            for ch in line.chars() {
                match ch {
                    ' ' => {
                        get_line_branch!(Space, Tab, last_indent, indents);
                    }
                    '\t' => {
                        get_line_branch!(Tab, Space, last_indent, indents);
                    }
                    _ => {
                        if let Some(indent) = last_indent {
                            indents.0.push(indent);
                        }
                        break;
                    }
                }
            }
            last.lock().unwrap().indents = Some(indents);
        }
    }

    // handle the multiline line continuation and bracket matching
    let line_content = get_line_content(last.lock().unwrap(), line)?;

    match line_content {
        Some(content) => {
            let mut last = last.lock().unwrap();
            Ok(Some(Line::new(last.indents.take().unwrap(), content)))
        }
        None => Ok(None),
    }
}

fn get_line_content(
    mut last: MutexGuard<'_, LineContext>,
    line: &str,
) -> Result<Option<String>, InterpreterError> {
    // We DO NOT handle the leading spaces here.
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    // handle bracket matching
    for ch in line.chars() {
        match ch {
            '(' => last.bracket_stack.push(BracketType::Parenthesis),
            '[' => last.bracket_stack.push(BracketType::Square),
            '{' => last.bracket_stack.push(BracketType::Curly),
            ')' => {
                if let Some(BracketType::Parenthesis) = last.bracket_stack.pop() {
                } else {
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Unmatched closing parenthesis",
                    )));
                }
            }
            ']' => {
                if let Some(BracketType::Square) = last.bracket_stack.pop() {
                } else {
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Unmatched closing square bracket",
                    )));
                }
            }
            '}' => {
                if let Some(BracketType::Curly) = last.bracket_stack.pop() {
                } else {
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Unmatched closing curly bracket",
                    )));
                }
            }
            _ => {}
        }
    }

    fn push_space_if_needed(cat: &mut LineConcatenator) {
        if !cat.get().is_empty() {
            cat.append(" ");
        }
    }

    // handle line continuation
    // end with backslash, wait for next line
    if line.ends_with('\\') {
        push_space_if_needed(&mut last.concatenator);
        last.concatenator.append(line.strip_suffix('\\').unwrap());
        Ok(None)
    } else if !last.bracket_stack.is_empty() {
        // open brackets
        push_space_if_needed(&mut last.concatenator);
        last.concatenator.append(line);
        Ok(None)
    } else {
        // finish a complete logical line, return it
        let mut result = last.concatenator.get().to_string();
        if !result.is_empty() {
            result.push(' ');
            result.push_str(line);
        } else {
            result = line.to_string();
        }
        last.concatenator.clear();
        Ok(Some(result))
    }
}
