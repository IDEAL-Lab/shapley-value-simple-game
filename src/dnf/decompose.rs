use super::{modular_closure::compute_modular_closure, unionfind::UnionFind, utils::*, Dnf, Var};
use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};

/// Sub-expression for the decomposition result
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SubExp<T: Var> {
    Exp(Dnf<T>),
    Var(T),
}

impl<T: Var> SubExp<T> {
    /// Expand to DNF. Only used in test
    #[cfg(test)]
    pub fn expand(self) -> Dnf<T> {
        match self {
            SubExp::Exp(exp) => exp,
            SubExp::Var(var) => Dnf::single_variable_exp(var),
        }
    }
}

/// The result of the decomposition
#[derive(Debug, Clone, PartialOrd, Ord)]
pub enum Decompose<T: Var> {
    Var(T),
    And(Vec<SubExp<T>>),
    Or(Vec<SubExp<T>>),
    Hybrid {
        hybrid_exp: Dnf<usize>,
        sub_exps: Vec<SubExp<T>>,
    },
}

impl<T: Var> PartialEq for Decompose<T> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Var(l0), Self::Var(r0)) => l0 == r0,
            (Self::And(l0), Self::And(r0)) => unordered_eq(l0, r0),
            (Self::Or(l0), Self::Or(r0)) => unordered_eq(l0, r0),
            (
                Self::Hybrid {
                    hybrid_exp: l_exp,
                    sub_exps: l_sub_exps,
                },
                Self::Hybrid {
                    hybrid_exp: r_exp,
                    sub_exps: r_sub_exps,
                },
            ) => {
                if l_exp != r_exp {
                    return false;
                }

                l_sub_exps == r_sub_exps
            }
            _ => false,
        }
    }
}

impl<T: Var> Eq for Decompose<T> {}

impl<T: Var> Decompose<T> {
    /// Expand to DNF. Only used in test
    #[cfg(test)]
    pub fn expand(self) -> Dnf<T> {
        match self {
            Decompose::Var(var) => Dnf::single_variable_exp(var),
            Decompose::And(list) => {
                let mut ans = Dnf::true_exp();
                for sub_exp in list {
                    ans &= sub_exp.expand();
                }
                ans
            }
            Decompose::Or(list) => {
                let mut ans = Dnf::false_exp();
                for sub_exp in list {
                    ans |= sub_exp.expand();
                }
                ans
            }
            Decompose::Hybrid {
                hybrid_exp,
                sub_exps,
            } => {
                let sub_exps: Vec<_> = sub_exps.into_iter().map(|e| e.expand()).collect();
                let mut ans = Dnf::false_exp();
                for t in hybrid_exp {
                    let mut expand_t = Dnf::true_exp();
                    for i in t {
                        expand_t &= &sub_exps[i];
                    }
                    ans |= expand_t;
                }
                ans
            }
        }
    }
}

/// Decompose a DNF
///
/// The input requires to be already minimized. It cannot be true or false.
pub fn decompose<T: Var>(exp: &Dnf<T>, all_variables: &BTreeSet<T>) -> Decompose<T> {
    debug_assert!(!exp.is_true());
    debug_assert!(!exp.is_false());

    if let Some(v) = set_contains_single_element(all_variables) {
        return Decompose::Var(v);
    }

    decompose_inner(exp, all_variables, true).0
}

pub(crate) fn decompose_inner<T: Var>(
    exp: &Dnf<T>,
    all_variables: &BTreeSet<T>,
    try_cc: bool,
) -> (Decompose<T>, Vec<BTreeSet<T>>) {
    if try_cc {
        if let Some(ans) = decompose_using_cc(exp) {
            return ans;
        }
    }

    let (modular_set_list, is_prime) = compute_all_disjoint_modular_set(exp, all_variables);

    let ans = if is_prime && modular_set_list.len() > 2 {
        let hybrid_exp = {
            #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
            enum VarOrSubExpId<T: Var> {
                Var(T),
                SubExp(usize),
            }

            let mut exp2 = exp.map_variable(|var| VarOrSubExpId::Var(var.clone()));
            for (i, set) in modular_set_list.iter().enumerate() {
                let set: BTreeSet<VarOrSubExpId<T>> = set
                    .iter()
                    .map(|var| VarOrSubExpId::Var(var.clone()))
                    .collect();
                // Ref: Eq. 19 (pp. 21)
                let exp3 = (Dnf::single_variable_exp(VarOrSubExpId::SubExp(i))
                    & exp2.partial_exp(&set).partial_eval(&set, true))
                    | exp2.partial_exp_complement(&set);
                exp2 = exp3;
            }

            exp2.map_variable(|var| match var {
                VarOrSubExpId::SubExp(id) => *id,
                VarOrSubExpId::Var(_) => unreachable!(),
            })
        };

        let sub_exps: Vec<_> = modular_set_list
            .par_iter()
            .map(|s| {
                if let Some(v) = set_contains_single_element(s) {
                    SubExp::Var(v)
                } else {
                    SubExp::Exp(exp.contraction_exp(s))
                }
            })
            .collect();

        Decompose::Hybrid {
            hybrid_exp,
            sub_exps,
        }
    } else {
        let mut is_and = false;
        for t in exp.iter() {
            if has_intersection(&t.0, &modular_set_list[0]) {
                if has_intersection(&t.0, &modular_set_list[1]) {
                    is_and = true;
                }
                break;
            }
        }

        let sub_exps: Vec<_> = modular_set_list
            .par_iter()
            .map(|s| {
                if let Some(v) = set_contains_single_element(s) {
                    SubExp::Var(v)
                } else {
                    SubExp::Exp(exp.contraction_exp(s))
                }
            })
            .collect();

        if is_and {
            Decompose::And(sub_exps)
        } else {
            Decompose::Or(sub_exps)
        }
    };

    (ans, modular_set_list)
}

