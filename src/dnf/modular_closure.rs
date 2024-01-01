use super::{utils::*, Dnf, Var};
use rayon::prelude::*;
use std::{cmp::Ordering, collections::BTreeSet};

type SList<T> = Vec<(BTreeSet<T>, BTreeSet<T>)>;

/// Compute the modular closure from an candidate set.
///
/// The input exp is required to be already minimized.
pub fn compute_modular_closure<T: Var>(exp: &Dnf<T>, mut seed: BTreeSet<T>) -> BTreeSet<T> {
    while let Some(new_seed) = solve_pmodular(exp, &seed) {
        debug_assert!(seed != new_seed, "infinite loop detected");
        seed = new_seed;
    }
    seed
}

/// Solve the PMODULAR problem (pp. 27)
///
/// The input exp is required to be already minimized.
/// Return None if maybe_modular is a modular set
/// otherwise return maybe_modular + x, where x \in Closure(maybe_modular) \ maybe_modular
fn solve_pmodular<T: Var>(exp: &Dnf<T>, maybe_modular: &BTreeSet<T>) -> Option<BTreeSet<T>> {
    let partial_exp = exp.partial_exp(maybe_modular);
    if partial_exp.len() < 2 {
        return None;
    }

    let mut list: SList<T> = partial_exp
        .par_iter()
        .map(|t| {
            (
                t.intersection(maybe_modular).cloned().collect(),
                t.difference(maybe_modular).cloned().collect(),
            )
        })
        .collect();
    list.sort_unstable();
    let culprit = find_culprit(&list)?;

    // Ref: theorem 28 (pp. 26)
    let mut modular = partial_exp
        .into_par_iter()
        .map(|mut u| {
            for t in culprit.0 {
                u.remove(t);
            }
            for t in culprit.1 {
                u.remove(t);
            }
            u.0
        })
        .filter(|ut| !has_intersection(ut, maybe_modular))
        .min_by_key(|ut| ut.len())
        .expect("this should always return u0t.");

    modular.extend(maybe_modular.iter().cloned());
    Some(modular)
}

/// Find the culprit from list S.
///
/// Ref: pp. 30--31
fn find_culprit<T: Var>(list: &SList<T>) -> Option<(&BTreeSet<T>, &BTreeSet<T>)> {
    let segment_len = {
        let first_tuple = &list[0];
        list.iter()
            .skip(1)
            .take_while(|x| x.0 == first_tuple.0)
            .count()
            + 1
    };

    let mut i = segment_len;
    while i < list.len() {
        // the last segment is longer than the first segment
        if list[i - 1].0 == list[i].0 {
            return Some(corollary_7(list, 0, i));
        }

        let mut j = 0;

        while j < segment_len {
            if i + j >= list.len() || list[i].0 != list[i + j].0 {
                return Some(corollary_7(list, i, j));
            }

            match list[j].1.cmp(&list[i + j].1) {
                Ordering::Less => {
                    return Some(corollary_7(list, i, j));
                }
                Ordering::Greater => {
                    return Some(corollary_7(list, 0, i + j));
                }
                Ordering::Equal => {}
            }

            j += 1;
        }

        i += j;
    }

    None
}

