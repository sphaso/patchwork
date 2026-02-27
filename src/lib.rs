pub mod diff;

type Diff<T> = Vec<Change<T>>;
type Patch<U, T> = Vec<Hunk<U, T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk<U, T> {
    old_start: U,
    new_start: U,
    changes: Vec<Change<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Change<T> {
    Insert(T),
    Delete(T),
    Equal(T),
}
