use super::{utils::*, Implicant, Var};
use rayon::prelude::*;
use std::{
    collections::BTreeSet,
    fmt, mem,
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign},
};

/// A boolean expression in DNF.
///
/// If the underlying set is empty, the expression is considered as FALSE.
/// If it contains a empty implicant, the expression is considered as TRUE.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
)]
#[cfg_attr(test, derive(proptest_derive::Arbitrary))]
#[as_ref(forward)]
#[as_mut(forward)]
#[from(forward)]
pub struct Dnf<T: Var>(pub BTreeSet<Implicant<T>>);

impl<T: Var> Dnf<T> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn true_exp() -> Self {
        Self::from([Implicant::new()])
    }

    pub fn false_exp() -> Self {
        Self::new()
    }

    pub fn single_variable_exp(var: T) -> Self {
        Self::from([Implicant::from([var])])
    }

    pub fn is_true(&self) -> bool {
        self.contains(&Implicant::new())
    }

    pub fn is_false(&self) -> bool {
        self.is_empty()
    }

    /// Return a set of all variables.
    pub fn all_variables(&self) -> BTreeSet<T> {
        let mut ans = BTreeSet::new();
        for t in self.iter() {
            ans.extend(t.iter().cloned());
        }
        ans
    }

    /// Remove non-hybrid implicants in DNF.
    pub fn minimize(&mut self) {
        let mut skips = vec![false; self.len()];
        for (i, term_i) in self.iter().enumerate() {
            if skips[i] {
                continue;
            }

            for (j, term_j) in self.iter().enumerate().skip(i + 1) {
                if skips[j] {
                    continue;
                }

                if term_i.is_subset(term_j) {
                    skips[j] = true;
                }
            }
        }

        let original = mem::take(&mut self.0);
        let ans: Dnf<T> = original
            .into_iter()
            .enumerate()
            .filter_map(|(i, t)| if skips[i] { None } else { Some(t) })
            .collect();
        *self = ans;
    }

    /// Eval to TRUE or FALSE.
    pub fn eval(&self, input_set: &BTreeSet<T>, input_is_true: bool) -> bool {
        self.par_iter().any(|t| t.eval(input_set, input_is_true))
    }

    /// Partially eval the expression with variables in `input_set` set to be `input_is_true`.
    pub fn partial_eval(&self, input_set: &BTreeSet<T>, input_is_true: bool) -> Dnf<T> {
        let ans: BTreeSet<_> = self
            .par_iter()
            .filter_map(|t| t.partial_eval(input_set, input_is_true))
            .collect();
        let mut ans = Dnf::from(ans);
        ans.minimize();
        ans
    }

    /// Partially eval the expression with variables not in `input_set` set to be `complement_is_true`.
    pub fn partial_eval_complement(
        &self,
        input_set: &BTreeSet<T>,
        complement_is_true: bool,
    ) -> Dnf<T> {
        let ans: BTreeSet<_> = self
            .par_iter()
            .filter_map(|t| t.partial_eval_complement(input_set, complement_is_true))
            .collect();
        let mut ans = Dnf::from(ans);
        ans.minimize();
        ans
    }

    /// A DNF with implicants who have intersection with input_set, i.e., f^a in Def. 7 (pp. 20).
    pub fn partial_exp(&self, input_set: &BTreeSet<T>) -> Dnf<T> {
        let ans: BTreeSet<_> = self
            .par_iter()
            .filter(|t| has_intersection(&t.0, input_set))
            .cloned()
            .collect();
        Dnf::from(ans)
    }

    /// A DNF with implicants who do not have intersection with input_set. Equivalent to `partial_eval(input_set, false)`
    pub fn partial_exp_complement(&self, input_set: &BTreeSet<T>) -> Dnf<T> {
        let ans: BTreeSet<_> = self
            .par_iter()
            .filter(|t| !has_intersection(&t.0, input_set))
            .cloned()
            .collect();
        Dnf::from(ans)
    }

    /// f_a = f^a(\hat(a) = 1). See Def. 8 (pp. 20).
    pub fn contraction_exp(&self, input_set: &BTreeSet<T>) -> Dnf<T> {
        let partial_exp = self.partial_exp(input_set);
        partial_exp.partial_eval_complement(input_set, true)
    }

    /// Apply f to every variable in the DNF.
    pub fn map_variable<U: Var>(&self, f: impl Fn(&T) -> U) -> Dnf<U> {
        self.iter().map(|t| t.map_variable(&f)).collect()
    }
}

