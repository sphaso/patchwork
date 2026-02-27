use crate::recursive::types::*;
use std::collections::HashMap;

/// Trait to transform a given structure into a `[Node]` tree or viceversa.
///
/// Exposes two functions:
/// `to_node` transforms a structure into a `[Node]` tree
/// `from_node` transforms a `[Node]` tree into the initial structure
///
/// It's implemented for `Vec<T>`, `HashMap<String, T>` where T : Diffable
/// as well as Rust primitives except floats which lack `[Eq]`.
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
