use std::{collections::HashSet, sync::LazyLock};

use crate::{error::InterpreterError, lexer::line::Line};

pub static KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "and", "as", "assert", "async", "await", "break", "class", "continue", "def", "del",
        "elif", "else", "except", "finally", "for", "from", "global", "if", "import", "in", "is",
        "lambda", "nonlocal", "not", "or", "pass", "raise", "return", "try", "while", "with",
        "yield",
    ])
});

pub struct Token {
    pub info: (), // TODO
    pub value: TokenKind,
}

impl Token {
    pub fn new(value: TokenKind) -> Self {
        Self { info: (), value }
    }
}

#[repr(u8)]
pub enum TokenKind {
    None,
    Identifier(String),
    Keyword(String),
    Operator(String),
    Separator(String),
    Number(String),
    Unknown(String),
}

impl TokenKind {
    pub fn new_none() -> Self {
        Self::None
    }

    pub fn new_identifier(name: String) -> Self {
        Self::Identifier(name)
    }

    pub fn new_keyword(name: String) -> Self {
        Self::Keyword(name)
    }

    pub fn new_operator(name: String) -> Self {
        Self::Operator(name)
    }

    pub fn new_separator(name: String) -> Self {
        Self::Separator(name)
    }

    pub fn new_number(name: String) -> Self {
        Self::Number(name)
    }

    pub fn new_unknown(name: String) -> Self {
        Self::Unknown(name)
    }

    #[allow(dead_code)]
    fn discriminant(&self) -> u8 {
        // SAFETY: Because `Self` is marked `repr(u8)`, its layout is a `repr(C)` `union`
        // between `repr(C)` structs, each of which has the `u8` discriminant as its first
        // field, so we can read the discriminant without offsetting the pointer.
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

fn tokenize_identifier_following(mut idx: usize, chars: &[char], ident: &mut String) -> usize {
    debug_assert!(ident.chars().collect::<Vec<_>>().len() == 1);

    while idx < chars.len() {
        if chars[idx].is_alphanumeric() || chars[idx] == '_' {
            ident.push(chars[idx]);
            idx += 1;
        } else {
            break;
        }
    }

    idx
}

fn tokenize_number_following(mut idx: usize, chars: &[char], number: &mut String) -> usize {
    debug_assert!(number.chars().collect::<Vec<_>>().len() == 1);

    while idx < chars.len() {
        if chars[idx].is_ascii_digit() {
            number.push(chars[idx]);
            idx += 1;
        } else {
            break;
        }
    }

    idx
}

pub fn tokenize(line: Line) -> Result<Vec<Token>, InterpreterError> {
    let chars: Vec<_> = line.content.chars().collect();
    let mut idx = 0;
    let mut tokens = Vec::new();
    while idx < chars.len() {
        // Comment
        if chars[idx] == '#' {
            while (chars[idx] != '\n') && (idx < chars.len()) {
                idx += 1;
            }
            continue;
        }

        // Skip whitespace
        if chars[idx].is_whitespace() {
            idx += 1;
            continue;
        }

        // Identifier/Keyword
        if chars[idx].is_alphabetic() || chars[idx] == '_' {
            let mut ident = String::new();
            ident.push(chars[idx]);
            idx += 1;
            idx = tokenize_identifier_following(idx, &chars, &mut ident);
            if KEYWORDS.contains(ident.as_str()) {
                tokens.push(Token::new(TokenKind::new_keyword(ident)));
                continue;
            }
            tokens.push(Token::new(TokenKind::new_identifier(ident)));
            continue;
        }

        // Number
        if chars[idx].is_ascii_digit() {
            let mut number = String::new();
            number.push(chars[idx]);
            idx += 1;
            idx = tokenize_number_following(idx, &chars, &mut number);
            tokens.push(Token::new(TokenKind::new_number(number)));
            continue;
        }

        // Operator
        match chars[idx] {
            // <op>
            '.' | '~' => {
                tokens.push(Token::new(TokenKind::new_operator(chars[idx].to_string())));
                idx += 1;
                continue;
            }
            // <op> | <op>=
            '+' | '%' | '&' | '|' | '^' | '!' | '=' | '@' => {
                let first = chars[idx];
                idx += 1;
                if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(Token::new(TokenKind::new_operator(format!("{}=", first))));
                    idx += 1;
                } else {
                    tokens.push(Token::new(TokenKind::new_operator(first.to_string())));
                }
                continue;
            }
            // <op> | <op>= | <op><op> | <op><op>=
            '*' | '/' | '<' | '>' => {
                let first = chars[idx];
                idx += 1;
                if idx < chars.len() && chars[idx] == first {
                    idx += 1;
                    if idx < chars.len() && chars[idx] == '=' {
                        tokens.push(Token::new(TokenKind::new_operator(format!(
                            "{}{}=",
                            first, first
                        ))));
                        idx += 1;
                    } else {
                        tokens.push(Token::new(TokenKind::new_operator(format!(
                            "{}{}",
                            first, first
                        ))));
                    }
                } else if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(Token::new(TokenKind::new_operator(format!("{}=", first))));
                    idx += 1;
                } else {
                    tokens.push(Token::new(TokenKind::new_operator(first.to_string())));
                }
                continue;
            }
            // - and -= but not ->
            '-' => {
                let first = chars[idx];
                idx += 1;
                if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(Token::new(TokenKind::new_operator(format!("{}=", first))));
                    idx += 1;
                    continue;
                } else if idx < chars.len() && chars[idx] == '>' {
                    idx -= 1;
                } else {
                    tokens.push(Token::new(TokenKind::new_operator(first.to_string())));
                    continue;
                }
            }
            // := but not :
            ':' => {
                if idx + 1 < chars.len() && chars[idx + 1] == '=' {
                    tokens.push(Token::new(TokenKind::new_operator(":=".to_string())));
                    idx += 2;
                    continue;
                }
            }
            _ => {}
        }

