use std::{
    collections::{HashMap, hash_map::Entry},
    io::{self, Write},
    sync::{Arc, Mutex},
};

use crate::{error::InterpreterError, lexer::line::LineContext, types::PyType};

pub mod error;
pub mod eval;
pub mod lexer;
pub mod meta;
pub mod types;
pub mod var;

pub struct Interpreter {
    type_mapper: Mutex<HashMap<String, Arc<PyType>>>,
    line_context: Mutex<LineContext>,
    repl_output: Option<io::Stdout>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    pub fn new() -> Self {
        let interpreter = Self::new_arc();
        // Never panic
        Arc::into_inner(interpreter).unwrap()
    }

    pub fn new_arc() -> Arc<Self> {
        let interpreter = Arc::new(Self::build());
        interpreter.clone().init_builtin_types();
        interpreter
    }

    /// Get the interpreter without initializing the built-in types.
    pub fn build() -> Self {
        Self {
            type_mapper: Mutex::new(HashMap::new()),
            line_context: Mutex::new(LineContext::new()),
            repl_output: None,
        }
    }

    pub fn init_builtin_types(self: Arc<Self>) {
        types::init::register_types(self);
    }

    /// Registers a new type with the interpreter. If a type with the same name already exists, an
    /// error is returned.
    pub fn register_type(
        self: Arc<Self>,
        ty: Arc<PyType>,
    ) -> Result<Arc<PyType>, InterpreterError> {
        let mut type_mapper = self.type_mapper.lock().unwrap();
        match type_mapper.entry(ty.get_name().to_string()) {
            Entry::Occupied(_) => Err(InterpreterError::new(String::from(
                "Type already registered",
            ))),
            Entry::Vacant(entry) => Ok(entry.insert(ty).clone()),
        }
    }

    pub fn open_repl_output(&mut self) {
        if self.repl_output.is_none() {
            self.repl_output = Some(io::stdout());
        }
    }

    pub fn get_type(&self, name: &str) -> Option<Arc<PyType>> {
        self.type_mapper.lock().unwrap().get(name).cloned()
    }

    pub fn eval_line(self: Arc<Self>, line: &str) -> Result<(), InterpreterError> {
        let output = eval::eval_line(self.clone(), line)?;
        if let Some(output) = output
            && let Some(repl_output) = &self.repl_output
        {
            writeln!(repl_output.lock(), "{}", output).ok();
        }
        Ok(())
    }
}
