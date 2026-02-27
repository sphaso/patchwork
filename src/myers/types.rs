pub type Diff<T> = Vec<Edit<T>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit<T> {
    Insert(T),
    Delete(T),
    Equal(T),
}
