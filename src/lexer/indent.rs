use std::{
    fmt::Debug,
    num::NonZero,
    ops::{Deref, DerefMut},
};

use crate::{error::InterpreterError, eval::SetBlock};

/// A single indentation unit: consecutive tabs or spaces, stored as a non-zero count.
///
/// Mixed tabs and spaces within a single indentation level are tracked as separate `Indent`
/// elements in a [`LineIndent`] sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indent {
    Tab(NonZero<usize>),
    Space(NonZero<usize>),
}

/// Result of comparing two indentation levels via [`LineIndent::cmp_level`].
///
/// The payload of `Less` and `Greater` holds the **remainder** — the portion of the winning
/// side that was not consumed by the common prefix. This allows callers to know exactly how
/// much indentation was added or removed.
#[derive(Debug, Clone)]
pub enum CmpIndent {
    /// Current indent is **less** than the reference — i.e., a dedent (leaving a block).
    Less(Vec<Indent>),
    /// Current indent is exactly equal to the reference.
    Equal,
    /// Current indent is **greater** than the reference — i.e., an indent (entering a block).
    Greater(Vec<Indent>),
}

/// A borrowed, read-only view of an indentation sequence.
///
/// Prefer this over [`OwnedLineIndent`] for comparisons and lookups to avoid allocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineIndent<'a>(&'a [Indent]);

impl<'a> From<&'a [Indent]> for LineIndent<'a> {
    fn from(indents: &'a [Indent]) -> Self {
        Self(indents)
    }
}

impl<'a, const N: usize> From<&'a [Indent; N]> for LineIndent<'a> {
    fn from(indents: &'a [Indent; N]) -> Self {
        Self(indents)
    }
}

impl<'a> From<LineIndent<'a>> for &'a [Indent] {
    fn from(line_indent: LineIndent<'a>) -> Self {
        line_indent.0
    }
}

impl Default for LineIndent<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> LineIndent<'a> {
    /// Creates an empty (zero-indentation) `LineIndent`.
    pub fn new() -> Self {
        Self(&[])
    }

    /// Copies the indentation sequence into an owned `Vec`.
    pub fn to_vec(&self) -> Vec<Indent> {
        self.0.to_vec()
    }

    /// Returns `true` if `self` and `indents` represent the same total indentation amount.
    ///
    /// Two sequences are considered equal even if they differ in how units are grouped — e.g.,
    /// `[Tab(2)]` and `[Tab(1), Tab(1)]` — as long as their total amounts match.
    ///
    /// Returns an error if tabs and spaces are mixed in an incompatible way.
    pub fn is_same_level(&self, indents: &[Indent]) -> Result<bool, InterpreterError> {
        let mut checker: Vec<Indent> = self.0.iter().copied().rev().collect();
        let mut given_indents: Vec<Indent> = indents.iter().copied().rev().collect();
        while !checker.is_empty() && !given_indents.is_empty() {
            let indent = checker.pop().unwrap();
            let given_indent = given_indents.pop().unwrap();
            match (indent, given_indent) {
                (Indent::Tab(tab), Indent::Tab(given_tab)) => {
                    if tab == given_tab {
                        continue;
                    } else if given_tab > tab {
                        given_indents.push(Indent::Tab(
                            NonZero::new(given_tab.get() - tab.get()).unwrap(),
                        ));
                    } else {
                        checker.push(Indent::Tab(
                            NonZero::new(tab.get() - given_tab.get()).unwrap(),
                        ));
                    }
                }
                (Indent::Space(space), Indent::Space(given_space)) => {
                    if space == given_space {
                        continue;
                    } else if given_space > space {
                        given_indents.push(Indent::Space(
                            NonZero::new(given_space.get() - space.get()).unwrap(),
                        ));
                    } else {
                        checker.push(Indent::Space(
                            NonZero::new(space.get() - given_space.get()).unwrap(),
                        ));
                    }
                }
                _ => {
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Inconsistent indentation: mixing tabs and spaces is not allowed",
                    )));
                }
            }
        }
        Ok(checker.is_empty() && given_indents.is_empty())
    }

    /// Compares `self` against `indents` and returns a [`CmpIndent`] describing the relationship.
    ///
    /// Like [`is_same_level`], grouping differences are normalized before comparison.
    /// Returns an error if tabs and spaces are mixed in an incompatible way.
    ///
    /// [`is_same_level`]: LineIndent::is_same_level
    pub fn cmp_level(&self, indents: &[Indent]) -> Result<CmpIndent, InterpreterError> {
        let mut checker: Vec<Indent> = self.0.iter().copied().rev().collect();
        let mut given_indents: Vec<Indent> = indents.iter().copied().rev().collect();
        while !checker.is_empty() && !given_indents.is_empty() {
            let indent = checker.pop().unwrap();
            let given_indent = given_indents.pop().unwrap();
            match (indent, given_indent) {
                (Indent::Tab(tab), Indent::Tab(given_tab)) => {
                    if tab == given_tab {
                        continue;
                    } else if given_tab > tab {
                        given_indents.push(Indent::Tab(
                            NonZero::new(given_tab.get() - tab.get()).unwrap(),
                        ));
                    } else {
                        checker.push(Indent::Tab(
                            NonZero::new(tab.get() - given_tab.get()).unwrap(),
                        ));
                    }
                }
                (Indent::Space(space), Indent::Space(given_space)) => {
                    if space == given_space {
                        continue;
                    } else if given_space > space {
                        given_indents.push(Indent::Space(
                            NonZero::new(given_space.get() - space.get()).unwrap(),
                        ));
                    } else {
                        checker.push(Indent::Space(
                            NonZero::new(space.get() - given_space.get()).unwrap(),
                        ));
                    }
                }
                _ => {
                    return Err(InterpreterError::new_lexical_error(String::from(
                        "Inconsistent indentation: mixing tabs and spaces is not allowed",
                    )));
                }
            }
        }
        if given_indents.is_empty() && checker.is_empty() {
            Ok(CmpIndent::Equal)
        } else if given_indents.is_empty() {
            Ok(CmpIndent::Greater(checker.into_iter().rev().collect()))
        } else {
            // checker.is_empty() && !given_indents.is_empty()
            Ok(CmpIndent::Less(given_indents.into_iter().rev().collect()))
        }
    }

    /// Converts this borrowed view into an [`OwnedLineIndent`] by cloning the sequence.
    pub fn to_owned(&self) -> OwnedLineIndent {
        OwnedLineIndent(self.0.to_vec())
    }
}