        // Separator
        match chars[idx] {
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | ';' => {
                tokens.push(Token::new(TokenKind::new_separator(chars[idx].to_string())));
                idx += 1;
                continue;
            }
            '-' => {
                if idx + 1 < chars.len() && chars[idx + 1] == '>' {
                    tokens.push(Token::new(TokenKind::new_separator("->".to_string())));
                    idx += 2;
                    continue;
                }
            }
            _ => {}
        }

        // error
        // TODO: better error message with line and column info
        return Err(InterpreterError::new(format!(
            "Unexpected character '{}'",
            chars[idx]
        )));
    }

    Ok(tokens)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_operator_tokenize() {
        let line = Line::new(
            Vec::new(),
            String::from(
                r"
                    += -= *= **= /= //= %= &= |= ^= <<= >>= @= := & |
                    ^ ~ << >> <= >= < > == != ! = + - ** * // / % . @
                ",
            ),
        );

        let expected = vec![
            TokenKind::new_operator("+=".to_string()),
            TokenKind::new_operator("-=".to_string()),
            TokenKind::new_operator("*=".to_string()),
            TokenKind::new_operator("**=".to_string()),
            TokenKind::new_operator("/=".to_string()),
            TokenKind::new_operator("//=".to_string()),
            TokenKind::new_operator("%=".to_string()),
            TokenKind::new_operator("&=".to_string()),
            TokenKind::new_operator("|=".to_string()),
            TokenKind::new_operator("^=".to_string()),
            TokenKind::new_operator("<<=".to_string()),
            TokenKind::new_operator(">>=".to_string()),
            TokenKind::new_operator("@=".to_string()),
            TokenKind::new_operator(":=".to_string()),
            TokenKind::new_operator("&".to_string()),
            TokenKind::new_operator("|".to_string()),
            TokenKind::new_operator("^".to_string()),
            TokenKind::new_operator("~".to_string()),
            TokenKind::new_operator("<<".to_string()),
            TokenKind::new_operator(">>".to_string()),
            TokenKind::new_operator("<=".to_string()),
            TokenKind::new_operator(">=".to_string()),
            TokenKind::new_operator("<".to_string()),
            TokenKind::new_operator(">".to_string()),
            TokenKind::new_operator("==".to_string()),
            TokenKind::new_operator("!=".to_string()),
            TokenKind::new_operator("!".to_string()),
            TokenKind::new_operator("=".to_string()),
            TokenKind::new_operator("+".to_string()),
            TokenKind::new_operator("-".to_string()),
            TokenKind::new_operator("**".to_string()),
            TokenKind::new_operator("*".to_string()),
            TokenKind::new_operator("//".to_string()),
            TokenKind::new_operator("/".to_string()),
            TokenKind::new_operator("%".to_string()),
            TokenKind::new_operator(".".to_string()),
            TokenKind::new_operator("@".to_string()),
        ];

        let result = tokenize(line).unwrap();
        for (res, exp) in result.iter().zip(expected.iter()) {
            assert_eq!(res.value.discriminant(), exp.discriminant());
            match (&res.value, exp) {
                (TokenKind::Operator(res_op), TokenKind::Operator(exp_op)) => {
                    assert_eq!(res_op, exp_op);
                }
                _ => unreachable!(),
            }
        }
    }
}
