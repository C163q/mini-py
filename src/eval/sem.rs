use crate::lexer::indent::{IndentHistory, LineIndent, OwnedLineIndent};

/// Runtime state passed between successive statements during evaluation.
///
/// Tracks whether the most recent `if` condition was true or false, so that a following
/// `else` branch can decide whether to execute, and whether the statement currently being
/// evaluated is (directly or through nested blocks) inside a loop body, so that a bare
/// `break`/`continue` can be rejected as a [`SyntaxError`] when it is not.
///
/// [`SyntaxError`]: crate::types::error::get_syntax_error
#[derive(Debug, Clone)]
pub struct SemState {
    /// `None` — the previous statement was not an `if`.
    /// `Some(true)` — the previous `if` condition was true (body was executed).
    /// `Some(false)` — the previous `if` condition was false (body was skipped).
    pub last_if_result: Option<bool>,
    /// `true` while evaluating a statement nested (at any depth) inside a loop's body.
    pub in_loop: bool,
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
            in_loop: false,
        }
    }

    /// Clears the per-statement state (`last_if_result`) while leaving `in_loop` untouched.
    ///
    /// `in_loop` is deliberately not reset here: it is scoped to a loop body, not a single
    /// statement, and is instead saved/restored around [`WhileStmt`]'s `eval_with_state`.
    ///
    /// [`WhileStmt`]: crate::eval::stmt::ast::WhileStmt
    pub fn reset(&mut self) {
        self.last_if_result = None;
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
