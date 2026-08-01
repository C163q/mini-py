use std::sync::Arc;

use crate::{
    Interpreter,
    error::PyError,
    eval::{
        expr::ast::Expr,
        stmt::{
            assign::Assign,
            ast::{BreakStmt, ContinueStmt, ElifStmt, ElseStmt, IfStmt, WhileStmt},
        },
    },
    lexer::{
        self,
        indent::LineIndent,
        tokenize::{Token, TokenNode},
    },
    types::{error, tstr::PyStr},
    var::PyValue,
};

pub mod basic;
pub mod expr;
pub mod output;
pub mod sem;
pub mod stmt;

/// Evaluates an AST node and optionally returns a value.
///
/// Returns `Ok(Some(value))` for expressions, `Ok(None)` for statements with no result,
/// or `Err(PyError::Exception(_))` on runtime errors. A `break`/`continue` also travels
/// through this same `Err` path as `Err(PyError::ControlFlow(_))`; implementors that are not
/// a loop (e.g. `if`) should just propagate it via `?` rather than handling it — only the
/// loop that owns the body (e.g. [`WhileStmt`]) is expected to catch it.
///
/// [`WhileStmt`]: stmt::ast::WhileStmt
pub trait Eval: Send + Sync {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, PyError>;

    /// Evaluates the node and resets [`SemState`] afterwards, unless the node itself wrote new
    /// state (e.g. an `if` statement records its condition result for a potential `else` branch).
    ///
    /// Prefer this over calling [`eval`] directly so that stale state from a previous statement
    /// does not leak into the next one.
    ///
    /// [`SemState`]: sem::SemState
    /// [`eval`]: Eval::eval
    fn eval_with_state(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, PyError> {
        let result = self.eval(interpreter.clone());
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .get_sem_state_mut()
            .reset();
        result
    }
}

/// Supplies the deferred [`Block`] body to a compound statement (e.g. `if`).
///
/// Compound statements are parsed before their body is available — the body arrives only after
/// the interpreter processes the subsequent indented lines. Implementors store the block and
/// use it when [`Eval::eval`] is called.
///
/// [`Block`]: basic::ast::Block
pub trait SetBlock: Eval + Send + Sync {
    /// Attaches `block` as the body of this statement.
    fn set_block(&mut self, block: basic::ast::Block);
}

/// Attempts to parse `Self` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence at `idx` does not match `Self`'s grammar, letting the
/// caller fall through and try a different rule. On success, returns a [`ParseResult`] whose
/// `idx` points to the first token **not** consumed by the match.
pub trait Parse: Sized {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>>;
}

/// The result of a successful parse step.
///
/// `idx` points to the first token that was **not** consumed, so the caller can continue
/// parsing from there.
#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    pub idx: usize,
    pub value: T,
}

