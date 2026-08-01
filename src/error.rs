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

/// A `break` or `continue` that is propagating out of a loop body.
///
/// This is carried inside [`PyError`] so that `break`/`continue` can be threaded up through
/// [`Eval::eval`] using the same `Result::Err` path as exceptions, without having to add a
/// separate return channel to every `eval` implementation. The statement that owns the loop
/// (e.g. [`WhileStmt`]) is responsible for catching this variant and turning it into actual
/// control flow; every other caller should let it propagate.
///
/// [`Eval::eval`]: crate::eval::Eval::eval
/// [`WhileStmt`]: crate::eval::stmt::ast::WhileStmt
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

/// The error type returned by [`Eval::eval`].
///
/// Wraps two unrelated reasons an evaluation can fail to produce a value: an actual Python
/// exception ([`PyError::Exception`]), or a `break`/`continue` ([`PyError::ControlFlow`]) that
/// still needs to travel up to its enclosing loop. Bundling both into one error type lets
/// `break`/`continue` reuse `?` through ordinary statement evaluation instead of a bespoke
/// return type; callers that are not a loop should propagate this value unchanged rather than
/// inspecting it, so that it reaches the loop that can actually handle it.
///
/// TODO: add InterpreterError to PyError
///
/// [`Eval::eval`]: crate::eval::Eval::eval
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

    /// Returns the wrapped exception, or `None` if this is a [`PyError::ControlFlow`].
    ///
    /// Useful at the boundary between the interpreter core and its callers (e.g. the REPL),
    /// which only ever expect to see exceptions — a `break`/`continue` reaching that boundary
    /// means it escaped its loop and should have been rejected earlier as a syntax error.
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
