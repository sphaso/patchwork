use crate::{Change, Diff};
use std::cmp::max;

#[derive(Clone)]
struct V {
    data: Vec<usize>,
    offset: isize,
}

impl V {
    fn new(size: usize) -> Self {
        V { data: vec![0; 2 * size + 1], offset: size as isize }
    }

    fn get(&self, k: isize) -> usize {
        self.data[(k + self.offset) as usize]
    }

    fn set(&mut self, k: isize, val: usize) {
        self.data[(k + self.offset) as usize] = val;
    }
}

pub fn diff<T: Eq + Clone>(old: &[T], new: &[T]) -> Diff<T> {
    if old.is_empty() {
        return new.iter().map(|e| Change::Insert(e.clone())).collect();
    }
    if new.is_empty() {
        return old.iter().map(|e| Change::Delete(e.clone())).collect();
    }

    let n = old.len();
    let m = new.len();
    let maxi = n + m;
    let mut v = V::new(maxi);
    let mut trace: Vec<V> = Vec::new();
    let mut end_x = n;
let mut end_y = m;
    'edits: for d in 0..=maxi as isize {
        for k in (-d..=d).step_by(2) {
            let mut x = if k == -d {
                v.get(k + 1)
            } else if k == d {
                v.get(k - 1) + 1
            } else {
                max(v.get(k + 1), v.get(k - 1) + 1)
            };
            let mut y = (x as isize - k) as usize;
            while x < n && y < m && old[x] == new[y] {
                x += 1;
                y += 1;
            }
            v.set(k, x);
            if x >= n && y >= m {
                end_x = x;
                end_y = y;
                trace.push(v.clone());
                break 'edits;
            }
        }
        trace.push(v.clone());
    }
    traceback(old, new, trace, end_x, end_y)
}

fn traceback<T: Eq + Clone>(old: &[T], new: &[T], trace: Vec<V>, mut x: usize, mut y: usize) -> Diff<T> {
    let maxi = old.len() + new.len();
    let mut changes: Diff<T> = Vec::new();
    for d in (0..trace.len()).rev() {
        let d = d as isize;
        let k = x as isize - y as isize;
        let prev_k = if k == -d {
            k + 1
        } else if k == d {
            k - 1
        } else {
            if trace[d as usize].get(k - 1) + 1
                >= trace[d as usize].get(k + 1)
            {
                k - 1
            } else {
                k + 1
            }
        };
        let prev_x = trace[d as usize].get(prev_k);
        let prev_y = prev_x as isize - prev_k;
        while x as isize > prev_x as isize
            && y as isize > prev_y as isize
            && old[x - 1] == new[y - 1]
        {
            changes.push(Change::Equal(old[x - 1].clone()));
            x -= 1;
            y -= 1;
        }
        if d > 0 {
            if prev_k == k - 1 {
                changes.push(Change::Delete(old[x - 1].clone()))
            } else {
                changes.push(Change::Insert(new[y - 1].clone()))
            }
        }
        x = prev_x as usize;
        y = prev_y as usize;
    }
    while x > 0 && y > 0 {
        changes.push(Change::Equal(old[x - 1].clone()));
        x -= 1;
        y -= 1;
    }

    changes.reverse();
    changes
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_length_invariant(old: Vec<u8>, new: Vec<u8>) {
            let result = diff(&old, &new);
            let deletes = result.iter().filter(|c| matches!(c, Change::Delete(_))).count();
            let equals = result.iter().filter(|c| matches!(c, Change::Equal(_))).count();
            let inserts = result.iter().filter(|c| matches!(c, Change::Insert(_))).count();
            assert_eq!(old.len(), deletes + equals);
            assert_eq!(new.len(), inserts + equals);
        }

        #[test]
        fn test_idempotency(els: Vec<u8>) {
            let result = diff(&els, &els);
            let expected : Diff<u8> = els.iter().map(|e| Change::Equal(e.clone())).collect();
            assert_eq!(result, expected);
        }

        #[test]
        fn test_new_empty(els: Vec<u8>) {
            let result = diff(&els, &Vec::new());
            let expected : Diff<u8> = els.iter().map(|e| Change::Delete(e.clone())).collect();
            assert_eq!(result, expected);
        }

        #[test]
        fn test_old_empty(els: Vec<u8>) {
            let result = diff(&Vec::new(), &els);
            let expected : Diff<u8> = els.iter().map(|e| Change::Insert(e.clone())).collect();
            assert_eq!(result, expected);
        }

        #[test]
        fn test_symmetry(old: Vec<u8>, new: Vec<u8>) {
            let result = diff(&old, &new);
            let result_2 = diff(&new, &old);
            let deletes = result.iter().filter(|c| matches!(c, Change::Delete(_))).count();
            let deletes_2 = result_2.iter().filter(|c| matches!(c, Change::Delete(_))).count();
            let equals = result.iter().filter(|c| matches!(c, Change::Equal(_))).count();
            let equals_2 = result_2.iter().filter(|c| matches!(c, Change::Equal(_))).count();
            let inserts = result.iter().filter(|c| matches!(c, Change::Insert(_))).count();
            let inserts_2 = result_2.iter().filter(|c| matches!(c, Change::Insert(_))).count();

            assert_eq!(equals, equals_2);
            assert_eq!(inserts, deletes_2);
            assert_eq!(deletes, inserts_2);
        }
    }

    #[test]
    fn test_simple_diff() {
        let old = vec!["a", "b", "c"];
        let new = vec!["a", "x", "c"];
        let result = diff(&old, &new);
        assert_eq!(
            result,
            [
                Change::Equal("a"),
                Change::Insert("x"),
                Change::Delete("b"),
                Change::Equal("c")
            ]
        );
    }

    #[test]
    fn test_completely_different() {
        let old = vec!["a", "b", "c"];
        let new = vec!["x", "y", "z"];
        let result = diff(&old, &new);
         assert_eq!(result, vec![Change::Insert("x"), Change::Insert("y"), Change::Insert("z"), Change::Delete("a"), Change::Delete("b"), Change::Delete("c")])
    }

    #[test]
    fn test_single_element_different() {
        let old = vec!["a"];
        let new = vec!["b"];
        let result = diff(&old, &new);
        assert_eq!(result, vec![Change::Insert("b"), Change::Delete("a")]);
    }

    #[test]
    fn test_duplicates() {
        let old = vec!["a", "a", "b"];
        let new = vec!["a", "b", "b"];
        let result = diff(&old, &new);
        assert_eq!(result, vec![Change::Equal("a"), Change::Delete("a"), Change::Equal("b"), Change::Insert("b")]);
    }

    #[test]
    fn test_insertion_in_middle() {
        let old = vec!["a", "c"];
        let new = vec!["a", "b", "c"];
        let result = diff(&old, &new);
        assert_eq!(result, vec![Change::Equal("a"), Change::Insert("b"), Change::Equal("c")]);
    }
}
