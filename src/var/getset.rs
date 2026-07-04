use std::{
    fmt::Debug,
    sync::{Arc, LazyLock},
};

use crate::{Interpreter, types::error, var::PyValue};

pub type Getter = Arc<
    dyn Fn(Arc<Interpreter>, &Arc<dyn PyValue>) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>>
        + 'static
        + Send
        + Sync,
>;
pub type Setter = Arc<
    dyn Fn(
            Arc<Interpreter>,
            &mut Arc<dyn PyValue>,
            Arc<dyn PyValue>,
        ) -> Result<(), Arc<dyn PyValue>>
        + 'static
        + Send
        + Sync,
>;

fn default_getter(
    _interpreter: Arc<Interpreter>,
    target: &Arc<dyn PyValue>,
) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
    Ok(target.clone())
}

fn default_setter(
    _interpreter: Arc<Interpreter>,
    target: &mut Arc<dyn PyValue>,
    value: Arc<dyn PyValue>,
) -> Result<(), Arc<dyn PyValue>> {
    *target = value;
    Ok(())
}

static DEFAULT_GETTER: LazyLock<Getter> = LazyLock::new(|| Arc::new(default_getter));
static DEFAULT_SETTER: LazyLock<Setter> = LazyLock::new(|| Arc::new(default_setter));

#[derive(Clone)]
pub struct PyGetSetDef {
    getter: Option<Getter>,
    setter: Option<Setter>,
}

impl Default for PyGetSetDef {
    fn default() -> Self {
        Self {
            getter: Some(DEFAULT_GETTER.clone()),
            setter: Some(DEFAULT_SETTER.clone()),
        }
    }
}

impl Debug for PyGetSetDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<PyGetSetDef getter={:?} setter={:?}>",
            self.getter.is_some(),
            self.setter.is_some()
        )
    }
}

impl PyGetSetDef {
    pub fn new(getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self { getter, setter }
    }

    pub fn get(
        &self,
        interpreter: Arc<Interpreter>,
        target: &Arc<dyn PyValue>,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        match &self.getter {
            Some(getter) => getter(interpreter, target),
            None => Err(error::get_attribute_error(
                interpreter,
                "Attribute is not readable".to_string(),
            )),
        }
    }

    pub fn set(
        &self,
        interpreter: Arc<Interpreter>,
        target: &mut Arc<dyn PyValue>,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        match &self.setter {
            Some(setter) => setter(interpreter, target, value),
            None => Err(error::get_attribute_error(
                interpreter,
                "Attribute is not writable".to_string(),
            )),
        }
    }
}
