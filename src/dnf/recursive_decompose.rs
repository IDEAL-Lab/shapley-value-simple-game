use super::{
    decompose::{decompose_inner, Decompose, SubExp},
    utils::*,
    Dnf, Var,
};
use ptree::{Style, TreeItem};
use rayon::prelude::*;
use std::{borrow::Cow, collections::BTreeSet, fmt::Display, io};

#[derive(Debug, Clone, PartialOrd, Ord)]
pub enum RecursiveDecompose<T: Var> {
    Var(T),
    And(Vec<RecursiveDecompose<T>>),
    Or(Vec<RecursiveDecompose<T>>),
    Hybrid {
        hybrid_exp: Dnf<usize>,
        sub_exps: Vec<RecursiveDecompose<T>>,
    },
}

impl<T: Var + Display> TreeItem for RecursiveDecompose<T> {
    type Child = Self;

    fn write_self<W: io::Write>(&self, f: &mut W, style: &Style) -> io::Result<()> {
        match self {
            Self::Var(owner_id) => write!(f, "{}", style.paint(owner_id)),
            Self::And(_) => write!(f, "{}", style.paint("And")),
            Self::Or(_) => write!(f, "{}", style.paint("Or")),
            Self::Hybrid { sub_exps: _, .. } => write!(f, "{}", style.paint("Hybrid")),
        }
    }

    fn children(&self) -> Cow<[Self::Child]> {
        match self {
            Self::Var(_) => Cow::from(vec![]),
            Self::And(list) | Self::Or(list) | Self::Hybrid { sub_exps: list, .. } => {
                Cow::from(list)
            }
        }
    }
}

impl<T: Var> PartialEq for RecursiveDecompose<T> {
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

impl<T: Var> Eq for RecursiveDecompose<T> {}

impl<T: Var> RecursiveDecompose<T> {
    /// Expand to DNF.
    pub fn expand(self) -> Dnf<T> {
        match self {
            Self::Var(var) => Dnf::single_variable_exp(var),
            Self::And(list) => {
                let mut ans = Dnf::true_exp();
                for sub_exp in list {
                    ans &= sub_exp.expand();
                }
                ans
            }
            Self::Or(list) => {
                let mut ans = Dnf::false_exp();
                for sub_exp in list {
                    ans |= sub_exp.expand();
                }
                ans
            }
            Self::Hybrid {
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

/// Recursively decompose a DNF.
///
/// The input requires to be already minimized. It cannot be true or false.
pub fn recursive_decompose<T: Var>(
    exp: &Dnf<T>,
    all_variables: &BTreeSet<T>,
) -> RecursiveDecompose<T> {
    debug_assert!(!exp.is_true());
    debug_assert!(!exp.is_false());

    if let Some(v) = set_contains_single_element(all_variables) {
        return RecursiveDecompose::Var(v);
    }

    recursive_decompose_inner(exp, all_variables, true)
}

fn recursive_decompose_inner<T: Var>(
    exp: &Dnf<T>,
    all_variables: &BTreeSet<T>,
    try_cc: bool,
) -> RecursiveDecompose<T> {
    let (d, modular_set_list) = decompose_inner(exp, all_variables, try_cc);

    match d {
        Decompose::Var(var) => RecursiveDecompose::Var(var),
        Decompose::And(list) => {
            let list = sub_exp_list_to_recursive_decompose_list(list, modular_set_list, true);
            RecursiveDecompose::And(list)
        }
        Decompose::Or(list) => {
            let list = sub_exp_list_to_recursive_decompose_list(list, modular_set_list, false);
            RecursiveDecompose::Or(list)
        }
        Decompose::Hybrid {
            hybrid_exp,
            sub_exps,
        } => {
            let sub_exps =
                sub_exp_list_to_recursive_decompose_list(sub_exps, modular_set_list, true);
            RecursiveDecompose::Hybrid {
                hybrid_exp,
                sub_exps,
            }
        }
    }
}

fn sub_exp_list_to_recursive_decompose_list<T: Var>(
    list: Vec<SubExp<T>>,
    modular_set_list: Vec<BTreeSet<T>>,
    try_cc_in_recursive: bool,
) -> Vec<RecursiveDecompose<T>> {
    list.into_par_iter()
        .enumerate()
        .map(|(i, sub_exp)| match sub_exp {
            SubExp::Exp(sub) => {
                let var_set = &modular_set_list[i];
                recursive_decompose_inner(&sub, var_set, try_cc_in_recursive)
            }
            SubExp::Var(var) => RecursiveDecompose::Var(var),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use ptree::print_tree;

    use super::*;
    use crate::dnf;

    #[test]
    fn test_recursive_decompose() {
        let exp = dnf!(1);
        let all_variables = exp.all_variables();
        let d = recursive_decompose(&exp, &all_variables);
        assert_eq!(d, RecursiveDecompose::Var(1));
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 + 2 + 3 + 4);
        let all_variables = exp.all_variables();
        let d = recursive_decompose(&exp, &all_variables);
        assert_eq!(
            d,
            RecursiveDecompose::Or(vec![
                RecursiveDecompose::Var(1),
                RecursiveDecompose::Var(2),
                RecursiveDecompose::Var(3),
                RecursiveDecompose::Var(4),
            ])
        );
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 3 + 2 3 + 1 4 + 2 4);
        let all_variables = exp.all_variables();
        let d = recursive_decompose(&exp, &all_variables);
        assert_eq!(
            d,
            RecursiveDecompose::And(vec![
                RecursiveDecompose::Or(vec![
                    RecursiveDecompose::Var(1),
                    RecursiveDecompose::Var(2),
                ]),
                RecursiveDecompose::Or(vec![
                    RecursiveDecompose::Var(3),
                    RecursiveDecompose::Var(4),
                ]),
            ])
        );
        assert_eq!(exp, d.expand());

        let exp = dnf!(1 2 4 + 1 3 4 + 2 3 4 + 1 2 5 6 + 1 3 5 6 + 2 3 5 6 + 4 5 6 + 1 2 7 + 1 3 7 + 2 3 7 + 4 7);
        let all_variables = exp.all_variables();
        let d = recursive_decompose(&exp, &all_variables);
        assert_eq!(
            d,
            RecursiveDecompose::Hybrid {
                hybrid_exp: dnf!(0 1 + 0 2 + 1 2),
                sub_exps: vec![
                    RecursiveDecompose::Hybrid {
                        hybrid_exp: dnf!(0 1 + 0 2 + 1 2),
                        sub_exps: vec![
                            RecursiveDecompose::Var(1),
                            RecursiveDecompose::Var(2),
                            RecursiveDecompose::Var(3),
                        ]
                    },
                    RecursiveDecompose::Var(4),
                    RecursiveDecompose::Or(vec![
                        RecursiveDecompose::And(vec![
                            RecursiveDecompose::Var(5),
                            RecursiveDecompose::Var(6),
                        ]),
                        RecursiveDecompose::Var(7),
                    ]),
                ]
            }
        );
        assert_eq!(exp, d.expand());
    }

    #[test]
    fn build_tree() {
        let exp = dnf!(1 2 4 + 1 3 4 + 2 3 4 + 1 2 5 6 + 1 3 5 6 + 2 3 5 6 + 4 5 6 + 1 2 7 + 1 3 7 + 2 3 7 + 4 7);
        let all_variables = exp.all_variables();
        let d = recursive_decompose(&exp, &all_variables);
        print_tree(&d).ok();
    }
}
