use std::{
    collections::{HashMap, HashSet},
    sync::LazyLock,
};

use crate::{
    error::InterpreterError,
    eval::ast::Block,
    lexer::{
        indent::{CmpIndent, LineIndent},
        line::Line,
    },
};

/// Set of all Python keywords, used during tokenization to distinguish identifiers from keywords.
pub static KEYWORDS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        "False", "None", "True", "and", "as", "assert", "async", "await", "break", "class",
        "continue", "def", "del", "elif", "else", "except", "finally", "for", "from", "global",
        "if", "import", "in", "is", "lambda", "nonlocal", "not", "or", "pass", "raise", "return",
        "try", "while", "with", "yield",
    ])
});

/// Maps operator strings to their [`Operator`] variants, used for tokenizing operator tokens.
pub static STR_OPERATOR_MAPPER: LazyLock<HashMap<&'static str, Operator>> = LazyLock::new(|| {
    HashMap::from([
        ("+=", Operator::AddAssign),
        ("-=", Operator::SubAssign),
        ("*=", Operator::MulAssign),
        ("**=", Operator::PowAssign),
        ("/=", Operator::TrueDivAssign),
        ("//=", Operator::FloorDivAssign),
        ("%=", Operator::ModAssign),
        ("&=", Operator::AndAssign),
        ("|=", Operator::OrAssign),
        ("^=", Operator::XorAssign),
        ("<<=", Operator::LShiftAssign),
        (">>=", Operator::RShiftAssign),
        ("@=", Operator::MatMulAssign),
        (":=", Operator::ColonAssign),
        ("&", Operator::And),
        ("|", Operator::Or),
        ("^", Operator::Xor),
        ("~", Operator::Not),
        ("<<", Operator::LShift),
        (">>", Operator::RShift),
        ("<=", Operator::LessEqual),
        (">=", Operator::GreaterEqual),
        ("<", Operator::Less),
        (">", Operator::Greater),
        ("==", Operator::Equal),
        ("!=", Operator::NotEqual),
        ("=", Operator::Assign),
        ("+", Operator::Add),
        ("-", Operator::Sub),
        ("**", Operator::Pow),
        ("*", Operator::Mul),
        ("/", Operator::TrueDiv),
        ("//", Operator::FloorDiv),
        ("%", Operator::Mod),
        (".", Operator::Dot),
        ("@", Operator::MatMul),
    ])
});

/// Maps keyword strings to their [`Keyword`] variants.
pub static STR_KEYWORD_MAPPER: LazyLock<HashMap<&'static str, Keyword>> = LazyLock::new(|| {
    HashMap::from([
        ("False", Keyword::False),
        ("None", Keyword::None),
        ("True", Keyword::True),
        ("and", Keyword::And),
        ("as", Keyword::As),
        ("assert", Keyword::Assert),
        ("async", Keyword::Async),
        ("await", Keyword::Await),
        ("break", Keyword::Break),
        ("class", Keyword::Class),
        ("continue", Keyword::Continue),
        ("def", Keyword::Def),
        ("del", Keyword::Del),
        ("elif", Keyword::Elif),
        ("else", Keyword::Else),
        ("except", Keyword::Except),
        ("finally", Keyword::Finally),
        ("for", Keyword::For),
        ("from", Keyword::From),
        ("global", Keyword::Global),
        ("if", Keyword::If),
        ("import", Keyword::Import),
        ("in", Keyword::In),
        ("is", Keyword::Is),
        ("lambda", Keyword::Lambda),
        ("nonlocal", Keyword::Nonlocal),
        ("not", Keyword::Not),
        ("or", Keyword::Or),
        ("pass", Keyword::Pass),
        ("raise", Keyword::Raise),
        ("return", Keyword::Return),
        ("try", Keyword::Try),
        ("while", Keyword::While),
        ("with", Keyword::With),
        ("yield", Keyword::Yield),
    ])
});

/// Maps separator strings to their [`Separator`] variants.
pub static STR_SEPARATOR_MAPPER: LazyLock<HashMap<&'static str, Separator>> = LazyLock::new(|| {
    HashMap::from([
        ("(", Separator::LeftParen),
        (")", Separator::RightParen),
        ("[", Separator::LeftBracket),
        ("]", Separator::RightBracket),
        ("{", Separator::LeftBrace),
        ("}", Separator::RightBrace),
        (",", Separator::Comma),
        (":", Separator::Colon),
        (";", Separator::Semicolon),
        ("!", Separator::Exclamation),
        ("->", Separator::Arrow),
    ])
});