impl<T: Var> Default for Dnf<T> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T: Var> FromIterator<Implicant<T>> for Dnf<T> {
    fn from_iter<I: IntoIterator<Item = Implicant<T>>>(iter: I) -> Self {
        Self(BTreeSet::from_iter(iter))
    }
}

impl<T: Var> IntoIterator for Dnf<T> {
    type Item = <BTreeSet<Implicant<T>> as IntoIterator>::Item;
    type IntoIter = <BTreeSet<Implicant<T>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Var> IntoParallelIterator for Dnf<T> {
    type Iter = <BTreeSet<Implicant<T>> as IntoParallelIterator>::Iter;
    type Item = <BTreeSet<Implicant<T>> as IntoParallelIterator>::Item;

    fn into_par_iter(self) -> Self::Iter {
        self.0.into_par_iter()
    }
}

impl<T: Var + fmt::Display> fmt::Display for Dnf<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "FALSE");
        }

        for (i, t) in self.iter().enumerate() {
            if i != 0 {
                write!(f, " + ")?;
            }
            write!(f, "{t}")?;
        }
        Ok(())
    }
}

impl<T: Var> BitAnd for Dnf<T> {
    type Output = Dnf<T>;

    fn bitand(self, rhs: Dnf<T>) -> Self::Output {
        (&self) & (&rhs)
    }
}

impl<'a, 'b, T: Var> BitAnd<&'b Dnf<T>> for &'a Dnf<T> {
    type Output = Dnf<T>;

    fn bitand(self, rhs: &'b Dnf<T>) -> Self::Output {
        let ans: BTreeSet<_> = self
            .par_iter()
            .flat_map(|lhs_t| rhs.par_iter().map(move |rhs_t| lhs_t & rhs_t))
            .collect();
        let mut ans = Dnf::from(ans);
        ans.minimize();
        ans
    }
}

impl<T: Var> BitAndAssign for Dnf<T> {
    fn bitand_assign(&mut self, rhs: Dnf<T>) {
        *self = (&*self) & (&rhs);
    }
}

impl<'a, T: Var> BitAndAssign<&'a Dnf<T>> for Dnf<T> {
    fn bitand_assign(&mut self, rhs: &'a Dnf<T>) {
        *self = (&*self) & rhs;
    }
}

impl<T: Var> BitOr for Dnf<T> {
    type Output = Dnf<T>;

    fn bitor(self, rhs: Dnf<T>) -> Self::Output {
        let (mut to_mutate, mut to_consume) = if self.len() < rhs.len() {
            (rhs, self)
        } else {
            (self, rhs)
        };
        to_mutate.append(&mut to_consume);
        to_mutate.minimize();
        to_mutate
    }
}

impl<'a, 'b, T: Var> BitOr<&'b Dnf<T>> for &'a Dnf<T> {
    type Output = Dnf<T>;

    fn bitor(self, rhs: &'b Dnf<T>) -> Self::Output {
        let mut ans: Dnf<T> = self.union(rhs).cloned().collect();
        ans.minimize();
        ans
    }
}

impl<T: Var> BitOrAssign for Dnf<T> {
    fn bitor_assign(&mut self, mut rhs: Dnf<T>) {
        self.append(&mut rhs);
        self.minimize();
    }
}

impl<'a, T: Var> BitOrAssign<&'a Dnf<T>> for Dnf<T> {
    fn bitor_assign(&mut self, rhs: &'a Dnf<T>) {
        self.extend(rhs.iter().cloned());
        self.minimize();
    }
}

