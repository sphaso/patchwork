pub mod myers;
pub mod patch;
pub mod recursive;

use crate::myers::Edit;
use crate::recursive::*;
use std::collections::HashSet;

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
            result
                .iter()
                .fold(
                    (0, 0, vec![]),
                    |(old_idx, new_idx, mut changes), edit| match edit {
                        Edit::Insert(Node::Leaf(v)) => {
                            let mut new_path = path.clone();
                            new_path.push(PathSegment::Index(new_idx));
                            changes.push(Change {
                                path: new_path,
                                kind: ChangeKind::Added(v.clone()),
                            });
                            (old_idx, new_idx + 1, changes)
                        }
                        Edit::Insert(v) => {
                            let mut new_path = path.clone();
                            new_path.push(PathSegment::Index(new_idx));
                            changes.push(Change {
                                path: new_path,
                                kind: ChangeKind::StructureAdded(v.clone()),
                            });
                            (old_idx, new_idx + 1, changes)
                        }
                        Edit::Delete(Node::Leaf(v)) => {
                            let mut new_path = path.clone();
                            new_path.push(PathSegment::Index(old_idx));
                            changes.push(Change {
                                path: new_path,
                                kind: ChangeKind::Removed(v.clone()),
                            });
                            (old_idx + 1, new_idx, changes)
                        }
                        Edit::Delete(v) => {
                            let mut new_path = path.clone();
                            new_path.push(PathSegment::Index(old_idx));
                            changes.push(Change {
                                path: new_path,
                                kind: ChangeKind::StructureRemoved(v.clone()),
                            });
                            (old_idx + 1, new_idx, changes)
                        }
                        Edit::Equal(_) => (old_idx + 1, new_idx + 1, changes),
                    },
                )
                .2
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
            vec![
                Change {
                    path: vec![PathSegment::Index(1)],
                    kind: ChangeKind::Removed(2)
                },
                Change {
                    path: vec![PathSegment::Index(2)],
                    kind: ChangeKind::Added(4)
                },
            ]
        );
    }

    #[test]
    fn test_no_changes() {
        let a = vec![1, 2, 3];
        let result = diff(&a, &a);
        assert_eq!(result, vec![]);
    }
}