/// All Python keywords recognized by the lexer.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    False,
    None,
    True,
    And,
    As,
    Assert,
    Async,
    Await,
    Break,
    Class,
    Continue,
    Def,
    Del,
    Elif,
    Else,
    Except,
    Finally,
    For,
    From,
    Global,
    If,
    Import,
    In,
    Is,
    Lambda,
    Nonlocal,
    Not,
    Or,
    Pass,
    Raise,
    Return,
    Try,
    While,
    With,
    Yield,
}

/// All operators recognized by the lexer.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    AddAssign,
    SubAssign,
    MulAssign,
    PowAssign,
    TrueDivAssign,
    FloorDivAssign,
    ModAssign,
    AndAssign,
    OrAssign,
    XorAssign,
    LShiftAssign,
    RShiftAssign,
    MatMulAssign,
    ColonAssign,
    And,
    Or,
    Xor,
    Not,
    LShift,
    RShift,
    LessEqual,
    GreaterEqual,
    Less,
    Greater,
    Equal,
    NotEqual,
    Assign,
    Add,
    Sub,
    Pow,
    Mul,
    TrueDiv,
    FloorDiv,
    Mod,
    Dot,
    MatMul,
}

/// Punctuation and delimiters that separate syntactic elements.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Separator {
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Colon,        // :
    Semicolon,    // ;
    Exclamation,  // !
    Arrow,        // ->
}

/// A lexed token together with reserved space for future source location info.
///
/// The actual token value is in [`TokenNode::value`]. Once source location tracking is
/// implemented, `info` will carry line and column data.
#[derive(Debug, Clone)]
pub struct TokenNode {
    pub info: (), // TODO: source location (line, column)
    pub value: Token,
}

impl TokenNode {
    pub fn new(value: Token) -> Self {
        Self { info: (), value }
    }
}

/// A lexed token, carrying its variant and any associated payload (e.g. identifier name, number
/// literal). Wrapped in [`TokenNode`] when source location info is needed.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
    None,
    Identifier(String),
    Keyword(Keyword),
    Operator(Operator),
    Separator(Separator),
    Number(String),
    String(String),
    Block(Block),
    Unknown(String),
}

impl Token {
    pub fn new_none() -> Self {
        Self::None
    }

    pub fn new_identifier(name: String) -> Self {
        Self::Identifier(name)
    }

    pub fn new_keyword(keyword: Keyword) -> Self {
        Self::Keyword(keyword)
    }

    pub fn new_keyword_from_str(keyword_str: &str) -> Option<Self> {
        STR_KEYWORD_MAPPER
            .get(keyword_str)
            .map(|kw| Self::new_keyword(*kw))
    }

    pub fn new_operator(op: Operator) -> Self {
        Self::Operator(op)
    }

    pub fn new_operator_from_str(op_str: &str) -> Option<Self> {
        STR_OPERATOR_MAPPER
            .get(op_str)
            .map(|op| Self::new_operator(*op))
    }

    pub fn new_separator(sep: Separator) -> Self {
        Self::Separator(sep)
    }

    pub fn new_separator_from_str(sep_str: &str) -> Option<Self> {
        STR_SEPARATOR_MAPPER
            .get(sep_str)
            .map(|sep| Self::new_separator(*sep))
    }

    pub fn new_number(name: String) -> Self {
        Self::Number(name)
    }

    pub fn new_string(name: String) -> Self {
        Self::String(name)
    }

