use std::{error::Error, fmt::Display};

use crate::lexer::line::Line;

#[derive(Debug, Clone)]
pub enum InterpreterError {
    UnhandledError(String),
    UnfinishedBlock(Line),
    LexicalError(String),
}

impl InterpreterError {
    pub fn new_unhandled(message: String) -> Self {
        Self::UnhandledError(message)
    }

    pub fn new_unfinished_block(line: Line) -> Self {
        Self::UnfinishedBlock(line)
    }

    pub fn new_lexical_error(message: String) -> Self {
        Self::LexicalError(message)
    }

    pub fn get_message(&self) -> &str {
        match self {
            Self::UnhandledError(msg) => msg,
            Self::LexicalError(msg) => msg,
            Self::UnfinishedBlock(_) => "Unfinished block detected.",
        }
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl Error for InterpreterError {}
