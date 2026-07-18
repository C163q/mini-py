use std::{mem, sync::Arc};

use crate::{
    Interpreter,
    eval::{
        Eval, ParseResult, SetBlock,
        ast::{ElifStmt, ElseStmt, IfStmt},
    },
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

    if cond {
        super::block::eval_block(
            interpreter.clone(),
            if_stmt.body.expect("IfStmt body is None"),
        )?;
    }

    // Statements in the body may changes the state, so we need to update the last_if_result after
    // executing the body.
    interpreter.sem_context.lock().unwrap().state.last_if_result = Some(cond);

    Ok(())
}

impl SetBlock for ElifStmt {
    fn set_block(&mut self, block: crate::eval::ast::Block) {
        assert!(
            self.body.is_none(),
            "Block is already set for this ElifStmt"
        );
        self.body = Some(block);
    }
}

impl Eval for ElifStmt {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval_elif(interpreter, *self).map(|_| None)
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

/// Attempts to parse `elif <condition> :` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match an `elif` header. The returned
/// [`ElifStmt`] has no body yet; the body is attached later via [`SetBlock::set_block`].
pub fn parse_elif(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<ElifStmt>> {
    // idx + 1 is the index of the next token after 'elif'
    if idx + 1 >= tokens.len() {
        return None;
    }

    if tokens[idx].value == Token::Keyword(Keyword::Elif)
        && let Some(condition) = super::expr::parse_expr(interpreter.clone(), tokens, idx + 1)
        && condition.idx < tokens.len()
        && tokens[condition.idx].value == Token::Separator(Separator::Colon)
    {
        return Some(ParseResult::new(
            condition.idx + 1,
            ElifStmt::new(condition.value),
        ));
    }

    None
}

/// Evaluates an [`ElifStmt`]: evaluates the condition if the last `if` condition results in false,
/// records the result in [`SemState`] for a potential `else` branch, and executes the body block
/// if the condition is truthy.
///
/// [`SemState`]: crate::eval::SemState
pub fn eval_elif(
    interpreter: Arc<Interpreter>,
    elif_stmt: ElifStmt,
) -> Result<(), Arc<dyn PyValue>> {
    match interpreter.sem_context.lock().unwrap().state.last_if_result {
        Some(true) => {
            // Ingore the condition and body
            Ok(())
        }
        None => Err(error::get_syntax_error(
            interpreter.clone(),
            String::from("An elif statement evaluated without a preceding if statement"),
        )),
        Some(false) => {
            let cond = super::expr::eval_expr(interpreter.clone(), elif_stmt.condition)?;
            let func = cond.get_binding(interpreter.clone(), "__bool__")?;
            let cond = match crate::var::call::call(func, interpreter.clone(), vec![cond])?
                .as_any()
                .downcast_ref::<PyBool>()
            {
                None => {
                    return Err(error::get_type_error(
                        interpreter.clone(),
                        "__bool__ did not return a boolean".to_string(),
                    ));
                }
                Some(b) => b.get_value(),
            };

            if cond {
                super::block::eval_block(
                    interpreter.clone(),
                    elif_stmt.body.expect("ElifStmt body is None"),
                )?;
            }

            interpreter.sem_context.lock().unwrap().state.last_if_result = Some(cond);

            Ok(())
        }
    }
}

impl SetBlock for ElseStmt {
    fn set_block(&mut self, block: crate::eval::ast::Block) {
        assert!(
            self.body.is_none(),
            "Block is already set for this ElseStmt"
        );
        self.body = Some(block);
    }
}

impl Eval for ElseStmt {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval_else(interpreter, *self).map(|_| None)
    }
}

/// Attempts to parse `else :` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match an `else` header. The returned
/// [`ElseStmt`] has no body yet; the body is attached later via [`SetBlock::set_block`].
pub fn parse_else(
    _interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<ElseStmt>> {
    // idx + 1 is the index of the next token after 'else'
    if idx + 1 >= tokens.len() {
        return None;
    }

    if tokens[idx].value == Token::Keyword(Keyword::Else)
        && tokens[idx + 1].value == Token::Separator(Separator::Colon)
    {
        return Some(ParseResult::new(idx + 2, ElseStmt::new()));
    }

    None
}

/// Evaluates an [`ElseStmt`]: executes the body block if the last evaluated `if` condition was
/// false.
pub fn eval_else(
    interpreter: Arc<Interpreter>,
    else_stmt: ElseStmt,
) -> Result<(), Arc<dyn PyValue>> {
    let last_cond = interpreter.sem_context.lock().unwrap().state.last_if_result;
    let last_cond = match last_cond {
        Some(cond) => cond,
        None => {
            return Err(error::get_syntax_error(
                interpreter.clone(),
                String::from("An else statement evaluated without a preceding if statement"),
            ));
        }
    };

    if !last_cond {
        super::block::eval_block(
            interpreter.clone(),
            else_stmt.body.expect("ElseStmt body is None"),
        )?;
    }

    Ok(())
}
