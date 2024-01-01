use super::*;
use crate::{
    dnf::{Dnf, Implicant},
    union_combination::*,
};
use bit_set::BitSet;
use rayon::prelude::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct HybridCoeffs {
    input_len: usize,
    /// A map of element set to their coeffs without sign information.
    ///
    /// e.g., if the input is a hybrid `a b + a c + b c`, its input is noted:
    /// `0: [a b], 1: [a c], 2: [b c]
    /// It contains mapping from all subset of { a, b, c } to their coeffs.
    coeffs_map: HashMap<BitSet, IECoeffs>,
}

impl HybridCoeffs {
    pub fn new(input: &[IECoeffs]) -> Self {
        let len = input.len();
        match len {
            0 => unreachable!(),
            1 => {
                let coeffs = input[0].clone();
                let input_set = BitSet::from_iter([0]);
                let coeffs_map = HashMap::from([(input_set, coeffs)]);
                return Self {
                    input_len: len,
                    coeffs_map,
                };
            }
            _ => {}
        }

        #[derive(Debug, Clone)]
        struct UnionData {
            input_set: BitSet,
            coeffs: IECoeffs,
        }

        let unions: UnionCombination<UnionData> = UnionCombination::new(
            len,
            |i| {
                let coeffs = input[i].clone();
                let mut input_set = BitSet::with_capacity(len);
                input_set.insert(i);
                UnionData { input_set, coeffs }
            },
            |old, i| {
                let mut new_set = old.input_set.clone();
                new_set.insert(i);
                let coeffs = vertical_op(&old.coeffs, &input[i]);
                Some(UnionData {
                    input_set: new_set,
                    coeffs,
                })
            },
        );

        let mut coeffs_map = HashMap::with_capacity(unions.len());

        for u in unions.0 {
            let u = u.into_inner();
            coeffs_map.insert(u.input_set, u.coeffs);
        }

        Self {
            input_len: len,
            coeffs_map,
        }
    }

    pub fn exp_coeffs(&self, exp: &Dnf<usize>) -> IECoeffs {
        match exp.len() {
            0 => unreachable!(),
            1 => {
                let imp = exp.iter().next().unwrap();
                let input_set = imp_to_bitset(imp, self.input_len);
                let coeffs = self.coeffs_map[&input_set].clone();
                return coeffs;
            }
            _ => {}
        }

        let unions = exp_to_input_unions(exp);
        self.exp_unions_coeffs(&unions)
    }

    pub fn exp_unions_coeffs(&self, exp_unions: &UnionCombination<ExpInputUnion>) -> IECoeffs {
        exp_unions
            .0
            .par_iter()
            .map(|u| {
                let u = u.get();
                let sign = if u.num_of_imp % 2 == 0 { -1 } else { 1 };
                let mut coeffs = self.coeffs_map[&u.input_set].clone();
                coeffs.apply_sign(sign);
                coeffs
            })
            .sum()
    }

    #[cfg(test)]
    pub fn interaction(&self, exp1: &Dnf<usize>, exp2: &Dnf<usize>) -> IECoeffs {
        let unions1 = exp_to_input_unions(exp1);
        let unions2 = exp_to_input_unions(exp2);
        self.exp_unions_interaction(&unions1, &unions2)
    }

    pub fn exp_unions_interaction(
        &self,
        exp_unions1: &UnionCombination<ExpInputUnion>,
        exp_unions2: &UnionCombination<ExpInputUnion>,
    ) -> IECoeffs {
        exp_unions1
            .0
            .par_iter()
            .flat_map(|u1| {
                let u1 = u1.get();
                exp_unions2.0.par_iter().map(move |u2| {
                    let u2 = u2.get();
                    let input_set = u1.input_set.union(&u2.input_set).collect();
                    let mut coeffs = self.coeffs_map[&input_set].clone();
                    let sign = if (u1.num_of_imp + u2.num_of_imp) % 2 == 0 {
                        1
                    } else {
                        -1
                    };
                    coeffs.apply_sign(sign);
                    coeffs
                })
            })
            .sum()
    }
}

#[inline]
fn imp_to_bitset(imp: &Implicant<usize>, cap: usize) -> BitSet {
    let mut ans = BitSet::with_capacity(cap);
    for i in imp.iter() {
        ans.insert(*i);
    }
    ans
}

#[derive(Debug, Clone)]
pub struct ExpInputUnion {
    num_of_imp: usize,
    input_set: BitSet,
}

