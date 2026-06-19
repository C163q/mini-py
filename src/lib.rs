use std::{
    collections::hash_map::Entry,
    io::{self, Write},
    sync::{
        Arc, Mutex,
        atomic::{self, AtomicBool},
    },
};

use crate::{
    error::InterpreterError,
    lexer::line::LineContext,
    types::{PyType, tstr::PyStr},
    var::{
        PyValue,
        getset::PyGetSetDef,
        manager::{Var, VarManager},
    },
};

pub mod error;
pub mod eval;
pub mod lexer;
pub mod types;
pub mod var;

pub struct Interpreter {
    initialized: AtomicBool,
    var_mapper: Mutex<VarManager>,
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
            initialized: AtomicBool::new(false),
            var_mapper: Mutex::new(VarManager::new()),
            line_context: Mutex::new(LineContext::new()),
            repl_output: None,
        }
    }

    pub fn init_builtin_types(self: Arc<Self>) {
        types::init::register_types(self.clone());
        AtomicBool::store(&self.initialized, true, atomic::Ordering::SeqCst);
    }

    /// Registers a new type with the interpreter. If a type with the same name already exists, an
    /// error is returned.
    pub fn register_type(
        self: Arc<Self>,
        ty: Arc<PyType>,
    ) -> Result<Arc<PyType>, InterpreterError> {
        let mut var_mapper = self.var_mapper.lock().unwrap();
        match var_mapper.get_mapper_mut().entry(ty.get_name().to_string()) {
            Entry::Occupied(_) => Err(InterpreterError::new(String::from(
                "Type already registered",
            ))),
            Entry::Vacant(entry) => {
                entry.insert(Var::new(ty.clone(), PyGetSetDef::default()));
                Ok(ty)
            }
        }
    }

    pub fn open_repl_output(&mut self) {
        if self.repl_output.is_none() {
            self.repl_output = Some(io::stdout());
        }
    }

    pub fn get_type(self: Arc<Self>, name: &str) -> Result<Arc<PyType>, Arc<dyn PyValue>> {
        let var = self
            .var_mapper
            .lock()
            .unwrap() // VarManager
            .get_mapper()
            .get(name) // Option<&Var>
            .cloned();

        let var = match var {
            Some(var) => var,
            None => {
                if !self.initialized.load(atomic::Ordering::SeqCst) {
                    panic!("Interpreter is not initialized. Cannot get type '{}'", name);
                }

                return Err(types::error::get_name_error(
                    self.clone(),
                    format!("name '{}' is not defined", name),
                ));
            }
        }
        .get(self.clone())?; // Arc<dyn PyValue>

        match var.as_arc_any().downcast::<PyType>() {
            Ok(ty) => Ok(ty),
            Err(_) => {
                if !self.initialized.load(atomic::Ordering::SeqCst) {
                    panic!("Interpreter is not initialized. Cannot get type '{}'", name);
                }

                Err(types::error::get_type_error(
                    self.clone(),
                    format!("name '{}' is not a type", name),
                ))
            }
        }
    }

    pub fn get_var(self: Arc<Self>, name: &str) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        let var = self
            .var_mapper
            .lock()
            .unwrap() // VarManager
            .get_mapper()
            .get(name) // Option<Var>
            .cloned();

        let var = var
            .ok_or_else(|| {
                types::error::get_name_error(
                    self.clone(),
                    format!("name '{}' is not defined", name),
                )
            })?
            .get(self.clone())?; // Arc<dyn PyValue>

        Ok(var)
    }

    pub fn set_var(
        self: Arc<Self>,
        name: &str,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        let interp = self.clone();

        let mut lock = interp.var_mapper.lock().unwrap();
        let mapper = lock.get_mapper_mut();

        match mapper.entry(name.to_string()) {
            Entry::Occupied(mut occupied) => occupied.get_mut().set(self, value),
            Entry::Vacant(vacant) => {
                vacant.insert(Var::new(value, PyGetSetDef::default()));
                Ok(())
            }
        }
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