/// Find culprit using the missing tuple.
///
/// Ref: pp. 29
fn corollary_7<T: Var>(
    list: &SList<T>,
    missing_s_index: usize,
    missing_t_index: usize,
) -> (&BTreeSet<T>, &BTreeSet<T>) {
    let missing_s = &list[missing_s_index].0;
    let missing_t = &list[missing_t_index].1;

    list.par_iter()
        .enumerate()
        .find_map_any(|(i, (s, t))| {
            if i != missing_s_index && s != missing_s {
                if s.is_subset(missing_s) {
                    return Some((s, &list[missing_s_index].1));
                } else if missing_s.is_subset(s) {
                    return Some((missing_s, t));
                }
            }

            if i != missing_t_index && t != missing_t {
                if t.is_subset(missing_t) {
                    return Some((&list[missing_t_index].0, t));
                } else if missing_t.is_subset(t) {
                    return Some((s, missing_t));
                }
            }

            None
        })
        .unwrap_or((missing_s, missing_t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dnf;

    #[test]
    fn test_find_culprit() {
        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1, 2].into(), [4, 5].into()),
            ([1, 2].into(), [6].into()),
            ([2, 3].into(), [4, 5].into()),
            ([2, 3].into(), [6].into()),
        ];
        assert_eq!(find_culprit(&list), None);

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [5].into()),
            ([1].into(), [6].into()),
            ([2, 4].into(), [5].into()),
            ([3].into(), [5].into()),
            ([3].into(), [6].into()),
            ([4].into(), [6].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[4].into(), &[5].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [5].into()),
            ([1, 4].into(), [2].into()),
            ([3, 4].into(), [2].into()),
            ([3, 4].into(), [5].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[1].into(), &[2].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [3].into()),
            ([1].into(), [4].into()),
            ([2].into(), [3].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[2].into(), &[4].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [2].into()),
            ([1].into(), [3].into()),
            ([1].into(), [4].into()),
        ];
        assert_eq!(find_culprit(&list), None);

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [4].into()),
            ([2].into(), [4].into()),
            ([3].into(), [4].into()),
        ];
        assert_eq!(find_culprit(&list), None);

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [4].into()),
            ([2].into(), [5].into()),
            ([3].into(), [4].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[2].into(), &[4].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1, 2].into(), [4, 5].into()),
            ([1, 2].into(), [6].into()),
            ([2, 3].into(), [4, 5].into()),
            ([2, 3].into(), [6].into()),
            ([2, 3].into(), [7].into()),
            ([8].into(), [4, 5].into()),
            ([8].into(), [6].into()),
            ([8].into(), [7].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[1, 2].into(), &[7].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [4].into()),
            ([1].into(), [5].into()),
            ([2].into(), [3].into()),
            ([2].into(), [4].into()),
            ([2].into(), [5].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[1].into(), &[3].into())));

        let list: Vec<(BTreeSet<i32>, BTreeSet<i32>)> = vec![
            ([1].into(), [3].into()),
            ([1].into(), [5].into()),
            ([2].into(), [3].into()),
            ([2].into(), [4].into()),
            ([2].into(), [5].into()),
        ];
        assert_eq!(find_culprit(&list), Some((&[1].into(), &[4].into())));
    }

    #[test]
    fn test_solve_pmodular() {
        let exp = dnf!(1 2 4 5 + 1 2 6 + 2 3 4 5 + 2 3 6 + 4 6);
        assert_eq!(solve_pmodular(&exp, &([1, 2, 3].into())), None);

        let exp = dnf!(1 2 + 1 3 + 2 3) & dnf!(4 + 5) & dnf!(6);
        assert_eq!(
            solve_pmodular(&exp, &([1, 2].into())),
            Some([1, 2, 3].into())
        );

        let exp = dnf!(1 2 + 1 3 + 2 3) & dnf!(4 + 5) & dnf!(6);
        assert_eq!(solve_pmodular(&exp, &([1, 2, 3].into())), None);
        assert_eq!(solve_pmodular(&exp, &([1, 2, 3, 4, 5].into())), None);
    }

    #[test]
    fn test_compute_modular_closure() {
        let exp = dnf!(1 2 + 1 3 + 2 3) & dnf!(4 + 5) & dnf!(6);
        assert_eq!(compute_modular_closure(&exp, [1].into()), [1].into());
        assert_eq!(
            compute_modular_closure(&exp, [1, 2].into()),
            [1, 2, 3].into()
        );
        assert_eq!(
            compute_modular_closure(&exp, [1, 4].into()),
            [1, 2, 3, 4, 5].into()
        );
    }
}
