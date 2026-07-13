use std::sync::Arc;

use crate::{
    Interpreter,
    lexer::{
        self,
        indent::{IndentHistory, LineIndent, OwnedLineIndent},
        tokenize::{Token, TokenNode},
    },
    types::{error, tstr::PyStr},
    var::PyValue,
};

pub mod assign;
pub mod ast;
pub mod block;
pub mod expr;
pub mod output;
pub mod sif;

/// Evaluates an AST node and optionally returns a value.
///
/// Returns `Ok(Some(value))` for expressions, `Ok(None)` for statements with no result,
/// or `Err(exception)` on runtime errors.
pub trait Eval: Send + Sync {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>>;

    /// Evaluates the node and resets [`SemState`] afterwards, unless the node itself wrote new
    /// state (e.g. an `if` statement records its condition result for a potential `else` branch).
    ///
    /// Prefer this over calling [`eval`] directly so that stale state from a previous statement
    /// does not leak into the next one.
    ///
    /// [`eval`]: Eval::eval
    fn eval_with_state(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        let result = self.eval(interpreter.clone());
        *interpreter.sem_context.lock().unwrap().get_sem_state_mut() = SemState::default();
        result
    }
}

/// Supplies the deferred [`Block`] body to a compound statement (e.g. `if`).
///
/// Compound statements are parsed before their body is available — the body arrives only after
/// the interpreter processes the subsequent indented lines. Implementors store the block and
/// use it when [`Eval::eval`] is called.
///
/// [`Block`]: ast::Block
pub trait SetBlock: Eval + Send + Sync {
    /// Attaches `block` as the body of this statement.
    fn set_block(&mut self, block: ast::Block);
}

/// Runtime state passed between successive statements during evaluation.
///
/// Currently tracks only whether the most recent `if` condition was true or false, so that a
/// following `else` branch can decide whether to execute.
#[derive(Debug, Clone)]
pub struct SemState {
    /// `None` — the previous statement was not an `if`.
    /// `Some(true)` — the previous `if` condition was true (body was executed).
    /// `Some(false)` — the previous `if` condition was false (body was skipped).
    pub last_if_result: Option<bool>,
}

impl Default for SemState {
    fn default() -> Self {
        Self::new()
    }
}

impl SemState {
    pub fn new() -> Self {
        Self {
            last_if_result: None,
        }
    }
}

/// Persistent evaluation context shared across successive lines.
///
/// Holds the indentation history (for block tracking) and the current [`SemState`]
/// (for inter-statement control flow such as `if`/`else`).
#[derive(Debug)]
pub struct SemContext {
    indent: IndentHistory,
    state: SemState,
}

impl Default for SemContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SemContext {
    pub fn new() -> Self {
        Self {
            indent: IndentHistory::new(),
            state: SemState::new(),
        }
    }

    /// Returns a shared reference to the indentation history.
    pub fn get_indent_history(&self) -> &IndentHistory {
        &self.indent
    }

    /// Returns a mutable reference to the indentation history.
    pub fn get_indent_history_mut(&mut self) -> &mut IndentHistory {
        &mut self.indent
    }

    /// Returns the current indentation level if a deeper block is expected on the next line,
    /// or `None` if no block is pending.
    pub fn get_last_indent_if_expect(&self) -> Option<LineIndent<'_>> {
        if self.indent.expected_indent.is_some() {
            Some(self.indent.stack.current().unwrap_or_default())
        } else {
            None
        }
    }

    /// Returns the current (innermost) indentation level, defaulting to empty if the stack is empty.
    pub fn get_last_indent(&self) -> LineIndent<'_> {
        self.indent.stack.current().unwrap_or_default()
    }

    /// Returns the full indentation stack as a slice.
    pub fn get_indent_stack(&self) -> &[OwnedLineIndent] {
        &self.indent.stack
    }

    /// Returns a shared reference to the current [`SemState`].
    pub fn get_sem_state(&self) -> &SemState {
        &self.state
    }

    /// Returns a mutable reference to the current [`SemState`].
    pub fn get_sem_state_mut(&mut self) -> &mut SemState {
        &mut self.state
    }
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
    eval_line_with_indent(interpreter, line, LineIndent::new(), lexer::lex_raw_line)
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
/// [`Block`]: ast::Block
/// [`SyntaxError`]: crate::types::error::get_syntax_error
pub fn eval_line_finished(
    interpreter: Arc<Interpreter>,
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
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

    Ok(None)
}

type LexerFn<T> = fn(Arc<Interpreter>, T, LineIndent) -> Result<lexer::LexTokens, Arc<dyn PyValue>>;

fn eval_line_with_indent<T>(
    interpreter: Arc<Interpreter>,
    line: T,
    indent: LineIndent,
    lexer: LexerFn<T>,
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    let lex_tokens = lexer(interpreter.clone(), line, indent)?;

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

fn eval_line_from_token(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    if tokens.is_empty() {
        return Ok(None);
    }
    parse_and_eval_line(interpreter, tokens)
}

/// Parses `tokens` as a statement or expression and evaluates it.
///
/// Returns the string representation of the result for REPL display, or `None` for
/// statements. Side effects (variable assignments, etc.) are applied to the interpreter
/// regardless of the return value.
fn parse_and_eval_line(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
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
                        let output = output::output_value(interpreter.clone(), value)?;
                        return Ok(Some(output));
                    }
                }
                // unreachable!()
            }

            drop(sem_context);
            return Err(error::get_syntax_error(
                interpreter,
                "Unexpected indent.".to_string(),
            ));
        }
    }

    // if <condition> :
    if let Some(if_stmt) = sif::parse_if(interpreter.clone(), tokens, idx)
        && if_stmt.idx == tokens.len()
    {
        // Store the parsed if statement so the next (indented) line can be attached as its body.
        interpreter
            .sem_context
            .lock()
            .unwrap()
            .indent
            .expected_indent = Some(Box::new(if_stmt.value));

        return Ok(None);
    }

    // <expr>
    if let Some(expr) = expr::parse_expr(interpreter.clone(), tokens, idx)
        && expr.idx == tokens.len()
    {
        // Expressions always produce a value (unwrap is safe here).
        let value = Box::new(expr.value)
            .eval_with_state(interpreter.clone())?
            .unwrap();
        let output = output::output_value(interpreter.clone(), value)?;
        return Ok(Some(output));
    }

    // <lvalue> = <expr>
    if let Some(assign) = assign::parse_assign(interpreter.clone(), tokens, idx)
        && assign.idx == tokens.len()
    {
        Box::new(assign.value).eval_with_state(interpreter.clone())?;
        return Ok(None);
    }

    Err(error::get_syntax_error(
        interpreter,
        "Invalid syntax".to_string(),
    ))
}
