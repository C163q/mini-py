use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{Parse, ParseResult, basic::ast::LValue},
    lexer::tokenize::{Token, TokenNode},
};

impl Parse for LValue {
    fn parse(
        interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<LValue>> {
        parse_lvalue(interpreter, tokens, idx)
    }
}

fn parse_lvalue(
    _interpreter: Arc<Interpreter>,
    tokens: &[TokenNode],
    idx: usize,
) -> Option<ParseResult<LValue>> {
    if idx >= tokens.len() {
        return None;
    }

    if let Token::Identifier(name) = &tokens[idx].value {
        Some(ParseResult::new(idx + 1, LValue::new(name.clone())))
    } else {
        None
    }
}
