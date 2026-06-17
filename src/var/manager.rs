use std::{collections::HashMap, sync::Arc};

use crate::var::{PyValue, getset::PyGetSetDef};

#[derive(Debug, Clone)]
pub struct Var {
    pub value: Arc<dyn PyValue>,
    pub getset: PyGetSetDef,
}

impl Var {
    pub fn new(value: Arc<dyn PyValue>, getset: PyGetSetDef) -> Self {
        Self { value, getset }
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
}
