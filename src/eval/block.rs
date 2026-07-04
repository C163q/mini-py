use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{ast::Block, eval_line_with_indent},
    lexer,
    var::PyValue,
};

pub fn eval_block(interpreter: Arc<Interpreter>, block: Block) -> Result<(), Arc<dyn PyValue>> {
    let indent = block.base_indent.as_slice();
    for line in block.lines {
        let py_str = eval_line_with_indent(interpreter.clone(), line, indent, lexer::lex_line)?;
        if let Some(py_str) = py_str {
            interpreter.clone().output_pystr_if_repl(py_str)?;
        }
    }

    Ok(())
}
