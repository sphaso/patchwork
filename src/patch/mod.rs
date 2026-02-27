mod types;
pub use types::*;

use crate::myers::Edit;
use crate::serialization::PatchError;
use std::collections::VecDeque;

struct HunkBuilder<T> {
    old_line: usize,
    new_line: usize,
    current: Option<Hunk<T>>,
    trailing_equal_count: usize,
    context_buffer: VecDeque<Edit<T>>,
    hunks: Vec<Hunk<T>>,
}

impl<T: Eq + Clone> HunkBuilder<T> {
    fn new() -> Self {
        HunkBuilder {
            old_line: 0,
            new_line: 0,
            current: None,
            trailing_equal_count: 0,
            context_buffer: VecDeque::new(),
            hunks: vec![],
        }
    }

    fn process(&mut self, edit: Edit<T>) {
        match edit {
            Edit::Equal(el) => {
                self.context_buffer.push_back(Edit::Equal(el.clone()));
                while self.context_buffer.len() > 3 {
                    self.context_buffer.pop_front();
                }

                if let Some(ref mut c) = self.current {
                    c.changes.push(Edit::Equal(el));
                    self.trailing_equal_count += 1;
                    if self.trailing_equal_count >= 3 {
                        self.hunks.push(self.current.take().unwrap());
                        self.current = None;
                    }
                }
                self.old_line += 1;
                self.new_line += 1;
            }
            modify => {
                self.trailing_equal_count = 0;
                if let Some(ref mut c) = self.current {
                    c.changes.push(modify.clone());
                } else {
                    let mut changes = vec![];
                    let old_start = self.old_line - self.context_buffer.len();
                    let new_start = self.new_line - self.context_buffer.len();
                    while let Some(e) = self.context_buffer.pop_front() {
                        changes.push(e);
                    }
                    changes.push(modify.clone());
                    self.current = Some(Hunk {
                        old_start,
                        new_start,
                        changes,
                    });
                };

                match modify {
                    Edit::Insert(_) => self.new_line += 1,
                    _ => self.old_line += 1,
                }
            }
        }
    }

    fn finish(mut self) -> Vec<Hunk<T>> {
        if let Some(c) = self.current {
            self.hunks.push(c);
        }
        self.hunks
    }
}

pub fn hunks<T: Eq + Clone>(edits: Vec<Edit<T>>) -> Vec<Hunk<T>> {
    let mut builder = HunkBuilder::new();
    for edit in edits {
        builder.process(edit);
    }
    builder.finish()
}

