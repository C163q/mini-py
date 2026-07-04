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
    eval::{SemContext, output},
    lexer::LexContext,
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
    lex_context: LexContext,
    sem_context: Mutex<SemContext>,
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
            sem_context: Mutex::new(SemContext::new()),
            lex_context: LexContext::new(),
            repl_output: None,
        }
    }

    pub fn init_builtin_types(self: Arc<Self>) {
        types::init::register_types(self.clone());
        types::init::register_functions(self.clone())
            .expect("Failed to register built-in functions");
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
            Entry::Occupied(_) => Err(InterpreterError::new_unhandled(String::from(
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

    pub fn get_lex_context(&self) -> &LexContext {
        &self.lex_context
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
                let err_msg = output::output_err_value(self.clone(), err)?;
                return Err(InterpreterError::new_unhandled(format!(
                    "Error evaluating line: {}",
                    err_msg
                )));
            }
        };
        if let Some(output) = output {
            let _ = self.output_pystr_if_repl(output);
        }
        Ok(())
    }

    pub fn output_pystr_if_repl(self: Arc<Self>, value: PyStr) -> Result<(), Arc<dyn PyValue>> {
        if let Some(repl_output) = &self.repl_output {
            writeln!(repl_output.lock(), "{}", value).map_err(|e| {
                types::error::get_runtime_error(
                    self.clone(),
                    format!("Error outputting result: {}", e),
                )
            })?;
        }
        Ok(())
    }
}
