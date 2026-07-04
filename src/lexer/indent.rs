use std::{
    fmt::Debug,
    num::NonZero,
    ops::{Deref, DerefMut},
};

use crate::{error::InterpreterError, eval::SetBlock};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Indent {
    Tab(NonZero<usize>),
    Space(NonZero<usize>),
}

#[derive(Debug, Clone)]
pub enum CmpIndent {
    Less(Vec<Indent>),
    Equal,
    Greater(Vec<Indent>),
}

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
    pub fn new() -> Self {
        Self(&[])
    }

    pub fn to_vec(&self) -> Vec<Indent> {
        self.0.to_vec()
    }

    pub fn current_indent(&self) -> Option<&Indent> {
        self.0.last()
    }

    pub fn level(&self) -> usize {
        self.0.len()
    }

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

    /// Less => current indent is less than the given indent, meaning we are going deeper into the
    /// block.
    ///
    /// Equal => current indent is the same as the given indent.
    ///
    /// Greater => current indent is greater than the given indent, meaning we are going back to the
    /// previous block.
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
    pub fn new() -> Self {
        Self(Vec::new())
    }

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
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

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

    pub fn pop(&mut self) -> Option<OwnedLineIndent> {
        self.stack.pop()
    }

    pub fn current(&self) -> Option<LineIndent<'_>> {
        self.stack.last().map(|indent| indent.as_slice())
    }
}

pub struct IndentHistory {
    pub stack: Vec<OwnedLineIndent>,
    /// Some(_) -> a new level of indentation is expected and the Block should be set
    /// None -> the same level of indentation or a lower level of indentation is expected
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
            stack: Vec::new(),
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
            println!("Result: {:?}", result);
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
