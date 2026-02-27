use crate::myers::Edit;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk<T> {
    pub old_start: usize,
    pub new_start: usize,
    pub changes: Vec<Edit<T>>,
}
