use std::sync::Arc;

use crate::{
    Interpreter,
    error::InterpreterError,
    eval::ast::Block,
    lexer::{
        indent::Indent,
        line::Line,
        tokenize::{Token, TokenKind},
    },
    types::error,
    var::PyValue,
};

pub mod indent;
pub mod line;
pub mod tokenize;

pub struct BlockBuilder {
    pub base_indent: Option<Vec<Indent>>,
    pub block_lines: Vec<Line>,
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockBuilder {
    pub fn new() -> Self {
        Self {
            base_indent: None,
            block_lines: Vec::new(),
        }
    }

    pub fn push_line(&mut self, line: Line) {
        if self.base_indent.is_none() {
            self.base_indent = Some(line.indent.clone());
        }
        self.block_lines.push(line);
    }

    pub fn build_block(self) -> Result<Block, String> {
        if self.block_lines.is_empty() {
            return Err("Cannot build a block from an empty set of lines.".to_string());
        }
        let base_indent = self.base_indent.ok_or("Base indent is not set.")?;
        let block = Block::new(base_indent, self.block_lines);
        Ok(block)
    }
}

impl From<Vec<Line>> for BlockBuilder {
    fn from(lines: Vec<Line>) -> Self {
        Self {
            base_indent: lines.first().map(|line| line.indent.clone()),
            block_lines: lines,
        }
    }
}

pub fn lex_line(
    interpreter: Arc<Interpreter>,
    new_line: &str,
) -> Result<Vec<Token>, Arc<dyn PyValue>> {
    lex_line_with_indent(interpreter, new_line, &[])
}

/// This function may return [Block, <other tokens>] if last line was an unfinished block. In that
/// case, the block is returned as the first token, and the rest of the tokens are returned as the
/// rest of the vector.
///
/// Make sure to check if the first token is a block, and if so, handle it accordingly.
pub fn lex_line_with_indent(
    interpreter: Arc<Interpreter>,
    new_line: &str,
    indent_base: &[Indent],
) -> Result<Vec<Token>, Arc<dyn PyValue>> {
    let line = line::get_line(&interpreter.line_context, new_line)
        .map_err(|e| error::get_syntax_error(interpreter.clone(), e.to_string()))?;

    if let Some(line) = line {
        let mut tokens = tokenize::tokenize(line, indent_base).map_err(|e| match e {
            InterpreterError::UnfinishedBlock(line) => {
                interpreter.block_context.lock().unwrap().push_line(line);
                error::get_syntax_error(
                    interpreter.clone(),
                    "Unfinished block detected.".to_string(),
                )
            }
            e => error::get_syntax_error(interpreter.clone(), e.to_string()),
        })?;

        if let Ok(block) = interpreter.get_and_clear_block_context().build_block() {
            tokens.insert(0, Token::new(TokenKind::new_block(block)));
        }

        return Ok(tokens);
    }

    Ok(vec![])
}