pub fn apply(old: &[String], hunks: &[Hunk<String>]) -> Result<Vec<String>, PatchError> {
    if old.is_empty() {
        return Ok(hunks
            .into_iter()
            .flat_map(|h| h.changes.iter())
            .filter_map(|e| match e {
                Edit::Insert(t) => Some(t.clone()),
                _ => None,
            })
            .collect());
    }

    if hunks.is_empty() {
        return Ok(old.to_vec());
    }

    let mut result = vec![];
    let mut hunk_iter = hunks.iter().peekable();
    let mut old_line = 0;

    while old_line < old.len() {
        if let Some(hunk) = hunk_iter.peek() {
            if old_line == hunk.old_start {
                for change in &hunk.changes {
                    match change {
                        Edit::Equal(t) => {
                            if old[old_line] != *t {
                                return Err(PatchError::InvalidFormat(format!(
                                    "Context mismatch at line {}: expected '{}', found '{}'",
                                    old_line, t, old[old_line]
                                )));
                            }
                            result.push(old[old_line].clone());
                            old_line += 1;
                        }
                        Edit::Insert(t) => {
                            result.push(t.clone());
                        }
                        Edit::Delete(_) => {
                            old_line += 1;
                        }
                    }
                }
                hunk_iter.next();
            } else if old_line < hunk.old_start {
                result.push(old[old_line].clone());
                old_line += 1;
            } else {
                return Err(PatchError::InvalidFormat("Cannot apply hunks".to_string()));
            }
        } else {
            result.push(old[old_line].clone());
            old_line += 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::myers::{diff, Edit};
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_all_changes_covered(
            old in prop::collection::vec(any::<u8>(), 0..20),
            new in prop::collection::vec(any::<u8>(), 0..20),
        ) {
            let edits = diff(&old, &new);
            let result = hunks(edits.clone());

            let all_hunk_edits: Vec<Edit<u8>> = result.iter()
                .flat_map(|h| h.changes.iter().cloned())
                .collect();

            for edit in &edits {
                if !matches!(edit, Edit::Equal(_)) {
                    prop_assert!(all_hunk_edits.contains(edit));
                }
            }
        }

        #[test]
        fn test_apply_roundtrip(
                    old in prop::collection::vec(".*", 0..20usize),
        new in prop::collection::vec(".*", 0..20usize),
            ) {
            let edits = diff(&old, &new);
            let hunks = hunks(edits.clone());
            let result = apply(&old, &hunks);
            assert_eq!(result, Ok(new));
        }
    }

    #[test]
    fn test_single_hunk() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![1, 2, 99, 4, 5];
        let expected_hunks = vec![Hunk {
            old_start: 0,
            new_start: 0,
            changes: vec![
                Edit::Equal(1),
                Edit::Equal(2),
                Edit::Insert(99),
                Edit::Delete(3),
                Edit::Equal(4),
                Edit::Equal(5),
            ],
        }];
        let edits = diff(&old, &new);
        let result = hunks(edits);
        assert_eq!(result, expected_hunks);
    }

    #[test]
    fn test_two_hunks() {
        // two changes far apart, should produce two hunks
        let old = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let new = vec![99, 2, 3, 4, 5, 6, 7, 8, 9, 99];
        let expected_hunks = vec![
            Hunk {
                old_start: 0,
                new_start: 0,
                changes: vec![
                    Edit::Insert(99),
                    Edit::Delete(1),
                    Edit::Equal(2),
                    Edit::Equal(3),
                    Edit::Equal(4),
                ],
            },
            Hunk {
                old_start: 6,
                new_start: 6,
                changes: vec![
                    Edit::Equal(7),
                    Edit::Equal(8),
                    Edit::Equal(9),
                    Edit::Insert(99),
                    Edit::Delete(10),
                ],
            },
        ];
        let edits = diff(&old, &new);
        let result = hunks(edits);
        assert_eq!(result, expected_hunks);
    }

    #[test]
    fn test_change_at_start() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![99, 2, 3, 4, 5];
        let expected_hunks = vec![Hunk {
            old_start: 0,
            new_start: 0,
            changes: vec![
                Edit::Insert(99),
                Edit::Delete(1),
                Edit::Equal(2),
                Edit::Equal(3),
                Edit::Equal(4),
            ],
        }];
        let edits = diff(&old, &new);
        let result = hunks(edits);
        assert_eq!(result, expected_hunks);
    }

    #[test]
    fn test_change_at_end() {
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![1, 2, 3, 4, 99];
        let expected_hunks = vec![Hunk {
            old_start: 1,
            new_start: 1,
            changes: vec![
                Edit::Equal(2),
                Edit::Equal(3),
                Edit::Equal(4),
                Edit::Insert(99),
                Edit::Delete(5),
            ],
        }];
        let edits = diff(&old, &new);
        let result = hunks(edits);
        assert_eq!(result, expected_hunks);
    }

    #[test]
    fn test_no_changes() {
        let old = vec![1, 2, 3, 4, 5];
        let edits = diff(&old, &old);
        let result = hunks(edits);
        assert_eq!(result, vec![]);
    }

    #[test]
    fn test_apply_change_in_middle() {
        let old = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let new = vec![
            "a".to_string(),
            "b".to_string(),
            "X".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let edits = diff(&old, &new);
        let hunks = hunks(edits);
        let result = apply(&old, &hunks);
        assert_eq!(result, Ok(new));
    }

    #[test]
    fn test_apply_multiple_hunks() {
        let old = vec!["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let new = vec!["X", "b", "c", "d", "e", "f", "g", "h", "i", "Y"]
            .into_iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let edits = diff(&old, &new);
        let hunks = hunks(edits);
        let result = apply(&old, &hunks);
        assert_eq!(result, Ok(new));
    }

    #[test]
    fn test_apply_invalid_patch() {
        let old = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let bad_hunk = Hunk {
            old_start: 0,
            new_start: 0,
            changes: vec![
                Edit::Equal("x".to_string()), // but old[0] is "a", mismatch!
                Edit::Delete("y".to_string()),
                Edit::Insert("z".to_string()),
            ],
        };

        let result = apply(&old, &[bad_hunk]);
        assert!(result.is_err());
    }
}