impl<T> ParseResult<T> {
    pub fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

/// Evaluates a raw `&str` line, using an empty indentation base (top-level scope).
///
/// Returns `Ok(Some(output))` if the line produces a displayable value (used by the REPL),
/// `Ok(None)` for statements with no output, or `Err` on errors.
pub fn eval_line(
    interpreter: Arc<Interpreter>,
    line: &str,
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    eval_line_with_indent(interpreter, line, LineIndent::new(), lexer::lex_raw_line).map_err(|e| {
        match e {
            PyError::Exception(err) => err,
            PyError::ControlFlow(cf) => {
                panic!("Control flows should not be handled here: {:?}", cf)
            }
        }
    })
}

/// Finalizes and evaluates any block still buffered in the [`LexContext`], for use once the
/// input stream has ended.
///
/// A block is normally closed off (and evaluated) when a later line arrives with a shallower
/// indent than the block. If the input ends first, no such line ever arrives, so the buffered
/// lines would otherwise be silently dropped. Call this once after the last line has been fed
/// to force the trailing block to build and evaluate.
///
/// Returns `Ok(None)` if there was no buffered block to finish.
///
/// This matters even for a nested block that is itself part of an outer block, not just for
/// the outermost one. In the example below, the outer `if 1:` treats `if 2:` and `2` together
/// as a single [`Block`], which is evaluated line by line once the outer block closes. That
/// evaluates `if 2:`, which in turn expects its own block and starts buffering `2` as that
/// block's body — but since no further line ever dedents past it, this inner block never
/// closes on its own. Without calling `eval_line_finished`, that buffered `2` is silently
/// dropped while the interpreter is left still expecting a block for the inner `if`,
/// which then surfaces as a misleading "unexpected indent" [`SyntaxError`] on whatever line
/// is fed in next (here, `1`) rather than the real issue.
///
/// ```mini-py
/// if 1:
///   if 2:
///     2 # The block should finish here, but since there are no more lines, the parsed lines are
///       # cached in the interpreter and the block is never built.
///       # In this case, eval_line_finished should be called to finish the block and evaluate it.
/// 1
/// ```
///
/// [`LexContext`]: crate::lexer::LexContext
/// [`Block`]: basic::ast::Block
/// [`SyntaxError`]: crate::types::error::get_syntax_error
pub fn eval_line_finished(
    interpreter: Arc<Interpreter>,
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    eval_line_finished_block(interpreter).map_err(|e| match e {
        PyError::Exception(err) => err,
        PyError::ControlFlow(cf) => panic!("Control flows should not be handled here: {:?}", cf),
    })
}

/// [`eval_line_finished`] is called by interpreter and [`eval_line_finished_block`] is called
/// by internal implementions that requires to handle control flows.
fn eval_line_finished_block(interpreter: Arc<Interpreter>) -> Result<Option<PyStr>, PyError> {
    if let Ok(block) = interpreter
        .get_lex_context()
        .get_and_clear_block_context()
        .build_block()
    {
        return eval_line_from_token(interpreter.clone(), &[TokenNode::new(Token::Block(block))]);
    }

    // TODO:
    // interpreter.sem_context.lock().unwrap().indent.expected_indent.is_some() -> SyntaxError:
    // expected indent.
    if interpreter
        .sem_context
        .lock()
        .unwrap()
        .get_indent_history()
        .expected_indent
        .is_some()
    {
        return Err(error::get_syntax_error(
            interpreter.clone(),
            "Expected indented block, but got end of input.".to_string(),
        )
        .into());
    }

    Ok(None)
}

type LexerFn<T> = fn(Arc<Interpreter>, T, LineIndent) -> Result<lexer::LexTokens, Arc<dyn PyValue>>;

fn eval_line_with_indent<T>(
    interpreter: Arc<Interpreter>,
    line: T,
    indent: LineIndent,
    lexer: LexerFn<T>,
) -> Result<Option<PyStr>, PyError> {
    let lex_tokens = lexer(interpreter.clone(), line, indent).map_err(PyError::new_exception)?;

    if let Some(block) = lex_tokens.block {
        eval_line_from_token(
            interpreter.clone(),
            &[TokenNode::new(Token::Block(block.clone()))],
        )?;
    }

    if lex_tokens.tokens.is_empty() {
        return Ok(None);
    }

    eval_line_from_token(interpreter, &lex_tokens.tokens)
}

/// Parses and evaluates `tokens`, rejecting a `break`/`continue` that is not inside a loop.
///
/// [`parse_and_eval_line`] turns a bare `break`/`continue` into `Err(PyError::ControlFlow(_))`
/// unconditionally, since at that point it has no way to know whether it is currently inside a
/// loop body. This function is the single place that checks [`SemState::in_loop`] and converts
/// an out-of-loop `break`/`continue` into a proper [`SyntaxError`].
///
/// [`SemState::in_loop`]: sem::SemState::in_loop
/// [`SyntaxError`]: crate::types::error::get_syntax_error
fn eval_line_from_token(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
) -> Result<Option<PyStr>, PyError> {
    if tokens.is_empty() {
        return Ok(None);
    }
    parse_and_eval_line(interpreter.clone(), tokens).map_err(|e| match e {
        PyError::ControlFlow(cf)
            if !interpreter
                .sem_context
                .lock()
                .unwrap()
                .get_sem_state()
                .in_loop =>
        {
            error::get_syntax_error(interpreter, format!("'{}' not properly in loop", cf)).into()
        }
        _ => e,
    })
}

/// Parses `tokens` as a statement or expression and evaluates it.
///
/// Returns the string representation of the result for REPL display, or `None` for
/// statements. Side effects (variable assignments, etc.) are applied to the interpreter
/// regardless of the return value.
///
/// Never call this function directly. Call [`eval_line_from_token`], which will apply more checks.
fn parse_and_eval_line(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
) -> Result<Option<PyStr>, PyError> {
    let idx = 0;

    {
        // check for block
        let arc_interpreter = interpreter.clone();
        let mut sem_context = arc_interpreter.sem_context.lock().unwrap();
        let indent_history = sem_context.get_indent_history_mut();

        if let Some(padding_block) = &mut indent_history.expected_indent {
            if tokens.len() == 1
                && let Token::Block(block) = &tokens[0].value
            {
                padding_block.set_block(block.clone());

                // We can assert that `expected_indent` is Some(_), so `take()` is safe.
                let eval_block = indent_history.expected_indent.take().unwrap();

                // Release the lock before calling eval_with_state; otherwise eval_with_state will
                // deadlock when it tries to acquire sem_context for itself.
                drop(sem_context);

                match eval_block.eval_with_state(interpreter.clone())? {
                    None => {
                        return Ok(None);
                    }
                    Some(value) => {
                        let output = output::output_value(interpreter.clone(), value)
                            .map_err(PyError::new_exception)?;
                        return Ok(Some(output));
                    }
                }
                // unreachable!()
            }

            drop(sem_context);
            return Err(
                error::get_syntax_error(interpreter, "Unexpected indent.".to_string()).into(),
            );
        }
    }

    // break
    if let Some(break_stmt) = BreakStmt::parse(interpreter.clone(), tokens, idx)
        && break_stmt.idx == tokens.len()
    {
        return Err(PyError::new_break());
    }

    // continue
    if let Some(continue_stmt) = ContinueStmt::parse(interpreter.clone(), tokens, idx)
        && continue_stmt.idx == tokens.len()
    {
        return Err(PyError::new_continue());
    }

    // if <condition> :
    if let Some(if_stmt) = IfStmt::parse(interpreter.clone(), tokens, idx)
        && if_stmt.idx == tokens.len()
    {
        // Store the parsed if statement so the next (indented) line can be attached as its body.
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .get_indent_history_mut()
            .expected_indent = Some(Box::new(if_stmt.value));

        return Ok(None);
    }

    // elif <condition> :
    if let Some(elif_stmt) = ElifStmt::parse(interpreter.clone(), tokens, idx)
        && elif_stmt.idx == tokens.len()
    {
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .get_indent_history_mut()
            .expected_indent = Some(Box::new(elif_stmt.value));

        return Ok(None);
    }

    // else :
    if let Some(else_stmt) = ElseStmt::parse(interpreter.clone(), tokens, idx)
        && else_stmt.idx == tokens.len()
    {
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .get_indent_history_mut()
            .expected_indent = Some(Box::new(else_stmt.value));

        return Ok(None);
    }

    // while <condition> :
    if let Some(while_stmt) = WhileStmt::parse(interpreter.clone(), tokens, idx)
        && while_stmt.idx == tokens.len()
    {
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .get_indent_history_mut()
            .expected_indent = Some(Box::new(while_stmt.value));

        return Ok(None);
    }

    // <expr>
    if let Some(expr) = Expr::parse(interpreter.clone(), tokens, idx)
        && expr.idx == tokens.len()
    {
        // Expressions always produce a value (unwrap is safe here).
        let value = Box::new(expr.value)
            .eval_with_state(interpreter.clone())?
            .unwrap();
        let output =
            output::output_value(interpreter.clone(), value).map_err(PyError::new_exception)?;
        return Ok(Some(output));
    }

    // <lvalue> = <expr>
    if let Some(assign) = Assign::parse(interpreter.clone(), tokens, idx)
        && assign.idx == tokens.len()
    {
        Box::new(assign.value).eval_with_state(interpreter.clone())?;
        return Ok(None);
    }

    Err(error::get_syntax_error(interpreter, "Invalid syntax".to_string()).into())
}
