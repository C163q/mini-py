use std::num::NonZero;

use crate::error::InterpreterError;

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone)]
pub struct IndentStack {
    stack: Vec<Indent>,
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

    pub fn push(&mut self, indent: Indent) {
        self.stack.push(indent);
    }

    pub fn pop(&mut self) -> Option<Indent> {
        self.stack.pop()
    }

    pub fn current_indent(&self) -> Option<&Indent> {
        self.stack.last()
    }

    pub fn level(&self) -> usize {
        self.stack.len()
    }

    pub fn is_same_level(&self, indents: &[Indent]) -> Result<bool, InterpreterError> {
        let mut checker: Vec<Indent> = self.stack.iter().copied().rev().collect();
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
                    return Err(InterpreterError::new(String::from(
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
        let mut checker: Vec<Indent> = self.stack.iter().copied().rev().collect();
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
                    return Err(InterpreterError::new(String::from(
                        "Inconsistent indentation: mixing tabs and spaces is not allowed",
                    )));
                }
            }
        }
        if given_indents.is_empty() && checker.is_empty() {
            Ok(CmpIndent::Equal)
        } else if given_indents.is_empty() {
            // ```text
            // given:
            // <Tab><Space><Space>
            // [Tab(1), Space(2)]
            //
            // indent history:
            // <Tab>
            // <Tab><Space><Space><Space><Space>
            // [Tab(1), Space(4)]
            //
            // remain:
            // [Space(2)]
            // ```
            //
            // This is an ERROR.
            let last = checker.last().unwrap();
            let cmp = self.stack[self.stack.len() - checker.len()];
            match (last, cmp) {
                (Indent::Tab(_), Indent::Space(_)) | (Indent::Space(_), Indent::Tab(_)) => {
                    unreachable!();
                }
                (Indent::Tab(found), Indent::Tab(expect)) => {
                    if *found == expect {
                        Ok(CmpIndent::Greater(checker.into_iter().rev().collect()))
                    } else {
                        Err(InterpreterError::new(format!(
                            "Inconsistent indentation: expected tabs of {}, got {}",
                            expect.get(),
                            found.get()
                        )))
                    }
                }
                (Indent::Space(found), Indent::Space(expect)) => {
                    if *found == expect {
                        Ok(CmpIndent::Greater(checker.into_iter().rev().collect()))
                    } else {
                        Err(InterpreterError::new(format!(
                            "Inconsistent indentation: expected spaces of {}, got {}",
                            expect.get(),
                            found.get()
                        )))
                    }
                }
            }
        } else {
            // Now the given_indents probably has mixed tabs and spaces, this is a error.
            let is_space = matches!(given_indents.first().unwrap(), Indent::Space(_));
            for indent in &given_indents {
                match indent {
                    Indent::Tab(_) if is_space => {
                        return Err(InterpreterError::new(String::from(
                            "Inconsistent indentation: mixing tabs and spaces is not allowed",
                        )));
                    }
                    Indent::Space(_) if !is_space => {
                        return Err(InterpreterError::new(String::from(
                            "Inconsistent indentation: mixing tabs and spaces is not allowed",
                        )));
                    }
                    _ => {}
                }
            }
            Ok(CmpIndent::Less(given_indents.into_iter().rev().collect()))
        }
    }
}

impl Extend<Indent> for IndentStack {
    fn extend<T: IntoIterator<Item = Indent>>(&mut self, iter: T) {
        self.stack.extend(iter);
    }
}

impl FromIterator<Indent> for IndentStack {
    fn from_iter<T: IntoIterator<Item = Indent>>(iter: T) -> Self {
        Self {
            stack: iter.into_iter().collect(),
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
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            assert!(matches!(stack.cmp_level(&given), Ok(CmpIndent::Equal)));
        }

        // Case 2
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(4).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
            ];

            assert!(matches!(stack.cmp_level(&given), Ok(CmpIndent::Equal)));
        }
    }

    #[test]
    fn test_cmp_level_less() {
        // Case 1
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ];

            let result = stack.cmp_level(&given);
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
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Tab(NonZero::new(3).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ];

            let result = stack.cmp_level(&given);
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

        // Case 3: given indents has mixed tabs and spaces, this is an error.
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let result = stack.cmp_level(&given);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_cmp_level_greater() {
        // Case 1
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ];

            let result = stack.cmp_level(&given);
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
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let result = stack.cmp_level(&given);
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

        // Case 3: given indents don't match any indent in the stack, this is an error.
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
            ];

            let result = stack.cmp_level(&given);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_cmp_level_err() {
        // Case 1: not match
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(3).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
            ];

            let result = stack.cmp_level(&given);
            assert!(result.is_err());
        }

        // Case 2: not match
        {
            let stack: IndentStack = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(4).unwrap()),
                Indent::Tab(NonZero::new(2).unwrap()),
            ]
            .into_iter()
            .collect();

            let given = [
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Space(NonZero::new(2).unwrap()),
                Indent::Space(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
                Indent::Tab(NonZero::new(1).unwrap()),
            ];

            let result = stack.cmp_level(&given);
            assert!(result.is_err());
        }
    }
}
