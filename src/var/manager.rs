use std::{collections::HashMap, sync::Arc};

use crate::{
    Interpreter,
    var::{PyValue, getset::PyGetSetDef},
};

#[derive(Debug, Clone)]
pub struct Var {
    value: Arc<dyn PyValue>,
    pub getset: PyGetSetDef,
}

impl Var {
    pub fn new(value: Arc<dyn PyValue>, getset: PyGetSetDef) -> Self {
        Self { value, getset }
    }

    pub fn get(&self, interpreter: Arc<Interpreter>) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        self.getset.get(interpreter, &self.value)
    }

    pub fn set(
        &mut self,
        interpreter: Arc<Interpreter>,
        value: Arc<dyn PyValue>,
    ) -> Result<(), Arc<dyn PyValue>> {
        self.getset.set(interpreter, &mut self.value, value)
    }
}

#[derive(Debug, Clone)]
pub struct VarManager {
    pub map: HashMap<String, Var>,
}

impl Default for VarManager {
    fn default() -> Self {
        Self::new()
    }
}

impl VarManager {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get_mapper(&self) -> &HashMap<String, Var> {
        &self.map
    }

    pub fn get_mapper_mut(&mut self) -> &mut HashMap<String, Var> {
        &mut self.map
    }
}
