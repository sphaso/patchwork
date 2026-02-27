use crate::myers::Edit;
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
    StructureAdded(Node<P>),
    Removed(P),
    StructureRemoved(Node<P>),
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

pub trait Diffable {
    type P: Primitive;
    fn to_node(&self) -> Node<Self::P>;
    fn from_node(node: Node<Self::P>) -> Self;
}

impl<T: Diffable> Diffable for Vec<T> {
    type P = T::P;
    fn to_node(&self) -> Node<T::P> {
        Node::Sequence(self.iter().map(|e| e.to_node()).collect())
    }

    fn from_node(node: Node<Self::P>) -> Self {
        match node {
            Node::Sequence(v) => v.into_iter().map(|e| T::from_node(e)).collect(),
            _ => unreachable!(),
        }
    }
}

impl<T: Diffable> Diffable for HashMap<String, T> {
    type P = T::P;
    fn to_node(&self) -> Node<T::P> {
        Node::Map(self.iter().map(|(k, v)| (k.clone(), v.to_node())).collect())
    }

    fn from_node(node: Node<Self::P>) -> Self {
        match node {
            Node::Map(v) => v.into_iter().map(|(k, v)| (k, T::from_node(v))).collect(),
            _ => unreachable!(),
        }
    }
}

macro_rules! impl_diffable_leaf {
    ($($t:ty),*) => {
        $(
            impl Primitive for $t {}
            impl Diffable for $t {
                type P = $t;
                fn to_node(&self) -> Node<Self::P> {
                    Node::Leaf(self.clone())
                }

                fn from_node(node : Node<Self::P>) -> Self {
                    match node {
                        Node::Leaf(v) => v,
                        _ => unreachable!()
                    }
                }
            }
        )*
    };
}

impl_diffable_leaf!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, bool, String, char
);
