use super::{utils::*, Dnf, Var};
use std::{
    collections::BTreeSet,
    fmt,
    ops::{BitAnd, BitAndAssign, BitOr},
};

/// A term of DNF boolean expression.
///
/// If the underlying set is empty, the term is considered as TRUE.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
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
pub struct Implicant<T: Var>(pub BTreeSet<T>);

impl<T: Var> Implicant<T> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Eval to TRUE or FALSE.
    pub fn eval(&self, input_set: &BTreeSet<T>, input_is_true: bool) -> bool {
        if input_is_true {
            self.is_subset(input_set)
        } else {
            !has_intersection(&self.0, input_set)
        }
    }

    /// Partially eval the expression with variables in `input_set` set to be `input_is_true`.
    /// Return `None` if the result is FALSE.
    pub fn partial_eval(
        &self,
        input_set: &BTreeSet<T>,
        input_is_true: bool,
    ) -> Option<Implicant<T>> {
        if input_is_true {
            Some(self.difference(input_set).cloned().collect())
        } else if has_intersection(&self.0, input_set) {
            None
        } else {
            Some(self.clone())
        }
    }

    /// Partially eval the expression with variables not in `input_set` set to be `complement_is_true`.
    /// Return `None` if the result is FALSE.
    pub fn partial_eval_complement(
        &self,
        input_set: &BTreeSet<T>,
        complement_is_true: bool,
    ) -> Option<Implicant<T>> {
        if complement_is_true {
            Some(self.intersection(input_set).cloned().collect())
        } else if self.is_subset(input_set) {
            Some(self.clone())
        } else {
            None
        }
    }

    /// Apply f to every variable in the implicant.
    pub fn map_variable<U: Var>(&self, f: impl Fn(&T) -> U) -> Implicant<U> {
        self.iter().map(f).collect()
    }
}

impl<T: Var> Default for Implicant<T> {
    fn default() -> Self {
        Self(BTreeSet::new())
    }
}

impl<T: Var> FromIterator<T> for Implicant<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(BTreeSet::from_iter(iter))
    }
}

impl<T: Var> IntoIterator for Implicant<T> {
    type Item = <BTreeSet<T> as IntoIterator>::Item;
    type IntoIter = <BTreeSet<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T: Var> PartialOrd for Implicant<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Var> Ord for Implicant<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // The order is defined to be used by `Dnf::minimize`.
        self.len()
            .cmp(&other.len())
            .then_with(|| self.0.cmp(&other.0))
    }
}

impl<T: Var + fmt::Display> fmt::Display for Implicant<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_empty() {
            return write!(f, "TRUE");
        }

        for (i, v) in self.iter().enumerate() {
            if i != 0 {
                write!(f, " ")?;
            }
            write!(f, "{v}")?;
        }
        Ok(())
    }
}

impl<T: Var> BitAnd for Implicant<T> {
    type Output = Implicant<T>;

    fn bitand(self, rhs: Implicant<T>) -> Self::Output {
        let (mut to_mutate, mut to_consume) = if self.len() < rhs.len() {
            (rhs, self)
        } else {
            (self, rhs)
        };
        to_mutate.append(&mut to_consume);
        to_mutate
    }
}

impl<'a, 'b, T: Var> BitAnd<&'b Implicant<T>> for &'a Implicant<T> {
    type Output = Implicant<T>;

    fn bitand(self, rhs: &'b Implicant<T>) -> Self::Output {
        self.union(rhs).cloned().collect()
    }
}

impl<T: Var> BitAndAssign for Implicant<T> {
    fn bitand_assign(&mut self, mut rhs: Implicant<T>) {
        self.append(&mut rhs);
    }
}

impl<'a, T: Var> BitAndAssign<&'a Implicant<T>> for Implicant<T> {
    fn bitand_assign(&mut self, rhs: &'a Implicant<T>) {
        self.extend(rhs.iter().cloned());
    }
}

impl<T: Var> BitOr for Implicant<T> {
    type Output = Dnf<T>;

    fn bitor(self, rhs: Implicant<T>) -> Self::Output {
        let mut ans: Dnf<T> = [self, rhs].into_iter().collect();
        ans.minimize();
        ans
    }
}

impl<'a, 'b, T: Var> BitOr<&'b Implicant<T>> for &'a Implicant<T> {
    type Output = Dnf<T>;

    fn bitor(self, rhs: &'b Implicant<T>) -> Self::Output {
        let mut ans: Dnf<T> = [self, rhs].into_iter().cloned().collect();
        ans.minimize();
        ans
    }
}

#[macro_export]
macro_rules! implicant {
    () => {
        $crate::dnf::Implicant::<i32>::new()
    };
    ($($x: literal)+) => {{
        let mut t = $crate::dnf::Implicant::new();
        $(
            t.insert($x);
        )+
        t
    }};
}

#[cfg(test)]
mod tests {
    use crate::{dnf, implicant};

    #[test]
    fn test_display() {
        assert_eq!("TRUE", format!("{}", implicant!()));
        assert_eq!("1", format!("{}", implicant!(1)));
        assert_eq!("1 2", format!("{}", implicant!(1 2)));
        assert_eq!("1 2", format!("{}", implicant!(1 2 )));
    }

    #[test]
    fn test_arithmetic() {
        assert_eq!(implicant!(1 2), implicant!(1) & implicant!(2));
        assert_eq!(dnf!(1 + 2), implicant!(1) | implicant!(2));
        assert_eq!(dnf!(1), implicant!(1) | implicant!(1 2));
    }
}
