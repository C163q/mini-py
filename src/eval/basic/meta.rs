use std::sync::Arc;

use crate::{
    Interpreter,
    eval::{Parse, ParseResult},
    lexer::tokenize::{Keyword, Operator, Separator, Token, TokenNode},
};

impl Parse for Keyword {
    fn parse(
        _interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        if idx >= tokens.len() {
            return None;
        }

        if let Token::Keyword(keyword) = &tokens[idx].value {
            Some(ParseResult::new(idx + 1, *keyword))
        } else {
            None
        }
    }
}

impl Parse for Operator {
    fn parse(
        _interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        if idx >= tokens.len() {
            return None;
        }

        if let Token::Operator(operator) = &tokens[idx].value {
            Some(ParseResult::new(idx + 1, *operator))
        } else {
            None
        }
    }
}

impl Parse for Separator {
    fn parse(
        _interpreter: Arc<Interpreter>,
        tokens: &[TokenNode],
        idx: usize,
    ) -> Option<ParseResult<Self>> {
        if idx >= tokens.len() {
            return None;
        }

        if let Token::Separator(separator) = &tokens[idx].value {
            Some(ParseResult::new(idx + 1, *separator))
        } else {
            None
        }
    }
}