impl<'a> Deref for LineIndent<'a> {
    type Target = [Indent];

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

/// An owned indentation sequence. The heap-allocated counterpart of [`LineIndent`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OwnedLineIndent(pub Vec<Indent>);

impl Default for OwnedLineIndent {
    fn default() -> Self {
        Self::new()
    }
}

impl From<Vec<Indent>> for OwnedLineIndent {
    fn from(indents: Vec<Indent>) -> Self {
        Self(indents)
    }
}

impl OwnedLineIndent {
    /// Creates an empty `OwnedLineIndent`.
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Returns a borrowed [`LineIndent`] view over this sequence.
    pub fn as_slice(&self) -> LineIndent<'_> {
        LineIndent(&self.0)
    }
}

impl Deref for OwnedLineIndent {
    type Target = [Indent];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OwnedLineIndent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A stack of strictly increasing indentation levels.
///
/// Each entry must be deeper than the one below it; pushing an equal or shallower level is
/// an error. Used during tokenization to track the nesting of blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndentStack {
    stack: Vec<OwnedLineIndent>,
}

impl Default for IndentStack {
    fn default() -> Self {
        Self::new()
    }
}

impl IndentStack {
    /// Creates an empty `IndentStack`.
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    /// Returns `true` if no indentation levels have been pushed.
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Pushes a new indentation level. Returns an error if `indent` is not strictly deeper than
    /// the current top of the stack.
    pub fn push(&mut self, indent: OwnedLineIndent) -> Result<(), InterpreterError> {
        if let Some(last) = self.stack.last() {
            match last.as_slice().cmp_level(&indent) {
                Ok(CmpIndent::Less(_)) => {
                    self.stack.push(indent);
                    Ok(())
                }
                Ok(CmpIndent::Equal) => Err(InterpreterError::new_lexical_error(String::from(
                    "Inconsistent indentation: duplicate indentation level",
                ))),
                Ok(CmpIndent::Greater(_)) => Err(InterpreterError::new_lexical_error(
                    String::from("Inconsistent indentation: cannot push greater indentation level"),
                )),
                Err(e) => Err(e),
            }
        } else {
            self.stack.push(indent);
            Ok(())
        }
    }