#[macro_export]
macro_rules! dnf {
    () => {
        $crate::dnf::Dnf::<i32>::false_exp()
    };
    (false) => {
        $crate::dnf::Dnf::<i32>::false_exp()
    };
    (true) => {{
        $crate::dnf::Dnf::<i32>::true_exp()
    }};
    ($($x: literal)+ $(+ $($y:literal)+)*) => {{
        let mut exp = $crate::dnf::Dnf::new();
        exp.insert($crate::implicant!($($x)+));
        $(
            exp.insert($crate::implicant!($($y)+));
        )*
        exp
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dnf;

    #[test]
    fn test_display() {
        assert_eq!("FALSE", format!("{}", dnf!()));
        assert_eq!("FALSE", format!("{}", dnf!(false)));
        assert_eq!("TRUE", format!("{}", dnf!(true)));
        assert_eq!("1", format!("{}", dnf!(1)));
        assert_eq!("1 2", format!("{}", dnf!(1 2)));
        assert_eq!("1 + 2 3", format!("{}", dnf!(1 + 2 3)));
    }

    #[test]
    fn test_is_true_is_false() {
        assert!(dnf!(true).is_true());
        assert!(dnf!(false).is_false());
        assert!(!dnf!(1 + 2).is_true());
        assert!(!dnf!(1 + 2).is_false());
    }

    #[test]
    fn test_minimize() {
        let mut actual = dnf!(1 + 1 2 + 1 2 3 + 4 5 6 + 4 6 + 5 6 7 8 9 + 6 8 + 10 11 + 11 12);
        actual.minimize();
        let expect = dnf!(1 + 4 6 + 6 8 + 10 11 + 11 12);
        assert_eq!(actual, expect);
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(dnf!(1 2), dnf!(1) & dnf!(2));
        assert_eq!(dnf!(1 + 2), dnf!(1) | dnf!(2));
        assert_eq!(dnf!(1 3 + 1 4 + 2 3 + 2 4), dnf!(1 + 2) & dnf!(3 + 4));
        assert_eq!(dnf!(1 + 2 + 3 + 4), dnf!(1 + 2) | dnf!(3 + 4));
    }

    #[test]
    fn test_eval() {
        assert!(dnf!(true).eval(&BTreeSet::from([1]), true));
        assert!(!dnf!(false).eval(&BTreeSet::from([1]), true));

        let exp = dnf!(1 2 3 + 4 5 6);
        assert!(!exp.eval(&BTreeSet::from([1, 4]), true));
        assert!(exp.eval(&BTreeSet::from([1, 2, 3]), true));
        assert!(!exp.eval(&BTreeSet::from([1, 4]), false));
    }

    #[test]
    fn test_partial_eval() {
        let exp = dnf!(1 2 3 + 4 5 6);
        assert_eq!(
            dnf!(2 3 + 5 6),
            exp.partial_eval(&BTreeSet::from([1, 4]), true)
        );
        assert_eq!(
            dnf!(true),
            exp.partial_eval(&BTreeSet::from([1, 2, 3]), true)
        );
        assert_eq!(dnf!(4 5 6), exp.partial_eval(&BTreeSet::from([1]), false));
        assert_eq!(
            dnf!(false),
            exp.partial_eval(&BTreeSet::from([1, 4]), false)
        );

        let exp = dnf!(1 2 + 1 3);
        assert_eq!(dnf!(1), exp.partial_eval(&BTreeSet::from([2, 3]), true));
    }

    #[test]
    fn test_partial_eval_complement() {
        let exp = dnf!(1 2 3 + 4 5 6);
        assert_eq!(
            dnf!(2 3 + 5 6),
            exp.partial_eval_complement(&BTreeSet::from([2, 3, 5, 6]), true)
        );
        assert_eq!(
            dnf!(true),
            exp.partial_eval_complement(&BTreeSet::from([4, 5, 6]), true)
        );
        assert_eq!(
            dnf!(4 5 6),
            exp.partial_eval_complement(&BTreeSet::from([2, 3, 4, 5, 6]), false)
        );
        assert_eq!(
            dnf!(false),
            exp.partial_eval_complement(&BTreeSet::from([2, 3, 5, 6]), false)
        );

        let exp = dnf!(1 2 + 1 3);
        assert_eq!(
            dnf!(1),
            exp.partial_eval_complement(&BTreeSet::from([1]), true)
        );
    }

    #[test]
    fn test_partial_exp_contraction_exp() {
        let exp = dnf!(1 2 4 5 + 1 2 6 + 2 3 4 5 + 2 3 6 + 4 6);
        let input_set = BTreeSet::from([1, 2, 3]);
        assert_eq!(
            dnf!(1 2 4 5 + 1 2 6 + 2 3 4 5 + 2 3 6),
            exp.partial_exp(&input_set)
        );
        assert_eq!(dnf!(4 6), exp.partial_exp_complement(&input_set));
        assert_eq!(dnf!(1 2 + 2 3), exp.contraction_exp(&input_set));
    }
}
