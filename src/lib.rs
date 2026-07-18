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
        namespace::{Binding, Namespace},
    },
};

pub mod error;
pub mod eval;
pub mod lexer;
pub mod types;
pub mod var;

/// A mini-Python interpreter.
///
/// It manages the variables (types are treated as variables), the lexer context, and the semantic
/// context.
///
/// Designed to be thread-safe; multiple threads may access and modify the interpreter concurrently
/// (not implemented yet). [`Arc<Self>`] is recommended wrapper when sharing across threads.
///
/// The interpreter **must** be initialized with built-in types and functions before use; otherwise,
/// numbers, strings, and other built-in types will not be available. Call
/// [`Interpreter::init_builtin_types`] to perform this initialization.
pub struct Interpreter {
    initialized: AtomicBool,
    var_mapper: Mutex<Namespace>,
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
    /// Creates an interpreter with built-in types initialized.
    pub fn new() -> Self {
        let interpreter = Self::new_arc();
        // Safe: no other `Arc` references exist since we just created this.
        Arc::into_inner(interpreter).unwrap()
    }

    /// Creates an [`Arc`]-wrapped interpreter with built-in types initialized.
    pub fn new_arc() -> Arc<Self> {
        let interpreter = Arc::new(Self::build());
        interpreter.clone().init_builtin_types();
        interpreter
    }

    /// Creates an interpreter without initializing the built-in types.
    ///
    /// Use this when you need to configure the interpreter before initialization
    /// (e.g., enabling REPL output before wrapping in [`Arc`]).
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use minipy::Interpreter;
    ///
    /// let mut interpreter = Interpreter::build();
    /// interpreter.open_repl_output();
    /// let interpreter = Arc::new(interpreter);
    /// interpreter.clone().init_builtin_types();
    /// ```
    pub fn build() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            var_mapper: Mutex::new(Namespace::new()),
            sem_context: Mutex::new(SemContext::new()),
            lex_context: LexContext::new(),
            repl_output: None,
        }
    }

    /// Initializes the built-in types and functions. This should be called before using the
    /// interpreter.
    pub fn init_builtin_types(self: Arc<Self>) {
        if self.initialized.load(atomic::Ordering::SeqCst) {
            return;
        }
        types::init::register_types(self.clone());
        types::init::register_functions(self.clone())
            .expect("Failed to register built-in functions");
        AtomicBool::store(&self.initialized, true, atomic::Ordering::SeqCst);
    }

    /// Registers a new type with the interpreter. If a type with the same name already exists, an
    /// error is returned.
    ///
    /// This is pretty much the same as [`Interpreter::set_var`], but it is more convenient to use
    /// for defining new types.
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
                entry.insert(Binding::new(ty.clone(), PyGetSetDef::default()));
                Ok(ty)
            }
        }
    }

    /// Opens the REPL output stream. If the REPL output stream is already open, this does nothing.
    ///
    /// After calling, the result of evaluating a line will be printed to the REPL output stream.
    /// By default, it is the [`Stdout`].
    ///
    /// [`Stdout`]: std::io::Stdout
    pub fn open_repl_output(&mut self) {
        if self.repl_output.is_none() {
            self.repl_output = Some(io::stdout());
        }
    }

    /// Returns the lexical context of the interpreter.
    pub fn get_lex_context(&self) -> &LexContext {
        &self.lex_context
    }

    /// Looks up a [`PyType`] by name.
    ///
    /// This is similar to [`Interpreter::get_var`], but additionally verifies that the resolved
    /// variable is a type.
    ///
    /// ## Errors
    ///
    /// - `NameError` if no variable with the given name exists.
    /// - `TypeError` if the variable exists but is not a [`PyType`].
    pub fn get_type(self: Arc<Self>, name: &str) -> Result<Arc<PyType>, Arc<dyn PyValue>> {
        let var: Option<Binding> = self
            .var_mapper
            .lock()
            .unwrap() // Namespace
            .get_mapper()
            .get(name) // Option<&Binding>
            .cloned();

        let var = match var {
            Some(var) => var,
            None => {
                // An uninitialized interpreter has no built-in types, so any lookup failure
                // at this stage is a programming error rather than a recoverable runtime error.
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

        // Check if the variable is a PyType. If not, return a TypeError.
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

    /// Looks up a variable by name.
    ///
    /// ## Errors
    ///
    /// - `NameError` if no variable with the given name exists.
    pub fn get_var(self: Arc<Self>, name: &str) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        let var = self
            .var_mapper
            .lock()
            .unwrap() // Namespace
            .get_mapper()
            .get(name) // Option<Binding>
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

    /// Assigns a value to a variable by name, overwriting any existing binding.
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
                vacant.insert(Binding::new(value, PyGetSetDef::default()));
                Ok(())
            }
        }
    }

    /// Evaluates a single line of code. If the REPL output stream is open, the result is printed to it.
    ///
    /// ## Errors
    ///
    /// Any evaluation error is currently returned as an unhandled error.
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

    /// Prints `value` to the REPL output stream, if one is open.
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
