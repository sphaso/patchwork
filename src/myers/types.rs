/// Alias for a vector of Edit
/// Result of the Myers diff function
pub type Diff<T> = Vec<Edit<T>>;

/// Each element in a diff can be
/// new (Insert)
/// removed (Delete)
/// equal (Equal)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit<T> {
    Insert(T),
    Delete(T),
    Equal(T),
}
