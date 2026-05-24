use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::func::PyFunction;

pub mod float;
pub mod init;
pub mod int;
pub mod none;
pub mod tbool;
pub mod tstr;

#[derive(Debug)]
pub struct PyType {
    name: String,
    functions: Mutex<HashMap<String, Arc<PyFunction>>>,
}

impl PyType {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_function(&self, name: &str, func: PyFunction) {
        self.functions
            .lock()
            .unwrap()
            .insert(name.to_string(), Arc::new(func));
    }

    pub fn get_function(&self, name: &str) -> Option<Arc<PyFunction>> {
        self.functions.lock().unwrap().get(name).cloned()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}
