use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::{
    Interpreter, get_type,
    types::{function::PyFunction, init::PyFunctionMapper, tstr::PyStr},
    var::{
        PyValue,
        getset::PyGetSetDef,
        manager::{Var, VarManager},
    },
};

const TYPE_NAME: &str = "type";

get_type!(TYPE_NAME);

pub fn init_raw_type(interpreter: Arc<Interpreter>) {
    type Current = PyType;

    // DO NOT Call init::init_type() here, it will cause circular dependency

    let inner = Arc::new(PyTypeInner {
        name: TYPE_NAME.to_string(),
        vars: Mutex::new(VarManager::new()),
        mro: vec![],
    });
    let mut ty = Current {
        inner: inner.clone(),
        ty: ArcOrWeak::Arc(Arc::new_cyclic(|weak| Current {
            inner,
            ty: ArcOrWeak::Weak(weak.clone()),
        })),
    };
    ty.ty = ArcOrWeak::Arc(ty.ty.into_arc());

    interpreter
        .clone()
        .register_type(Arc::new(ty))
        .expect("Failed to register type");

    // DO NOT add functions here, `PyFunction` is not initialized yet.
}

pub fn init_functions(interpreter: Arc<Interpreter>) {
    let ty = get_type(interpreter.clone());

    #[allow(clippy::single_element_loop)]
    for PyFunctionMapper { name, func } in [PyFunctionMapper::from_method(
        "__str__",
        interpreter.clone(),
        PyType::__str__,
    )] {
        ty.add_function(name, func);
    }
}

/// Holds either a strong or weak reference to a `T`.
///
/// `PyType` is self-referential: its `ty` field points back to `type` (the type of all types).
/// Internally this self-reference is stored as a `Weak` to break the cycle; external callers
/// always receive an `Arc`.
#[derive(Debug)]
enum ArcOrWeak<T> {
    Arc(Arc<T>),
    Weak(Weak<T>),
}

impl<T> ArcOrWeak<T> {
    fn into_arc(self) -> Arc<T> {
        match self {
            ArcOrWeak::Arc(arc) => arc.clone(),
            ArcOrWeak::Weak(weak) => weak.upgrade().expect("Failed to upgrade weak reference"),
        }
    }

    fn arc(self) -> Arc<T> {
        match self {
            ArcOrWeak::Arc(arc) => arc,
            ArcOrWeak::Weak(_) => {
                panic!("Type is not initialized yet, cannot get Arc reference")
            }
        }
    }
}

impl Clone for ArcOrWeak<PyType> {
    fn clone(&self) -> Self {
        match self {
            ArcOrWeak::Arc(arc) => ArcOrWeak::Arc(arc.clone()),
            ArcOrWeak::Weak(weak) => ArcOrWeak::Weak(weak.clone()),
        }
    }
}

/// Heap-allocated data shared among all clones of a [`PyType`].
#[derive(Debug)]
struct PyTypeInner {
    pub name: String,
    pub vars: Mutex<VarManager>,
    pub mro: Vec<Arc<PyType>>,
}

/// The runtime representation of a Python type object (the `type` type and all its instances).
///
/// Every [`PyValue`] carries an `Arc<PyType>` identifying its type. `PyType` itself is also a
/// `PyValue`, so `type` is its own type — the self-reference is managed via [`ArcOrWeak`].
///
/// [`PyValue`]: crate::var::PyValue
#[derive(Debug)]
pub struct PyType {
    ty: ArcOrWeak<PyType>,
    inner: Arc<PyTypeInner>,
}

impl PyValue for PyType {
    fn get_type(&self) -> Arc<PyType> {
        self.ty.clone().arc()
    }

    fn get_var_manager(&self) -> MutexGuard<'_, VarManager> {
        self.inner.vars.lock().unwrap()
    }
}

impl PyType {
    /// Creates a new `PyType` with the given name, looking up its own `type` from the interpreter.
    pub fn new(name: &str, interpreter: Arc<Interpreter>) -> Self {
        Self {
            ty: ArcOrWeak::Arc(get_type(interpreter)),
            inner: Arc::new(PyTypeInner {
                name: name.to_string(),
                vars: Mutex::new(VarManager::new()),
                mro: vec![],
            }),
        }
    }

    /// Registers a method on this type by name.
    pub fn add_function(&self, name: &str, func: PyFunction) {
        self.inner.vars.lock().unwrap().get_mapper_mut().insert(
            name.to_string(),
            Var::new(Arc::new(func), PyGetSetDef::default()),
        );
    }

    /// Returns the name of this type (e.g. `"int"`, `"str"`).
    pub fn get_name(&self) -> &str {
        &self.inner.name
    }

    /// Returns the method resolution order (MRO) of this type.
    pub fn get_mro(&self) -> &[Arc<PyType>] {
        &self.inner.mro
    }

    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "<Type>".to_string())
    }
}
