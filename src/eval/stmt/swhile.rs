use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{
        Eval, Parse, ParseResult, SetBlock,
        basic::{self, ast::Block},
        expr::{self, ast::Expr},
    },
    lexer::tokenize::{Keyword, Separator, TokenNode},
    types::{error, tbool::PyBool},
    var::PyValue,
};

/// An `while` statement whose body may not yet have been parsed.
///
/// When first constructed only the condition is known. The body [`Block`] is supplied later
/// via [`SetBlock::set_block`] once the indented lines have been collected.
///
/// [`SetBlock::set_block`]: crate::eval::SetBlock::set_block
#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    /// `None` until the body block has been parsed and attached.
    pub body: Option<Block>,
}

impl WhileStmt {
    pub fn new(condition: Expr) -> Self {
        Self {
            condition,
            body: None,
        }
    }
}

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
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        eval_while(interpreter, *self).map(|_| None)
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
/// Returns `None` if the token sequence does not match an `while` header. The returned
/// [`WhileStmt`] has no body yet; the body is attached later via [`SetBlock::set_block`].
fn parse_while(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<WhileStmt>> {
    // idx + 1 is the index of the next token after 'if'
    if idx + 1 >= tokens.len() {
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
fn eval_while(
    interpreter: Arc<Interpreter>,
    while_stmt: WhileStmt,
) -> Result<(), Arc<dyn PyValue>> {
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
                ));
            }
            Some(b) => b.get_value(),
        };

        if !cond {
            break;
        }

        basic::eval_block(
            interpreter.clone(),
            while_stmt.body.clone().expect("WhileStmt body is None"),
        )?;
    }

    Ok(())
}
