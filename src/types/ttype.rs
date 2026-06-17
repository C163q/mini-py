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

/// For cyclic reference of type, `Weak` is only used for internal reference, and `Arc` is used for
/// external reference.
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

#[derive(Debug)]
struct PyTypeInner {
    pub name: String,
    pub vars: Mutex<VarManager>,
    pub mro: Vec<Arc<PyType>>,
}

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

    pub fn add_function(&self, name: &str, func: PyFunction) {
        self.inner.vars.lock().unwrap().map.insert(
            name.to_string(),
            Var::new(Arc::new(func), PyGetSetDef::default()),
        );
    }

    pub fn get_name(&self) -> &str {
        &self.inner.name
    }

    pub fn get_mro(&self) -> &[Arc<PyType>] {
        &self.inner.mro
    }

    pub fn __str__(&self, interpreter: Arc<Interpreter>, _values: Vec<Arc<dyn PyValue>>) -> PyStr {
        PyStr::new(interpreter, "<Type>".to_string())
    }
}