/// Compute maximal modular set from a starting set.
///
/// Ref: lemma 8 (pp. 32)
fn compute_maximal_modular_set<T: Var>(
    exp: &Dnf<T>,
    seed: BTreeSet<T>,
    all_variables: &BTreeSet<T>,
    skip_variables: Option<&BTreeSet<T>>,
) -> BTreeSet<T> {
    let mut variable_set: BTreeSet<T> = if let Some(skip) = skip_variables {
        let mut set: BTreeSet<T> = all_variables.difference(skip).cloned().collect();
        for s in &seed {
            set.remove(s);
        }
        set
    } else {
        all_variables.difference(&seed).cloned().collect()
    };

    let mut ans = seed;

    while let Some(var) = pop_first(&mut variable_set) {
        let mut closure = ans.clone();
        closure.insert(var);
        closure = compute_modular_closure(exp, closure);

        if closure.len() != all_variables.len() {
            ans = closure;
        }
    }

    ans
}

/// Compute all disjoint modular sets.
/// Return (modular_set_list, is_prime)
///
/// Ref: proposition 7 (pp. 32)
fn compute_all_disjoint_modular_set<T: Var>(
    exp: &Dnf<T>,
    all_variables: &BTreeSet<T>,
) -> (Vec<BTreeSet<T>>, bool) {
    let c1 = {
        let start_var = all_variables
            .iter()
            .next()
            .cloned()
            .expect("the input exp should contain more than one variable.");
        compute_maximal_modular_set(exp, [start_var].into(), all_variables, None)
    };
    let c2 = {
        let start_var = all_variables
            .difference(&c1)
            .next()
            .cloned()
            .expect("the input exp should contain more than one variable.");
        compute_maximal_modular_set(exp, [start_var].into(), all_variables, None)
    };

    let mut ans = vec![c1, c2];
    if !has_intersection(&ans[0], &ans[1]) {
        let mut union: BTreeSet<T> = ans[0].union(&ans[1]).cloned().collect();
        while let Some(start_var) = all_variables.difference(&union).next().cloned() {
            let c =
                compute_maximal_modular_set(exp, [start_var].into(), all_variables, Some(&union));
            union.extend(c.iter().cloned());
            ans.push(c);
        }
        (ans, true)
    } else {
        let mut intersection: BTreeSet<T> = ans[0].intersection(&ans[1]).cloned().collect();
        while !intersection.is_empty() {
            let seed: BTreeSet<T> = all_variables.difference(&intersection).cloned().collect();
            let c = compute_maximal_modular_set(exp, seed, all_variables, None);
            intersection = intersection.intersection(&c).cloned().collect();
            ans.push(c);
        }
        let ans = ans
            .par_iter()
            .map(|s| all_variables.difference(s).cloned().collect())
            .collect();
        (ans, false)
    }
}

