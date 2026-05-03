use std::{
    collections::{HashMap, hash_map::Entry},
    sync::{Arc, Mutex},
};

use crate::{error::InterpreterError, types::PyType};

pub mod error;
pub mod func;
pub mod interprete;
pub mod types;
pub mod var;

pub struct Interpreter {
    type_mapper: Mutex<HashMap<String, Arc<PyType>>>,
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
        types::init::register_types(interpreter.clone());
        interpreter
    }

    /// Get the interpreter without initializing the built-in types.
    pub fn build() -> Self {
        Self {
            type_mapper: Mutex::new(HashMap::new()),
        }
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

    pub fn get_type(&self, name: &str) -> Option<Arc<PyType>> {
        self.type_mapper.lock().unwrap().get(name).cloned()
    }

    pub fn eval_line(self: Arc<Self>, line: &str) -> Result<(), InterpreterError> {
        // TODO
        Ok(())
    }
}
