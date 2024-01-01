use std::collections::BTreeSet;

/// Helper function to extract first element in the BTreeSet.
pub(crate) fn pop_first<T: Ord + Eq + Clone>(set: &mut BTreeSet<T>) -> Option<T> {
    let first = set.iter().next()?.clone();
    set.remove(&first);
    Some(first)
}

/// Helper function to check whether two sets intersect.
pub(crate) fn has_intersection<T: Ord + Eq>(lhs: &BTreeSet<T>, rhs: &BTreeSet<T>) -> bool {
    lhs.intersection(rhs).next().is_some()
}

/// Helper function to check whether set contains only single element.
pub(crate) fn set_contains_single_element<T: Ord + Eq + Clone>(set: &BTreeSet<T>) -> Option<T> {
    if set.len() == 1 {
        Some(set.iter().next().cloned().unwrap())
    } else {
        None
    }
}

/// Helper function to check whether two unordered lists are the same
pub(crate) fn unordered_eq<T: Ord + Eq>(lhs: &[T], rhs: &[T]) -> bool {
    if lhs.len() != rhs.len() {
        return false;
    }

    let mut lhs: Vec<_> = lhs.iter().collect();
    let mut rhs: Vec<_> = rhs.iter().collect();
    lhs.sort_unstable();
    rhs.sort_unstable();
    lhs == rhs
}
