mod diffable;
mod types;

pub use diffable::*;
pub use types::*;

use crate::myers;
use crate::myers::Edit;
use std::collections::{HashMap, HashSet};

pub fn diff<T: Diffable>(old: &T, new: &T) -> Vec<Change<T::P>> {
    diff_nodes(old.to_node(), new.to_node(), vec![])
}

fn diff_nodes<P: Primitive>(old: Node<P>, new: Node<P>, path: Vec<PathSegment>) -> Vec<Change<P>> {
    match (old, new) {
        (Node::Leaf(a), Node::Leaf(b)) => {
            if a != b {
                vec![Change {
                    path,
                    kind: ChangeKind::Modified(a, b),
                }]
            } else {
                vec![]
            }
        }
        (Node::Sequence(a), Node::Sequence(b)) => {
            let result = myers::diff(&a, &b);
            if result.iter().all(|e| matches!(e, Edit::Equal(_))) {
                vec![]
            } else {
                vec![Change {
                    path,
                    kind: ChangeKind::SequenceChange(result),
                }]
            }
        }
        (Node::Map(a), Node::Map(b)) => {
            let keys_a = a.keys().collect::<HashSet<_>>();
            let keys_b = b.keys().collect::<HashSet<_>>();

            keys_a
                .union(&keys_b)
                .flat_map(|key| {
                    let mut new_path = path.clone();
                    new_path.push(PathSegment::Key(key.to_string()));
                    match (a.get(*key), b.get(*key)) {
                        (Some(va), Some(vb)) => diff_nodes(va.clone(), vb.clone(), new_path),
                        (Some(va), None) => match va {
                            Node::Leaf(ve) => vec![Change {
                                path: new_path,
                                kind: ChangeKind::Removed(ve.clone()),
                            }],
                            ve => vec![Change {
                                path: new_path,
                                kind: ChangeKind::StructureRemoved(ve.clone()),
                            }],
                        },
                        (None, Some(vb)) => match vb {
                            Node::Leaf(ve) => vec![Change {
                                path: new_path,
                                kind: ChangeKind::Added(ve.clone()),
                            }],
                            ve => vec![Change {
                                path: new_path,
                                kind: ChangeKind::StructureAdded(ve.clone()),
                            }],
                        },
                        (None, None) => unreachable!(),
                    }
                })
                .collect()
        }
        (old, new) => vec![
            Change {
                path: path.clone(),
                kind: ChangeKind::StructureRemoved(old),
            },
            Change {
                path,
                kind: ChangeKind::StructureAdded(new),
            },
        ],
    }
}

pub fn apply<T: Diffable>(old: &T, changes: &[Change<T::P>]) -> T {
    let new_node = changes
        .iter()
        .fold(old.to_node(), |acc, e| apply_change(acc, e));
    T::from_node(new_node)
}

fn apply_change<P: Primitive>(node: Node<P>, change: &Change<P>) -> Node<P> {
    match (node, change.path.first()) {
        (Node::Map(m), Some(PathSegment::Key(k))) => apply_to_map(m, k, change),
        (Node::Sequence(_), _) => match &change.kind {
            ChangeKind::SequenceChange(edits) => apply_to_sequence(edits.to_vec()),
            _ => unreachable!(),
        },

        (Node::Leaf(_), _) => match &change.kind {
            ChangeKind::Modified(_, new) => Node::Leaf(new.clone()),
            _ => unreachable!(),
        },
        (Node::Map(_), _) => unreachable!(),
    }
}

fn apply_to_map<P: Primitive>(
    map: HashMap<String, Node<P>>,
    key: &String,
    change: &Change<P>,
) -> Node<P> {
    let mut new_map = map.clone();
    let node = if change.path.len() > 1 {
        let new_change = Change {
            kind: change.kind.clone(),
            path: change.path[1..].to_vec(),
        };
        new_map.insert(
            key.to_string(),
            apply_change(map.get(key).unwrap().clone(), &new_change),
        );
        new_map
    } else {
        match &change.kind {
            ChangeKind::StructureAdded(new) => new_map.insert(key.to_string(), new.clone()),
            ChangeKind::Added(new) => new_map.insert(key.to_string(), Node::Leaf(new.clone())),
            ChangeKind::StructureRemoved(_) | ChangeKind::Removed(_) => new_map.remove(key),
            ChangeKind::Modified(_, new) => {
                new_map.insert(key.to_string(), Node::Leaf(new.clone()))
            }
            _ => unreachable!(),
        };
        new_map
    };

    Node::Map(node)
}

fn apply_to_sequence<P: Primitive>(edits: Vec<Edit<Node<P>>>) -> Node<P> {
    let mut result = vec![];
    for edit in edits {
        match edit {
            Edit::Equal(v) => result.push(v.clone()),
            Edit::Insert(v) => result.push(v.clone()),
            Edit::Delete(_) => {}
        }
    }
    Node::Sequence(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_leaf_modified() {
        let mut a = HashMap::new();
        a.insert("a".to_string(), 1);
        let mut b = HashMap::new();
        b.insert("a".to_string(), 2);
        let result = diff(&a, &b);
        assert_eq!(
            result,
            vec![Change {
                path: vec![PathSegment::Key("a".to_string())],
                kind: ChangeKind::Modified(1, 2)
            }]
        );
    }

    #[test]
    fn test_key_added() {
        let mut a = HashMap::new();
        a.insert("a".to_string(), 1);
        let mut b = HashMap::new();
        b.insert("a".to_string(), 1);
        b.insert("c".to_string(), 2);
        let result = diff(&a, &b);
        assert_eq!(
            result,
            vec![Change {
                path: vec![PathSegment::Key("c".to_string())],
                kind: ChangeKind::Added(2)
            }]
        );
    }

    #[test]
    fn test_key_removed() {
        let mut a = HashMap::new();
        a.insert("a".to_string(), 1);
        a.insert("c".to_string(), 2);
        let mut b = HashMap::new();
        b.insert("a".to_string(), 1);
        let result = diff(&a, &b);
        assert_eq!(
            result,
            vec![Change {
                path: vec![PathSegment::Key("c".to_string())],
                kind: ChangeKind::Removed(2)
            }]
        );
    }

    #[test]
    fn test_nested_map() {
        let mut a = HashMap::new();
        let mut nested_a = HashMap::new();
        nested_a.insert("nested".to_string(), 1);
        a.insert("b".to_string(), nested_a);
        let mut b = HashMap::new();
        let mut nested_b = HashMap::new();
        nested_b.insert("nested".to_string(), 2);
        b.insert("b".to_string(), nested_b);
        let result = diff(&a, &b);
        assert_eq!(
            result,
            vec![Change {
                path: vec![
                    PathSegment::Key("b".to_string()),
                    PathSegment::Key("nested".to_string())
                ],
                kind: ChangeKind::Modified(1, 2)
            }]
        );
    }

    #[test]
    fn test_sequence_of_primitives() {
        let a = vec![1, 2, 3];
        let b = vec![1, 3, 4];
        let result = diff(&a, &b);
        assert_eq!(
            result,
            vec![Change {
                path: vec![],
                kind: ChangeKind::SequenceChange(vec![
                    Edit::Equal(Node::Leaf(1)),
                    Edit::Delete(Node::Leaf(2)),
                    Edit::Equal(Node::Leaf(3)),
                    Edit::Insert(Node::Leaf(4))
                ])
            }]
        );
    }

    #[test]
    fn test_no_changes() {
        let a = vec![1, 2, 3];
        let result = diff(&a, &a);

        assert_eq!(result, vec![]);
    }
}
