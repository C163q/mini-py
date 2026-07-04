use std::{
    num::NonZero,
    sync::{Mutex, MutexGuard},
};

use crate::{
    error::InterpreterError,
    lexer::indent::{Indent, OwnedLineIndent},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BracketType {
    Parenthesis,
    Square,
    Curly,
}

#[derive(Debug, Clone)]
pub struct LineContext {
    pub indents: OwnedLineIndent,
    pub bracket_stack: Vec<BracketType>,
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
            indents: OwnedLineIndent::new(),
            bracket_stack: Vec::new(),
            concatenator: LineConcatenator::new(),
        }
    }
}

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
    pub fn new() -> Self {
        Self {
            line: String::new(),
        }
    }

    pub fn clear(&mut self) {
        self.line.clear();
    }

    pub fn append(&mut self, command: &str) {
        self.line.push_str(command);
    }

    pub fn get(&self) -> &str {
        &self.line
    }
}

/// Line here does NOT mean that there is no '\n', for example:
///
/// ```python
/// s = """
/// value1
/// value2
/// """
/// ```
///
/// would be considered as a single line, and the content of the line would be `s = """\nvalue1\nvalue2\n"""`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub indent: OwnedLineIndent,
    pub content: String,
}

impl Line {
    pub fn new(indent: OwnedLineIndent, content: String) -> Self {
        Self { indent, content }
    }
}

macro_rules! get_line_branch {
    ($current:tt, $other:tt, $last_indent:ident, $last:tt) => {
        if let Some(indent) = $last_indent.as_mut() {
            match indent {
                Indent::$current(count) => {
                    *count = NonZero::new(count.get() + 1).ok_or_else(|| {
                        InterpreterError::new_lexical_error(String::from("Indentation too large"))
                    })?
                }
                Indent::$other(_) => {
                    $last.indents.0.push(*indent);
                    $last_indent = Some(Indent::$current(NonZero::new(1).unwrap()));
                }
            }
        } else {
            $last_indent = Some(Indent::$current(NonZero::new(1).unwrap()));
        }
    };
}

pub fn get_line(last: &Mutex<LineContext>, line: &str) -> Result<Option<Line>, InterpreterError> {
    if last.lock().unwrap().concatenator.get().is_empty() {
        let line = line.trim_end();
        if line.is_empty() {
            return Ok(None);
        }
        let mut last_indent: Option<Indent> = None;
        for ch in line.chars() {
            let mut last = last.lock().unwrap();
            match ch {
                ' ' => {
                    get_line_branch!(Space, Tab, last_indent, last);
                }
                '\t' => {
                    get_line_branch!(Tab, Space, last_indent, last);
                }
                _ => {
                    if let Some(indent) = last_indent {
                        last.indents.0.push(indent);
                    }
                    break;
                }
            }
        }
    }

    let line_content = get_line_content(last.lock().unwrap(), line)?;
    match line_content {
        Some(content) => {
            let mut last = last.lock().unwrap();
            let result = Ok(Some(Line::new(last.indents.clone(), content)));
            last.indents.0.clear();
            result
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

    if line.ends_with('\\') {
        push_space_if_needed(&mut last.concatenator);
        last.concatenator.append(line.strip_suffix('\\').unwrap());
        Ok(None)
    } else if !last.bracket_stack.is_empty() {
        push_space_if_needed(&mut last.concatenator);
        last.concatenator.append(line);
        Ok(None)
    } else {
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