    pub fn new_block(block: Block) -> Self {
        Self::Block(block)
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

fn tokenize_number(idx: usize, chars: &[char]) -> (usize, String) {
    let mut number = String::new();
    number.push(chars[idx]);
    let next_idx = tokenize_number_following(idx + 1, chars, &mut number);
    (next_idx, number)
}

fn get_escape_char(ch: char) -> Option<char> {
    match ch {
        'n' => Some('\n'),
        't' => Some('\t'),
        'r' => Some('\r'),
        '\\' => Some('\\'),
        '\'' => Some('\''),
        '"' => Some('"'),
        _ => None,
    }
}

fn tokenize_string_literal(
    mut idx: usize,
    chars: &[char],
    string: &mut String,
) -> Result<usize, InterpreterError> {
    debug_assert!(string.is_empty());

    let quote_char = chars[idx];
    idx += 1;
    loop {
        if idx >= chars.len() {
            return Err(InterpreterError::new_lexical_error(String::from(
                "Unexpected end of string literal",
            )));
        }

        if chars[idx] == '\\' {
            idx += 1;
            if idx >= chars.len() {
                return Err(InterpreterError::new_lexical_error(String::from(
                    "Unexpected end of string literal",
                )));
            }

            match get_escape_char(chars[idx]) {
                Some(esc) => string.push(esc),
                None => {
                    return Err(InterpreterError::new_lexical_error(format!(
                        "Invalid escape character '\\{}'",
                        chars[idx]
                    )));
                }
            }
            idx += 1;
        } else if chars[idx] == quote_char {
            idx += 1;
            break;
        } else {
            string.push(chars[idx]);
            idx += 1;
        }
    }

    Ok(idx)
}

/// Tokenizes a [`Line`] into a [`Vec<Token>`], given the current scope's `indent_base`.
///
/// ## Errors
///
/// - `Err(InterpreterError::UnfinishedBlock)` — the line is indented *beyond* `indent_base`,
///   meaning it belongs to a nested block that has not been closed yet.
/// - `Err(InterpreterError::FinishedBlock)` — the line is indented *less* than `indent_base`,
///   meaning the current block has ended; the caller must handle the dedent.
/// - Other `Err` variants on lexical errors (invalid characters, unterminated strings, etc.).
pub fn tokenize(line: Line, indent_base: LineIndent) -> Result<Vec<TokenNode>, InterpreterError> {
    let line_indent = line.indent.as_slice();
    match line_indent.cmp_level(&indent_base)? {
        CmpIndent::Less(_) => {
            // This is a dedent, so we should not tokenize this line. The caller should handle the
            // dedent.
            return Err(InterpreterError::new_finished_block(Line::new(
                line.indent,
                line.content,
            )));
        }
        CmpIndent::Greater(_) => {
            // This is an indent, so we should not tokenize this line. The caller should handle the
            // indent.
            return Err(InterpreterError::new_unfinished_block(Line::new(
                line.indent,
                line.content,
            )));
        }
        CmpIndent::Equal => {
            // This is a line with the same indent level, so we can tokenize it.
        }
    }

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
                tokens.push(TokenNode::new(Token::new_keyword_from_str(&ident).unwrap()));
                continue;
            }
            tokens.push(TokenNode::new(Token::new_identifier(ident)));
            continue;
        }

        // Number
        if chars[idx].is_ascii_digit() {
            let (i, mut number) = tokenize_number(idx, &chars);
            idx = i;

            // Float
            if idx < chars.len() && chars[idx] == '.' {
                number.push(chars[idx]);
                idx += 1;
                if idx < chars.len() && chars[idx].is_ascii_digit() {
                    // e.g. 1.2
                    let (i, after_dot) = tokenize_number(idx, &chars);
                    idx = i;
                    number.push_str(&after_dot);
                } else if idx < chars.len() && (chars[idx].is_alphabetic() || chars[idx] == '_') {
                    // 1.a INVALID
                    // 1.() OK, but we will tokenize it as 1. and () separately
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Invalid float literal: expected digit after '.'",
                    )));
                } else {
                    // Allow float literals like '1.'
                    number.push('0');
                }
            }

            tokens.push(TokenNode::new(Token::new_number(number)));
            continue;
        }

        // String literal
        if chars[idx] == '\'' || chars[idx] == '"' {
            let mut string = String::new();
            idx = tokenize_string_literal(idx, &chars, &mut string)?;
            tokens.push(TokenNode::new(Token::new_string(string)));
            continue;
        }

        // Operator
        match chars[idx] {
            // <op>
            '~' => {
                tokens.push(TokenNode::new(
                    Token::new_operator_from_str(&chars[idx].to_string()).unwrap(),
                ));
                idx += 1;
                continue;
            }
            // . => Dot, .1 => Float
            '.' => {
                idx += 1;

                if idx < chars.len() && chars[idx].is_ascii_digit() {
                    // Number starting with dot, e.g. .1
                    let (i, number) = tokenize_number(idx, &chars);
                    idx = i;
                    tokens.push(TokenNode::new(Token::new_number(format!("0.{}", number))));
                } else {
                    // Dot operator
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&chars[idx - 1].to_string()).unwrap(),
                    ));
                }

                continue;
            }
            // <op> | <op>=
            '+' | '%' | '&' | '|' | '^' | '=' | '@' => {
                let first = chars[idx];
                idx += 1;
                if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&format!("{}=", first)).unwrap(),
                    ));
                    idx += 1;
                } else {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&first.to_string()).unwrap(),
                    ));
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
                        tokens.push(TokenNode::new(
                            Token::new_operator_from_str(&format!("{}{}=", first, first)).unwrap(),
                        ));
                        idx += 1;
                    } else {
                        tokens.push(TokenNode::new(
                            Token::new_operator_from_str(&format!("{}{}", first, first)).unwrap(),
                        ));
                    }
                } else if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&format!("{}=", first)).unwrap(),
                    ));
                    idx += 1;
                } else {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&first.to_string()).unwrap(),
                    ));
                }
                continue;
            }
            // - and -= but not ->
            '-' => {
                let first = chars[idx];
                idx += 1;
                if idx < chars.len() && chars[idx] == '=' {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&format!("{}=", first)).unwrap(),
                    ));
                    idx += 1;
                    continue;
                } else if idx < chars.len() && chars[idx] == '>' {
                    idx -= 1;
                } else {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&first.to_string()).unwrap(),
                    ));
                    continue;
                }
            }
            // := and != but not : !
            '!' | ':' => {
                let first = chars[idx];
                if idx + 1 < chars.len() && chars[idx + 1] == '=' {
                    tokens.push(TokenNode::new(
                        Token::new_operator_from_str(&format!("{}=", first)).unwrap(),
                    ));
                    idx += 2;
                    continue;
                }
            }
            _ => {}
        }

        // Separator
        match chars[idx] {
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | ':' | ';' | '!' => {
                tokens.push(TokenNode::new(
                    Token::new_separator_from_str(&chars[idx].to_string()).unwrap(),
                ));
                idx += 1;
                continue;
            }
            '-' if idx + 1 < chars.len() && chars[idx + 1] == '>' => {
                tokens.push(TokenNode::new(Token::new_separator_from_str("->").unwrap()));
                idx += 2;
                continue;
            }
            _ => {}
        }

        // error
        // TODO: better error message with line and column info
        return Err(InterpreterError::new_lexical_error(format!(
            "Unexpected character '{}'",
            chars[idx]
        )));
    }

    Ok(tokens)
}

