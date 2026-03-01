use crate::myers::Edit;

/// Represents a Hunk resulting from a Myers diff.
/// Please note that `changes` will include maximum 3 context elements, i.e. `Edit::Equal`
/// and this is reflected in the `old_start` value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk<T> {
    pub old_start: usize,
    pub new_start: usize,
    pub changes: Vec<Edit<T>>,
}
