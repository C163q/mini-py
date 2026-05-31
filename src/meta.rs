use std::sync::Arc;

use crate::var::PyValue;

pub type Getter = Arc<dyn Fn(Arc<dyn PyValue>) -> Arc<dyn PyValue>>;
pub type Setter = Arc<dyn Fn(Arc<dyn PyValue>, Arc<dyn PyValue>) -> Arc<dyn PyValue>>;

pub struct PyGetSetDef {
    pub name: String,
    pub getter: Option<Getter>,
    pub setter: Option<Setter>,
}

impl PyGetSetDef {
    pub fn new(name: String, getter: Option<Getter>, setter: Option<Setter>) -> Self {
        Self {
            name,
            getter,
            setter,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn get(&self, instance: Arc<dyn PyValue>) -> Option<Arc<dyn PyValue>> {
        self.getter.as_ref().map(|getter| getter(instance))
    }

    pub fn set(
        &self,
        instance: Arc<dyn PyValue>,
        value: Arc<dyn PyValue>,
    ) -> Option<Arc<dyn PyValue>> {
        self.setter.as_ref().map(|setter| setter(instance, value))
    }
}
