use std::{error::Error, fmt::Display};

use crate::lexer::line::Line;

/// The error type for the interpreter.
///
/// ## `Arc<PyValue>` vs InterpreterError
///
/// `Arc<PyValue>` is treated as a Exception in Python, while InterpreterError is treated as an
/// internal error in the interpreter.
#[derive(Debug, Clone)]
pub enum InterpreterError {
    /// Represents unrecoverable errors or errors that haven't implement a way to handle.
    UnhandledError(String),
    UnfinishedBlock(Line),
    FinishedBlock(Line),
    LexicalError(String),
}

impl InterpreterError {
    pub fn new_unhandled(message: String) -> Self {
        Self::UnhandledError(message)
    }

    pub fn new_unfinished_block(line: Line) -> Self {
        Self::UnfinishedBlock(line)
    }

    pub fn new_finished_block(line: Line) -> Self {
        Self::FinishedBlock(line)
    }

    pub fn new_lexical_error(message: String) -> Self {
        Self::LexicalError(message)
    }

    pub fn get_message(&self) -> &str {
        match self {
            Self::UnhandledError(msg) => msg,
            Self::LexicalError(msg) => msg,
            Self::UnfinishedBlock(_) => "Unfinished block detected.",
            Self::FinishedBlock(_) => "Finished block detected. Try again.",
        }
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_message())
    }
}

impl Error for InterpreterError {}
