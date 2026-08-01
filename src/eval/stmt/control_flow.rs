use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{
        Parse, ParseResult,
        stmt::ast::{BreakStmt, ContinueStmt},
    },
    lexer::tokenize::{Keyword, TokenNode},
};

impl Parse for ContinueStmt {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<ContinueStmt>> {
        parse_continue(interpreter, tokens, idx)
    }
}

/// Attempts to parse `continue` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match a `continue`.
fn parse_continue(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<ContinueStmt>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(continue_kw) = Keyword::parse(interpreter.clone(), tokens, idx)
        && continue_kw.value == Keyword::Continue
    {
        return Some(ParseResult::new(continue_kw.idx, ContinueStmt));
    }

    None
}

impl Parse for BreakStmt {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        parse_break(interpreter, tokens, idx)
    }
}

/// Attempts to parse `break` from `tokens` starting at `idx`.
///
/// Returns `None` if the token sequence does not match a `break`.
fn parse_break(
    interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<BreakStmt>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Some(break_kw) = Keyword::parse(interpreter.clone(), tokens, idx)
        && break_kw.value == Keyword::Break
    {
        return Some(ParseResult::new(break_kw.idx, BreakStmt));
    }

    None
}
