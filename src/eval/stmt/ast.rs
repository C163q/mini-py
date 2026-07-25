use crate::eval::{basic::ast::Block, expr::ast::Expr};

/// An `else` statement whose body may not yet have been parsed.
///
/// When first constructed the body [`Block`] is unknown. The body is supplied later via
/// [`SetBlock::set_block`] once the indented lines have been collected.
///
/// [`SetBlock::set_block`]: crate::eval::SetBlock::set_block
#[derive(Debug, Clone)]
pub struct ElseStmt {
    /// `None` until the body block has been parsed and attached.
    pub body: Option<Block>,
}

impl ElseStmt {
    pub fn new() -> Self {
        Self { body: None }
    }
}

impl Default for ElseStmt {
    fn default() -> Self {
        Self::new()
    }
}

/// An `elif` statement whose body may not yet have been parsed.
///
/// When first constructed only the condition is known. The body [`Block`] is supplied later
/// via [`SetBlock::set_block`] once the indented lines have been collected.
///
/// [`SetBlock::set_block`]: crate::eval::SetBlock::set_block
#[derive(Debug, Clone)]
pub struct ElifStmt {
    pub condition: Expr,
    /// `None` until the body block has been parsed and attached.
    pub body: Option<Block>,
}

impl ElifStmt {
    pub fn new(condition: Expr) -> Self {
        Self {
            condition,
            body: None,
        }
    }
}

/// An `if` statement whose body may not yet have been parsed.
///
/// When first constructed only the condition is known. The body [`Block`] is supplied later
/// via [`SetBlock::set_block`] once the indented lines have been collected.
///
/// [`SetBlock::set_block`]: crate::eval::SetBlock::set_block
#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    /// `None` until the body block has been parsed and attached.
    pub body: Option<Block>,
}

impl IfStmt {
    pub fn new(condition: Expr) -> Self {
        Self {
            condition,
            body: None,
        }
    }
}