    /// Removes and returns the top indentation level, or `None` if the stack is empty.
    pub fn pop(&mut self) -> Option<OwnedLineIndent> {
        self.stack.pop()
    }

    /// Returns a borrowed view of the current (deepest) indentation level, or `None` if empty.
    pub fn current(&self) -> Option<LineIndent<'_>> {
        self.stack.last().map(|indent| indent.as_slice())
    }
}

impl Deref for IndentStack {
    type Target = [OwnedLineIndent];

    fn deref(&self) -> &Self::Target {
        &self.stack
    }
}

/// Tracks the indentation history of the current parsing session, along with an optional
/// callback for when a new block opens.
pub struct IndentHistory {
    pub stack: IndentStack,
    /// `Some(_)` — a deeper indentation level is expected on the next line; the callback will
    /// receive the completed [`Block`] once the block is closed.
    ///
    /// `None` — the next line is expected at the same or a shallower level.
    ///
    /// [`Block`]: crate::eval::ast::Block
    pub expected_indent: Option<Box<dyn SetBlock>>,
}

impl Debug for IndentHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndentHistory")
            .field("stack", &self.stack)
            .field(
                "expected_indent",
                if self.expected_indent.is_some() {
                    &"Some(SetBlock + Eval)"
                } else {
                    &"None"
                },
            )
            .finish()
    }
}

impl Default for IndentHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl IndentHistory {
    pub fn new() -> Self {
        Self {
            stack: IndentStack::new(),
            expected_indent: None,
        }
    }
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_cmp_level_equal() {
        // Case 1
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            assert!(matches!(indent.cmp_level(&given), Ok(CmpIndent::Equal)));
        }

        // Case 2
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(4).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
            ];

            assert!(matches!(indent.cmp_level(&given), Ok(CmpIndent::Equal)));
        }

        // Case 3
        {
            let indent = LineIndent::from(&[]);
            let given = LineIndent::from(&[]);
            assert!(matches!(indent.cmp_level(&given), Ok(CmpIndent::Equal)));
        }
    }

    #[test]
    fn test_cmp_level_less() {
        // Case 1
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(matches!(result, Ok(CmpIndent::Less(_))));

            match result.unwrap() {
                CmpIndent::Less(indents) => {
                    assert_eq!(indents.len(), 1);
                    assert!(matches!(indents[0], Indent::Tab(t) if t.get() == 2));
                }
                _ => unreachable!(),
            }
        }

        // Case 2
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(3).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(matches!(result, Ok(CmpIndent::Less(_))));

            match result.unwrap() {
                CmpIndent::Less(indents) => {
                    assert_eq!(indents.len(), 2);
                    assert!(matches!(indents[0], Indent::Tab(t) if t.get() == 2));
                    assert!(matches!(indents[1], Indent::Tab(t) if t.get() == 1));
                }
                _ => unreachable!(),
            }
        }

        // Case 3
        {
            let indent = LineIndent::from(&[]);
            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(matches!(result, Ok(CmpIndent::Less(_))));
        }
    }

    #[test]
    fn test_cmp_level_greater() {
        // Case 1
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(matches!(result, Ok(CmpIndent::Greater(_))));

            match result.unwrap() {
                CmpIndent::Greater(indents) => {
                    assert_eq!(indents.len(), 1);
                    assert!(matches!(indents[0], Indent::Tab(t) if t.get() == 2));
                }
                _ => unreachable!(),
            }
        }

        // Case 2
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(matches!(result, Ok(CmpIndent::Greater(_))));

            match result.unwrap() {
                CmpIndent::Greater(indents) => {
                    assert_eq!(indents.len(), 2);
                    assert!(matches!(indents[0], Indent::Tab(t) if t.get() == 1));
                    assert!(matches!(indents[1], Indent::Space(s) if s.get() == 2));
                }
                _ => unreachable!(),
            }
        }

        // Case 4
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let result = indent.cmp_level(&[]);
            assert!(matches!(result, Ok(CmpIndent::Greater(_))));
        }
    }

    #[test]
    fn test_cmp_level_err() {
        // Case 1: not match
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(3).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(result.is_err());
        }

        // Case 2: not match
        {
            let current = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ];

            let indent = LineIndent::from(&current);

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ];

            let result = indent.cmp_level(&given);
            assert!(result.is_err());
        }
    }
}
