use std::{
    mem,
    sync::{Arc, Mutex},
};

use crate::{
    Interpreter,
    error::InterpreterError,
    eval::ast::Block,
    lexer::{
        indent::{CmpIndent, LineIndent, OwnedLineIndent},
        line::{Line, LineContext},
        tokenize::Token,
    },
    types::error,
    var::PyValue,
};

pub mod indent;
pub mod line;
pub mod tokenize;

#[derive(Debug)]
pub struct BlockBuilder {
    pub base_indent: Option<OwnedLineIndent>,
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

#[derive(Debug, Clone)]
pub struct LexTokens {
    pub tokens: Vec<Token>,
    pub block: Option<Block>,
}

impl LexTokens {
    pub fn new(tokens: Vec<Token>, block: Option<Block>) -> Self {
        Self { tokens, block }
    }
}

impl Default for LexTokens {
    fn default() -> Self {
        Self::new(vec![], None)
    }
}

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

    pub fn get_and_clear_block_context(&self) -> BlockBuilder {
        let mut block_context = self.block_context.lock().unwrap();
        mem::take(&mut *block_context)
    }
}

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

pub fn lex_raw_line(
    interpreter: Arc<Interpreter>,
    new_line: &str,
    indent_base: LineIndent,
) -> Result<LexTokens, Arc<dyn PyValue>> {
    let line = line::get_line(&interpreter.get_lex_context().line_context, new_line)
        .map_err(|e| error::get_syntax_error(interpreter.clone(), e.to_string()))?;

    if let Some(line) = line {
        return lex_line(interpreter, line, indent_base);
    }

    Ok(LexTokens::default())
}
