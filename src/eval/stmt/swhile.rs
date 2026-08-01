use std::{mem, sync::Arc};

use crate::{
    Interpreter,
    error::{PyControlFlow, PyError},
    eval::{
        Eval, Parse, ParseResult, SetBlock,
        basic::{self, ast::Block},
        expr::{self, ast::Expr},
        stmt::ast::WhileStmt,
    },
    lexer::tokenize::{Keyword, Separator, TokenNode},
    types::{error, tbool::PyBool},
    var::PyValue,
};

impl SetBlock for WhileStmt {
    fn set_block(&mut self, block: Block) {
        assert!(
            self.body.is_none(),
            "Block is already set for this WhileStmt"
        );
        self.body = Some(block);
    }
}

impl Eval for WhileStmt {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, PyError> {
        eval_while(interpreter, *self).map(|_| None)
    }

    fn eval_with_state(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, PyError> {
        let last_in_loop = mem::replace(
            &mut interpreter
                .sem_context
                .lock()
                .unwrap()
                .get_sem_state_mut()
                .in_loop,
            true,
        );

        let result = self.eval(interpreter.clone());
        {
            let mut lock = interpreter.sem_context.lock().unwrap();
            let state = lock.get_sem_state_mut();
            state.reset();
            state.in_loop = last_in_loop;
        }
        result
    }
}

impl Parse for WhileStmt {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<WhileStmt>> {
        parse_while(interpreter, tokens, idx)
    }
}

/// Attempts to parse `while <condition> :` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match a `while` header. The returned
/// [`WhileStmt`] has no body yet; the body is attached later via [`SetBlock::set_block`].
fn parse_while(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<WhileStmt>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(while_kw) = Keyword::parse(interpreter.clone(), tokens, idx)
        && while_kw.value == Keyword::While
        && let Some(condition) = Expr::parse(interpreter.clone(), tokens, while_kw.idx)
        && let Some(colon) = Separator::parse(interpreter.clone(), tokens, condition.idx)
        && colon.value == Separator::Colon
    {
        return Some(ParseResult::new(colon.idx, WhileStmt::new(condition.value)));
    }

    None
}

/// Evaluates an [`WhileStmt`]: evaluates the condition.
fn eval_while(interpreter: Arc<Interpreter>, while_stmt: WhileStmt) -> Result<(), PyError> {
    loop {
        let cond = expr::eval_expr(interpreter.clone(), while_stmt.condition.clone())?;
        let func = cond.get_binding(interpreter.clone(), "__bool__")?;
        let cond = match crate::var::call::call(func, interpreter.clone(), vec![cond])?
            .as_any()
            .downcast_ref::<PyBool>()
        {
            None => {
                return Err(error::get_type_error(
                    interpreter,
                    "__bool__ did not return a boolean".to_string(),
                )
                .into());
            }
            Some(b) => b.get_value(),
        };

        if !cond {
            break;
        }

        if let Err(e) = basic::eval_block(
            interpreter.clone(),
            while_stmt.body.clone().expect("WhileStmt body is None"),
        ) {
            match e {
                PyError::ControlFlow(cf) => match cf {
                    PyControlFlow::Break => break,
                    PyControlFlow::Continue => continue,
                },
                _ => return Err(e),
            }
        }
    }

    Ok(())
}
