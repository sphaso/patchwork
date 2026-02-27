use crate::myers::types::Edit;
use std::collections::HashMap;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct Change<P: Primitive> {
    pub path: Vec<PathSegment>,
    pub kind: ChangeKind<P>,
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum PathSegment {
    Key(String),  // map key
    Index(usize), // sequence index
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum ChangeKind<P: Primitive> {
    Added(P),
    NodeAdded(Node<P>),
    Removed(P),
    NodeRemoved(Node<P>),
    Modified(P, P), // old, new
    SequenceChange(Vec<Edit<Node<P>>>),
}

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum Node<P: Primitive> {
    Map(HashMap<String, Node<P>>),
    Sequence(Vec<Node<P>>),
    Leaf(P),
}

pub trait Primitive: Eq + Clone {}
