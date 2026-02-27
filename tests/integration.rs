use patchwork::recursive::*;
use proptest::prelude::*;
use std::collections::HashMap;

// only flat for now
proptest! {
    #[test]
    fn test_round_trip_map(
        old in prop::collection::hash_map(".*", any::<i32>(), 0..10),
        new in prop::collection::hash_map(".*", any::<i32>(), 0..10),
    ) {
        let changes = diff(&old, &new);
        let result = apply(&old, &changes);
        prop_assert_eq!(result, new);
    }

    #[test]
    fn test_round_trip_vec(
        old in prop::collection::vec(any::<i32>(), 0..10),
        new in prop::collection::vec(any::<i32>(), 0..10),
    ) {
        let changes = diff(&old, &new);
        let result = apply(&old, &changes);
        prop_assert_eq!(result, new);
    }
}

// nested structures
#[test]
fn test_apply_round_trip_seq_with_maps() {
    let mut old = vec![];
    let mut a = HashMap::new();
    let mut b = HashMap::new();
    a.insert("a".to_string(), 1);
    b.insert("b".to_string(), 2);
    old.push(a);
    old.push(b);

    let mut new = vec![];
    let mut an = HashMap::new();
    let mut c = HashMap::new();
    an.insert("a".to_string(), 1);
    c.insert("c".to_string(), 2);
    new.push(an);
    new.push(c);

    let changes = diff(&old, &new);
    let result = apply(&old, &changes);
    assert_eq!(result, new);
}

#[test]
fn test_apply_nested_map() {
    let mut old = HashMap::new();
    let mut nested_a = HashMap::new();
    nested_a.insert("nested".to_string(), 1);
    old.insert("b".to_string(), nested_a);
    let mut new = HashMap::new();
    let mut nested_b = HashMap::new();
    nested_b.insert("nested".to_string(), 2);
    new.insert("b".to_string(), nested_b);
    let changes = diff(&old, &new);
    let result = apply(&old, &changes);
    assert_eq!(result, new);
}
