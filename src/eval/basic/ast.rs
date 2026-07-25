use crate::lexer::{
    indent::{LineIndent, OwnedLineIndent},
    line::Line,
};

/// An assignable target (currently only a bare identifier).
#[derive(Debug, Clone)]
pub struct LValue {
    pub name: String,
}

impl LValue {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

/// A sequence of lines that form the body of a compound statement (e.g. `if`).
///
/// All lines share the same `base_indent`. The first line's indentation must equal
/// `base_indent`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub base_indent: OwnedLineIndent,
    pub lines: Vec<Line>,
}

impl Block {
    pub fn new(base_indent: OwnedLineIndent, lines: Vec<Line>) -> Self {
        assert!(!lines.is_empty(), "Block must have at least one line");
        assert!(
            lines[0].indent == base_indent,
            "First line of block must have the same indent as the block"
        );
        Self { base_indent, lines }
    }

    pub fn get_indent(&self) -> LineIndent<'_> {
        self.base_indent.as_slice()
    }
}
