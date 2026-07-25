use std::{
    mem,
    sync::{Arc, Mutex},
};

use crate::{
    Interpreter,
    error::InterpreterError,
    eval::basic::ast::Block,
    lexer::{
        indent::{CmpIndent, LineIndent, OwnedLineIndent},
        line::{Line, LineContext},
        tokenize::TokenNode,
    },
    types::error,
    var::PyValue,
};

pub mod indent;
pub mod line;
pub mod tokenize;

/// Helper struct to build a block of lines with the same base indent. It is used to collect lines
/// that are part of the same block, and to check if the lines are properly indented.
///
/// Always use [`BlockBuilder`] to construct a [`Block`]. [`Block::new`] is not recommended.
#[derive(Debug)]
pub struct BlockBuilder {
    /// The base indent of the block. Every line in the block must have an indent that is greater
    /// than or equal to this base indent.
    ///
    /// If this is [`None`], it means that no lines have been added to the block yet. In this case,
    /// building a block will fail with an error.
    pub base_indent: Option<OwnedLineIndent>,

    /// Lines that are part of the block.
    pub block_lines: Vec<Line>,
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockBuilder {
    /// Creates a new [`BlockBuilder`] with no lines and no base indent.
    pub fn new() -> Self {
        Self {
            base_indent: None,
            block_lines: Vec::new(),
        }
    }

    /// Push a line to the block. If the line's indent is less than the base indent, an error is
    /// returned.
    ///
    /// It will set the base indent to the line's indent if the base indent is [`None`].
    ///
    /// ## Errors
    ///
    /// - [`InterpreterError::FinishedBlock`] if the line's indent is less than the base indent.
    pub fn push_line(&mut self, line: Line) -> Result<(), InterpreterError> {
        if self.base_indent.is_none() {
            self.base_indent = Some(line.indent.clone());
        }
        if let CmpIndent::Less(_) = line
            .indent
            .as_slice()
            .cmp_level(self.base_indent.as_ref().unwrap())?
        {
            // The line is less indented than the base indent, this is an error.
            return Err(InterpreterError::FinishedBlock(line));
        }
        self.block_lines.push(line);
        Ok(())
    }

    /// Construct a [`Block`] from [`BlockBuilder`]. If the block is empty, an error is returned.
    pub fn build_block(self) -> Result<Block, String> {
        if self.block_lines.is_empty() {
            return Err("Cannot build a block from an empty set of lines.".to_string());
        }
        let base_indent = self.base_indent.ok_or("Base indent is not set.")?;
        let block = Block::new(base_indent, self.block_lines);
        Ok(block)
    }
}

/// The result of lexing a line.
///
/// It gives the tokens of the line. If the line marks the end of a block, [`Some(Block)`] will be
/// set.
///
/// ## Example
///
/// ```text
/// # lexing the following lines will return a `LexTokens` with `block` set to `Some(Block)`
/// a = 1
/// if a > 0:
///   print(a)
/// b = 1   # Returns `LexTokens` with `block` set to `Some(Block)` which contains `print(0)`
///         # and tokens for `b = 1`
/// ```
///
/// [`Some(Block)`]: Some
#[derive(Debug, Clone)]
pub struct LexTokens {
    /// The [`TokenNode`]s produced from this line.
    pub tokens: Vec<TokenNode>,

    /// [`Some`] if the line marks the end of a block, [`None`] otherwise.
    pub block: Option<Block>,
}

impl LexTokens {
    pub fn new(tokens: Vec<TokenNode>, block: Option<Block>) -> Self {
        Self { tokens, block }
    }
}

impl Default for LexTokens {
    fn default() -> Self {
        Self::new(vec![], None)
    }
}

/// Persistent state for the lexer across successive input lines.
///
/// Tracks buffered input that has not yet formed a complete [`Line`], and the incomplete
/// [`BlockBuilder`] that accumulates indented lines until the block is closed.
#[derive(Debug)]
pub struct LexContext {
    line_context: Mutex<LineContext>,
    block_context: Mutex<BlockBuilder>,
}

impl Default for LexContext {
    fn default() -> Self {
        Self::new()
    }
}

impl LexContext {
    pub fn new() -> Self {
        Self {
            line_context: Mutex::new(LineContext::new()),
            block_context: Mutex::new(BlockBuilder::new()),
        }
    }

    /// Take the [`BlockBuilder`] and replace it with a new one.
    pub fn get_and_clear_block_context(&self) -> BlockBuilder {
        let mut block_context = self.block_context.lock().unwrap();
        mem::take(&mut *block_context)
    }
}

/// Lexes a single [`Line`] and returns its tokens, along with any completed [`Block`].
///
/// `indent_base` is the expected indentation level for lines at the current scope. A line
/// whose indent exactly matches `indent_base` is tokenized directly and returned. A line
/// indented *beyond* `indent_base` belongs to a nested block and is buffered in
/// [`LexContext`] instead; in that case an empty [`LexTokens`] is returned. Once a
/// less-indented line arrives, the buffered lines are packaged into a [`Block`] and attached
/// to that line's [`LexTokens`].
///
/// For example:
///
/// ```mini-py
/// if a > 0:
///   print(a)  # indent > indent_base  -> buffered, returns empty LexTokens
/// b = 1       # indent == indent_base -> tokenized; completed block attached to LexTokens
/// ```
pub fn lex_line(
    interpreter: Arc<Interpreter>,
    line: Line,
    indent_base: LineIndent,
) -> Result<LexTokens, Arc<dyn PyValue>> {
    let tokens = match tokenize::tokenize(line, indent_base) {
        Ok(tokens) => tokens,
        Err(InterpreterError::UnfinishedBlock(line)) => {
            interpreter
                .get_lex_context()
                .block_context
                .lock()
                .unwrap()
                // When push_line returns an error, it means that the line is less indented than
                // the base indent, which is a syntax error. We should return a syntax error in
                // this case.
                .push_line(line)
                .map_err(|e| match e {
                    InterpreterError::FinishedBlock(line) => error::get_syntax_error(
                        interpreter.clone(),
                        format!(
                            "Expect indented block, but got less indented line: {:?}",
                            line.indent
                        ),
                    ),
                    _ => unreachable!(
                        "Unexpected error when pushing line to block context: {:?}",
                        e
                    ),
                })?;

            return Ok(LexTokens::default());
        }
        Err(err) => {
            return Err(error::get_syntax_error(
                interpreter.clone(),
                err.to_string(),
            ));
        }
    };

    let mut tokens = LexTokens::new(tokens, None);

    if let Ok(block) = interpreter
        .get_lex_context()
        .get_and_clear_block_context()
        .build_block()
    {
        tokens.block = Some(block);
    }

    Ok(tokens)
}

/// Same as [`lex_line`], but accepts a raw `&str` instead of a [`Line`].
///
/// Returns `Ok(LexTokens::default())` if the input has not yet formed a complete line,
/// for example when brackets are still open across multiple input chunks.
pub fn lex_raw_line(
    interpreter: Arc<Interpreter>,
    new_line: &str,
    indent_base: LineIndent,
) -> Result<LexTokens, Arc<dyn PyValue>> {
    // Convert the raw string to a `Line` and delegate to `lex_line`.
    let line = line::get_line(&interpreter.get_lex_context().line_context, new_line)
        .map_err(|e| error::get_syntax_error(interpreter.clone(), e.to_string()))?;

    if let Some(line) = line {
        return lex_line(interpreter, line, indent_base);
    }

    Ok(LexTokens::default())
}
