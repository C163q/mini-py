use std::sync::Arc;

use crate::{
    Interpreter,
    lexer::{
        self,
        indent::{IndentHistory, LineIndent, OwnedLineIndent},
        tokenize::{Token, TokenKind},
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

pub trait Eval: Send + Sync {
    fn eval(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>>;

    fn eval_with_state(
        self: Box<Self>,
        interpreter: Arc<Interpreter>,
    ) -> Result<Option<Arc<dyn PyValue>>, Arc<dyn PyValue>> {
        let result = self.eval(interpreter.clone());
        *interpreter.sem_context.lock().unwrap().get_sem_state_mut() = SemState::default();
        result
    }
}

pub trait SetBlock: Eval + Send + Sync {
    fn set_block(&mut self, block: ast::Block);
}

#[derive(Debug, Clone)]
pub struct SemState {
    /// None => last statement is not if statement
    /// Some(true) => last statement is if statement and the condition is true
    /// Some(false) => last statement is if statement and the condition is false
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

    pub fn get_indent_history(&self) -> &IndentHistory {
        &self.indent
    }

    pub fn get_indent_history_mut(&mut self) -> &mut IndentHistory {
        &mut self.indent
    }

    /// None => No expected indent, Some(LineIndent) => Expected indent
    pub fn get_last_indent_if_expect(&self) -> Option<LineIndent<'_>> {
        if self.indent.expected_indent.is_some() {
            Some(
                self.indent
                    .stack
                    .last()
                    .map_or(LineIndent::new(), |v| v.as_slice()),
            )
        } else {
            None
        }
    }

    pub fn get_last_indent(&self) -> LineIndent<'_> {
        self.indent
            .stack
            .last()
            .map_or(LineIndent::new(), |v| v.as_slice())
    }

    pub fn get_indent_stack(&self) -> &[OwnedLineIndent] {
        self.indent.stack.as_slice()
    }

    pub fn get_sem_state(&self) -> &SemState {
        &self.state
    }

    pub fn get_sem_state_mut(&mut self) -> &mut SemState {
        &mut self.state
    }
}

#[derive(Debug, Clone)]
pub struct ParseResult<T> {
    /// Next index to parse. It should be the index of the first token that is not parsed yet.
    pub idx: usize,
    pub value: T,
}

impl<T> ParseResult<T> {
    pub fn new(idx: usize, value: T) -> Self {
        Self { idx, value }
    }
}

pub fn eval_line(
    interpreter: Arc<Interpreter>,
    line: &str,
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    eval_line_with_indent(interpreter, line, LineIndent::new(), lexer::lex_raw_line)
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
        eval_line_from_token(interpreter.clone(), &[Token::new(TokenKind::Block(block))])?;
    }

    if lex_tokens.tokens.is_empty() {
        return Ok(None);
    }

    eval_line_from_token(interpreter, &lex_tokens.tokens)
}

fn eval_line_from_token(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    if tokens.is_empty() {
        return Ok(None);
    }
    parse_and_eval_line(interpreter, tokens)
}

/// Ok(PyStr) if the line is valid, Err(Arc<dyn PyValue>) otherwise.
///
/// Note that PyStr is only used for REPL. Any effect of the line should be applied to the
/// interpreter.
fn parse_and_eval_line(
    interpreter: Arc<Interpreter>,
    tokens: &[Token],
) -> Result<Option<PyStr>, Arc<dyn PyValue>> {
    let idx = 0;

    {
        // check for block
        let arc_interpreter = interpreter.clone();
        let mut sem_context = arc_interpreter.sem_context.lock().unwrap();
        let indent_history = sem_context.get_indent_history_mut();

        if let Some(padding_block) = &mut indent_history.expected_indent {
            if tokens.len() == 1
                && let TokenKind::Block(block) = &tokens[0].value
            {
                padding_block.set_block(block.clone());

                // Some(padding_block) in the whole scope of this block, so we can safely take it
                // out of the expected_indent.
                let eval_block = indent_history.expected_indent.take().unwrap();

                // Drop sem_context before eval_block.eval_with_state to avoid deadlock when
                // eval_with_state tries to lock sem_context again.
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

    // if <expr> :
    if let Some(if_stmt) = sif::parse_if(interpreter.clone(), tokens, idx)
        && if_stmt.idx == tokens.len()
    {
        // Padding the expected indent for the next line to be parsed. The next line should be
        // indented more than the current line.
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
        // eval expr always returns a value
        let value = Box::new(expr.value)
            .eval_with_state(interpreter.clone())?
            .unwrap();
        let output = output::output_value(interpreter.clone(), value)?;
        return Ok(Some(output));
    }

    // assign <lval> = <expr>
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
