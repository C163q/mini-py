use std::{mem, sync::Arc};

use crate::{
    Interpreter,
    eval::{Eval, ParseResult, SetBlock, ast::IfStmt},
    lexer::tokenize::{Keyword, Separator, Token, TokenNode},
    types::{error, tbool::PyBool},
    var::PyValue,
};

impl SetBlock for IfStmt {
    fn set_block(&mut self, block: crate::eval::ast::Block) {
        assert!(self.body.is_none(), "Block is already set for this IfStmt");
        self.body = Some(block);
    }
}

impl Eval for IfStmt {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval_if(interpreter, *self).map(|_| None)
    }

    fn eval_with_state(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        let result = self.eval(interpreter.clone());
        {
            let mut lock = interpreter.sem_context.lock().unwrap();
            let state = lock.get_sem_state_mut();
            let last_state = mem::take(state);
            state.last_if_result = last_state.last_if_result;
        }
        result
    }
}

/// Attempts to parse `if <condition> :` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match an `if` header. The returned
/// [`IfStmt`] has no body yet; the body is attached later via [`SetBlock::set_block`].
pub fn parse_if(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<IfStmt>> {
    // idx + 1 is the index of the next token after 'if'
    if idx + 1 >= tokens.len() {
        return None;
    }

    if tokens[idx].value == Token::Keyword(Keyword::If)
        && let Some(condition) = super::expr::parse_expr(interpreter.clone(), tokens, idx + 1)
        && condition.idx < tokens.len()
        && tokens[condition.idx].value == Token::Separator(Separator::Colon)
    {
        return Some(ParseResult::new(
            condition.idx + 1,
            IfStmt::new(condition.value),
        ));
    }

    None
}

/// Evaluates an [`IfStmt`]: evaluates the condition, records the result in [`SemState`] for
/// a potential `else` branch, and executes the body block if the condition is truthy.
///
/// [`SemState`]: crate::eval::SemState
pub fn eval_if(interpreter: Arc<Interpreter>, if_stmt: IfStmt) -> Result<(), Arc<dyn PyValue>> {
    let cond = super::expr::eval_expr(interpreter.clone(), if_stmt.condition)?;
    let func = cond.get_binding(interpreter.clone(), "__bool__")?;
    let cond = match crate::var::call::call(func, interpreter.clone(), vec![cond])?
        .as_any()
        .downcast_ref::<PyBool>()
    {
        None => {
            return Err(error::get_type_error(
                interpreter,
                "__bool__ did not return a boolean".to_string(),
            ));
        }
        Some(b) => b.get_value(),
    };

    // TODO: eval Block if cond is true
    if cond {
        super::block::eval_block(
            interpreter.clone(),
            if_stmt.body.expect("IfStmt body is None"),
        )?;
    }

    interpreter.sem_context.lock().unwrap().state.last_if_result = Some(cond);

    Ok(())
}
