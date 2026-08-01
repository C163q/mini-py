use std::{error::Error, fmt::Display, sync::Arc};

use crate::{lexer::line::Line, var::PyValue};

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

#[derive(Debug, Clone, Copy)]
pub enum PyControlFlow {
    Break,
    Continue,
}

impl Display for PyControlFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Break => write!(f, "break"),
            Self::Continue => write!(f, "continue"),
        }
    }
}

/// TODO: add InterpreterError to PyError
#[derive(Debug, Clone)]
pub enum PyError {
    ControlFlow(PyControlFlow),
    Exception(Arc<dyn PyValue>),
}

impl PyError {
    pub fn new_control_flow(control_flow: PyControlFlow) -> Self {
        Self::ControlFlow(control_flow)
    }

    pub fn new_break() -> Self {
        Self::ControlFlow(PyControlFlow::Break)
    }

    pub fn new_continue() -> Self {
        Self::ControlFlow(PyControlFlow::Continue)
    }

    pub fn new_exception(exception: Arc<dyn PyValue>) -> Self {
        Self::Exception(exception)
    }

    pub fn into_exception(self) -> Option<Arc<dyn PyValue>> {
        match self {
            Self::Exception(v) => Some(v),
            Self::ControlFlow(_) => None,
        }
    }
}

impl Display for PyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ControlFlow(control_flow) => write!(f, "Control flow: {:?}", control_flow),
            Self::Exception(exception) => write!(f, "Exception: {:?}", exception),
        }
    }
}

impl Error for PyError {}

impl From<Arc<dyn PyValue>> for PyError {
    fn from(exception: Arc<dyn PyValue>) -> Self {
        Self::new_exception(exception)
    }
}