fn decompose_using_cc<T: Var>(exp: &Dnf<T>) -> Option<(Decompose<T>, Vec<BTreeSet<T>>)> {
    let imps: Vec<_> = exp.iter().collect();
    let mut union = UnionFind::new(imps.len());
    for (i, t_i) in imps.iter().enumerate() {
        for (j, t_j) in imps.iter().enumerate().skip(i + 1) {
            if !union.equiv(i, j) && has_intersection(&t_i.0, &t_j.0) {
                union.union(i, j);
            }
        }
    }
    let labels = union.into_labeling();
    let mut label_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, l) in labels.into_iter().enumerate() {
        label_map.entry(l).or_default().push(i);
    }

    if label_map.len() == 1 {
        None
    } else {
        let (ans_exp, ans_set) = label_map
            .into_par_iter()
            .map(|(_, list)| {
                if list.len() == 1 {
                    let t = imps[list[0]].clone();
                    if t.len() == 1 {
                        let v = t.into_iter().next().unwrap();
                        (SubExp::Var(v.clone()), BTreeSet::from([v]))
                    } else {
                        let s = t.0.clone();
                        let e = Dnf::from([t]);
                        (SubExp::Exp(e), s)
                    }
                } else {
                    let mut e = Dnf::new();
                    for i in list {
                        e.insert(imps[i].clone());
                    }
                    let s = e.all_variables();
                    (SubExp::Exp(e), s)
                }
            })
            .unzip();
        Some((Decompose::Or(ans_exp), ans_set))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dnf;
    use proptest::prelude::*;

    #[test]
    fn test_compute_all_disjoint_modular_set() {
        let exp = dnf!(1 + 2 + 3 + 4);
        let (mut modular_set_list, is_prime) =
            compute_all_disjoint_modular_set(&exp, &exp.all_variables());
        modular_set_list.sort_unstable();
        assert_eq!(
            modular_set_list,
            vec![[1].into(), [2].into(), [3].into(), [4].into()]
        );
        assert!(!is_prime);

        let exp = dnf!(1 2 4 + 1 3 4 + 2 3 4 + 1 2 5 6 + 1 3 5 6 + 2 3 5 6 + 4 5 6 + 1 2 7 + 1 3 7 + 2 3 7 + 4 7);
        let (mut modular_set_list, is_prime) =
            compute_all_disjoint_modular_set(&exp, &exp.all_variables());
        modular_set_list.sort_unstable();
        assert_eq!(
            modular_set_list,
            vec![[1, 2, 3].into(), [4].into(), [5, 6, 7].into()]
        );
        assert!(is_prime);
    }

    #[test]
    fn test_decompose_cc() {
        let exp = dnf!(1 3 + 2 3 + 1 4 + 2 4);
        let d = decompose_using_cc(&exp);
        assert_eq!(d, None);

        let exp = dnf!(1 3 + 1 3 + 2 3 + 4 5 + 6 + 7);
        let mut d = decompose_using_cc(&exp).unwrap();
        d.1.sort_unstable();
        assert_eq!(
            d.1,
            vec![[1, 2, 3].into(), [4, 5].into(), [6].into(), [7].into()]
        );
    }

    #[test]
    fn test_decompose() {
        let exp = dnf!(1);
        let all_variables = exp.all_variables();
        let d = decompose(&exp, &all_variables);
        assert_eq!(d, Decompose::Var(1));
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 + 2 + 3 + 4);
        let all_variables = exp.all_variables();
        let d = decompose(&exp, &all_variables);
        assert_eq!(
            d,
            Decompose::Or(vec![
                SubExp::Var(1),
                SubExp::Var(2),
                SubExp::Var(3),
                SubExp::Var(4),
            ])
        );
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 3 + 2 3 + 1 4 + 2 4);
        let all_variables = exp.all_variables();
        let d = decompose(&exp, &all_variables);
        assert_eq!(
            d,
            Decompose::And(vec![SubExp::Exp(dnf!(1 + 2)), SubExp::Exp(dnf!(3 + 4)),])
        );
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 2 4 + 1 3 4 + 2 3 4 + 1 2 5 6 + 1 3 5 6 + 2 3 5 6 + 4 5 6 + 1 2 7 + 1 3 7 + 2 3 7 + 4 7);
        let all_variables = exp.all_variables();
        let d = decompose(&exp, &all_variables);
        assert_eq!(
            d,
            Decompose::Hybrid {
                hybrid_exp: dnf!(0 1 + 0 2 + 1 2),
                sub_exps: vec![
                    SubExp::Exp(dnf!(1 2 + 2 3 + 1 3)),
                    SubExp::Var(4),
                    SubExp::Exp(dnf!(5 6 + 7)),
                ]
            }
        );
        assert_eq!(exp, d.expand());
    }

    #[derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        derive_more::Display,
        proptest_derive::Arbitrary,
    )]
    #[rustfmt::skip]
    enum Element {
        E1, E2, E3, E4, E5, E6, E7, E8, E9, E10, E11, E12, E13, E14, E15, E16,
        E17, E18, E19, E20, E21, E22, E23, E24, E25, E26, E27, E28, E29, E30, E31, E32,
    }

    proptest! {
        #[test]
        #[ignore = "fuzzy test is not run by default"]
        fn test_decompose_fuzzy(mut exp in any::<Dnf<Element>>()) {
            exp.minimize();
            prop_assume!(!exp.is_true());
            prop_assume!(!exp.is_false());
            let all_variables = exp.all_variables();
            let d = decompose(&exp, &all_variables);
            assert_eq!(exp, d.expand());
        }
    }
}
