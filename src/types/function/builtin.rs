use std::sync::Arc;

use crate::{
    Interpreter,
    eval::output,
    types::{function::PyFunction, none::PyNone},
    var::PyValue,
};

pub fn register_print(interpreter: Arc<Interpreter>) -> Result<(), Arc<dyn PyValue>> {
    fn print(
        interpreter: Arc<Interpreter>,
        values: Vec<Arc<dyn PyValue>>,
    ) -> Result<Arc<dyn PyValue>, Arc<dyn PyValue>> {
        for (i, value) in values.into_iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            let str = output::output_value(interpreter.clone(), value)?;
            print!("{}", str);
        }
        Ok(Arc::new(PyNone::new(interpreter)))
    }

    let function = Arc::new(PyFunction::new(interpreter.clone(), Arc::new(print)));
    interpreter.set_var("print", function)
}
