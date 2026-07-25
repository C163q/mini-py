use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{basic::ast::Block, eval_line_finished, eval_line_with_indent},
    lexer,
    var::PyValue,
};

/// Evaluates every line in `block` in order, using the block's own indentation as the base.
///
/// In REPL mode, any line that produces output is printed immediately via
/// [`Interpreter::output_pystr_if_repl`].
pub fn eval_block(interpreter: Arc<Interpreter>, block: Block) -> Result<(), Arc<dyn PyValue>> {
    let indent = block.base_indent.as_slice();
    for line in block.lines {
        let py_str = eval_line_with_indent(interpreter.clone(), line, indent, lexer::lex_line)?;
        if let Some(py_str) = py_str {
            interpreter.clone().output_pystr_if_repl(py_str)?;
        }
    }

    let py_str = eval_line_finished(interpreter.clone())?;
    if let Some(py_str) = py_str {
        interpreter.output_pystr_if_repl(py_str)?;
    }

    Ok(())
}