mod tests {
    #[allow(unused_imports)]
    use super::*;
    #[allow(unused_imports)]
    use crate::lexer::indent::OwnedLineIndent;

    #[test]
    fn test_operator_tokenize() {
        let line = Line::new(
            OwnedLineIndent::new(),
            String::from(
                r"
+= -= *= **= /= //= %= &= |= ^= <<= >>= @= := & |
^ ~ << >> <= >= < > == != = + - ** * // / % . @
                ",
            ),
        );

        let expected = vec![
            Token::new_operator(Operator::AddAssign),
            Token::new_operator(Operator::SubAssign),
            Token::new_operator(Operator::MulAssign),
            Token::new_operator(Operator::PowAssign),
            Token::new_operator(Operator::TrueDivAssign),
            Token::new_operator(Operator::FloorDivAssign),
            Token::new_operator(Operator::ModAssign),
            Token::new_operator(Operator::AndAssign),
            Token::new_operator(Operator::OrAssign),
            Token::new_operator(Operator::XorAssign),
            Token::new_operator(Operator::LShiftAssign),
            Token::new_operator(Operator::RShiftAssign),
            Token::new_operator(Operator::MatMulAssign),
            Token::new_operator(Operator::ColonAssign),
            Token::new_operator(Operator::And),
            Token::new_operator(Operator::Or),
            Token::new_operator(Operator::Xor),
            Token::new_operator(Operator::Not),
            Token::new_operator(Operator::LShift),
            Token::new_operator(Operator::RShift),
            Token::new_operator(Operator::LessEqual),
            Token::new_operator(Operator::GreaterEqual),
            Token::new_operator(Operator::Less),
            Token::new_operator(Operator::Greater),
            Token::new_operator(Operator::Equal),
            Token::new_operator(Operator::NotEqual),
            Token::new_operator(Operator::Assign),
            Token::new_operator(Operator::Add),
            Token::new_operator(Operator::Sub),
            Token::new_operator(Operator::Pow),
            Token::new_operator(Operator::Mul),
            Token::new_operator(Operator::FloorDiv),
            Token::new_operator(Operator::TrueDiv),
            Token::new_operator(Operator::Mod),
            Token::new_operator(Operator::Dot),
            Token::new_operator(Operator::MatMul),
        ];

        let result = tokenize(line, LineIndent::new()).unwrap();
        for (res, exp) in result.iter().zip(expected.iter()) {
            assert_eq!(res.value.discriminant(), exp.discriminant());
            match (&res.value, exp) {
                (Token::Operator(res_op), Token::Operator(exp_op)) => {
                    assert_eq!(res_op, exp_op);
                }
                _ => unreachable!(),
            }
        }
    }
}
