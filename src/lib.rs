use std::{
    collections::{HashMap, hash_map::Entry},
    io::{self, Write},
    sync::{Arc, Mutex},
};

use crate::{
    error::InterpreterError,
    lexer::line::LineContext,
    types::{PyType, tstr::PyStr},
};

pub mod error;
pub mod eval;
pub mod lexer;
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
        let output = eval::eval_line(self.clone(), line);
        let output = match output {
            Ok(value) => value,
            Err(err) => {
                let str_func = match err.get_var(self.clone(), "__str__") {
                    Ok(func) => func,
                    Err(_) => {
                        // Handle the case where the error does not have a __str__ method
                        return Err(InterpreterError::new(format!(
                            "Error does not have __str__ method: {}",
                            err.get_type().get_name()
                        )));
                    }
                };
                let err_msg = match var::call::call(str_func, self.clone(), vec![err.clone()]) {
                    Ok(msg) => match msg.as_any().downcast_ref::<PyStr>() {
                        Some(py_str) => py_str.to_string(),
                        None => {
                            // Handle the case where __str__ does not return a string
                            return Err(InterpreterError::new(format!(
                                "__str__ did not return a string for error: {}",
                                err.get_type().get_name()
                            )));
                        }
                    },
                    Err(err) => {
                        // Handle the case where calling __str__ on the error fails
                        return Err(InterpreterError::new(format!(
                            "Failed to call __str__ on error: {}",
                            err.get_type().get_name()
                        )));
                    }
                };
                eprintln!("{}", err_msg);
                return Ok(());
            }
        };
        if let Some(output) = output
            && let Some(repl_output) = &self.repl_output
        {
            writeln!(repl_output.lock(), "{}", output).ok();
        }
        Ok(())
    }
}
