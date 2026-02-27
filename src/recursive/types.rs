use crate::myers::types::Edit;
use std::collections::HashMap;

/// Represents a single change in a possibly recursive structure
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Change<P: Primitive> {
    pub path: Vec<PathSegment>,
    pub kind: ChangeKind<P>,
}

/// Represents either a list index or a map key
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PathSegment {
    Key(String),  // map key
    Index(usize), // sequence index
}

/// Represents a change in a possibly recursive structure.
///
/// `Added`, `Removed`, `Modified` are actions on leaves.
/// `NodeAdded`, `NodeRemoved` are actions on nodes.
/// `SequenceChange` contains the raw Myers edit script for a sequence.
///
/// # Note
///
/// We don't diff recursively inside lists as Rust lacks facilities
/// to dispatch between `Vec<Primitive>` and `Vec<Node<Primitive>>`.
/// For this reason we always apply Myers.
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ChangeKind<P: Primitive> {
    Added(P),
    NodeAdded(Node<P>),
    Removed(P),
    NodeRemoved(Node<P>),
    Modified(P, P), // old, new
    SequenceChange(Vec<Edit<Node<P>>>),
}

/// Represents a single Node.
/// We transform input structures into Node trees in order to recursively diff them
#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Node<P: Primitive> {
    Map(HashMap<String, Node<P>>),
    Sequence(Vec<Node<P>>),
    Leaf(P),
}

/// Trait for leaf values in a Node tree.
/// Implemented for all Rust primitives except floats, which lack `[Eq]`
pub trait Primitive: Eq + Clone {}
