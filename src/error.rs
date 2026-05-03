use std::{error::Error, fmt::Display};

#[derive(Debug, Clone)]
pub struct InterpreterError {
    message: String,
}

impl InterpreterError {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn get_message(&self) -> &str {
        &self.message
    }
}

impl Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for InterpreterError {}