pub fn exp_to_input_unions(exp: &Dnf<usize>) -> UnionCombination<ExpInputUnion> {
    let var_len = exp.all_variables().len();
    let imp_list: Vec<_> = exp.iter().collect();
    UnionCombination::new(
        imp_list.len(),
        |i| {
            let imp = imp_list[i];
            let input_set = imp_to_bitset(imp, var_len);
            ExpInputUnion {
                num_of_imp: 1,
                input_set,
            }
        },
        |old, i| {
            let new_imp = imp_list[i];
            let mut new_set = old.input_set.clone();
            new_set.extend(new_imp.iter().copied());
            // whether new set is full and cur_id != MAX_ID
            if new_set.len() == var_len && i != imp_list.len() - 1 {
                None
            } else {
                Some(ExpInputUnion {
                    num_of_imp: old.num_of_imp + 1,
                    input_set: new_set,
                })
            }
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{dnf, ie_coeffs};

    #[test]
    fn test_coeffs1() {
        let hybrid_exp = dnf!(0 1);
        let input = vec![
            ie_coeffs! { 1 => 1, 2 => 1, 3 => -1},
            ie_coeffs! { 1 => 2, 3 => -2, 4 => 1},
        ];

        let hybrid_coeffs = HybridCoeffs::new(&input);
        let actual = hybrid_coeffs.exp_coeffs(&hybrid_exp);
        let expect = ie_coeffs! { 2 => 2, 3 => 2, 4 => -4, 5 => -1, 6 => 3, 7 => -1 };
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_coeffs2() {
        let hybrid_exp = dnf!(0 1 + 0 2 + 1 2);
        let input = vec![
            ie_coeffs! { 1 => 1 },
            ie_coeffs! { 1 => 1 },
            ie_coeffs! { 1 => 3, 2 => -3, 3 => 1 },
        ];

        let hybrid_coeffs = HybridCoeffs::new(&input);
        let actual = hybrid_coeffs.exp_coeffs(&hybrid_exp);
        let expect = ie_coeffs! { 2 => 7, 3 => -12, 4 => 8, 5 => -2 };
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_coeffs3() {
        let exp = dnf!(0 1);
        let input = vec![
            ie_coeffs! { 1 => 1, 2 => 1, 3 => -1 },
            ie_coeffs! { 1 => 2, 3 => -2, 4 => 1 },
            ie_coeffs! { 1 => 3, 2 => -3, 3 => 1 },
        ];

        let hybrid_coeffs = HybridCoeffs::new(&input);
        let actual = hybrid_coeffs.exp_coeffs(&exp);
        let expect = ie_coeffs! { 2 => 2, 3 => 2, 4 => -4, 5 => -1, 6 => 3, 7 => -1 };
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_interaction1() {
        let exp1 = dnf!(0 + 2);
        let exp2 = dnf!(0 2);
        let input = vec![
            ie_coeffs! { 1 => 1 },
            ie_coeffs! { 1 => 1 },
            ie_coeffs! { 1 => 3, 2 => -3, 3 => 1 },
        ];

        let hybrid_coeffs = HybridCoeffs::new(&input);
        let actual = hybrid_coeffs.interaction(&exp1, &exp2);
        let expect = ie_coeffs! { 2 => 3, 3 => -3, 4 => 1 };
        assert_eq!(expect, actual);
    }

    #[test]
    fn test_interaction2() {
        let exp1 = dnf!(0 + 2);
        let exp2 = dnf!(2 3 + 3 4 + 4 5);
        let input = vec![
            ie_coeffs! { 1 => 1 },
            ie_coeffs! { 1 => 2, 2 => -1 },
            ie_coeffs! { 1 => 2, 2 => -1 },
            ie_coeffs! { 1 => 3, 2 => -3, 3 => 1 },
            ie_coeffs! { 2 => 1 },
            ie_coeffs! { 3 => 2, 5 => -1 },
        ];

        let hybrid_coeffs = HybridCoeffs::new(&input);
        let actual = hybrid_coeffs.interaction(&exp1, &exp2);
        let expect = ie_coeffs! { 2 => 6, 3 => -9, 4 => 8, 5 => -10, 6 => 16, 7 => -29, 8 => 36, 9 => -18, 10 => -7, 11 => 13, 12 => -6, 13 => 1 };
        assert_eq!(expect, actual);
    }
}
