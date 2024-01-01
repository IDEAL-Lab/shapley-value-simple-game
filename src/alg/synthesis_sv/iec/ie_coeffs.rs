use rayon::prelude::*;
use std::{
    collections::HashMap,
    iter::Sum,
    ops::{Add, Mul, Sub},
};

pub type SetLen = usize;
pub type Coeff = i32;

/// A hashmap of iec coefficients index by the size of subset.
#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
    derive_more::IntoIterator,
)]
#[from(forward)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct IECoeffs(pub(crate) HashMap<SetLen, Coeff>);

impl IECoeffs {
    pub fn to_sv(&self) -> f64 {
        self.par_iter()
            .map(|(set_len, coeff)| *coeff as f64 / *set_len as f64)
            .sum()
    }

    pub fn apply_sign(&mut self, sign: i32) {
        if sign == 1 {
            return;
        }

        self.iter_mut().for_each(|(_, v)| {
            *v *= sign;
        });
    }
}

impl Add<Self> for IECoeffs {
    type Output = Self;

    fn add(self, rhs: IECoeffs) -> Self::Output {
        let (to_consume, mut to_mutate) = if self.len() < rhs.len() {
            (self, rhs)
        } else {
            (rhs, self)
        };
        for (k, v) in to_consume {
            *to_mutate.entry(k).or_default() += v;
        }
        to_mutate
    }
}

impl<'a, 'b> Add<&'b IECoeffs> for &'a IECoeffs {
    type Output = IECoeffs;

    fn add(self, rhs: &'b IECoeffs) -> Self::Output {
        let (to_consume, mut to_mutate) = if self.len() < rhs.len() {
            (self, rhs.clone())
        } else {
            (rhs, self.clone())
        };
        for (k, v) in to_consume.iter() {
            *to_mutate.entry(*k).or_default() += *v;
        }
        to_mutate
    }
}

impl Sum for IECoeffs {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut ans = IECoeffs::default();
        for v in iter {
            ans = ans + v;
        }
        ans
    }
}

impl Sub<Self> for IECoeffs {
    type Output = Self;

    fn sub(mut self, rhs: IECoeffs) -> Self::Output {
        for (k, v) in rhs {
            *self.entry(k).or_default() -= v;
        }
        self
    }
}

impl<'a, 'b> Sub<&'b IECoeffs> for &'a IECoeffs {
    type Output = IECoeffs;

    fn sub(self, rhs: &'b IECoeffs) -> Self::Output {
        let mut ans = self.clone();
        for (k, v) in rhs.iter() {
            *ans.entry(*k).or_default() -= *v;
        }
        ans
    }
}

impl<'a, 'b> Mul<&'b IECoeffs> for &'a IECoeffs {
    type Output = IECoeffs;

    fn mul(self, rhs: &'b IECoeffs) -> Self::Output {
        let mut ans = IECoeffs::default();
        for (l_k, l_v) in self.iter() {
            for (r_k, r_v) in rhs.iter() {
                let k = l_k + r_k;
                let v = l_v * r_v;

                if k != 0_usize {
                    *ans.entry(k).or_default() += v;
                }
            }
        }
        ans
    }
}

pub fn horizontal_identity() -> IECoeffs {
    IECoeffs::default()
}

pub fn horizontal_op(a: &IECoeffs, b: &IECoeffs) -> IECoeffs {
    a + b - a * b
}

pub fn vertical_identity() -> IECoeffs {
    IECoeffs::from([(0, 1)])
}

pub fn vertical_op(a: &IECoeffs, b: &IECoeffs) -> IECoeffs {
    a * b
}

#[macro_export]
macro_rules! ie_coeffs {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$($crate::ie_coeffs!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { $crate::ie_coeffs!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = $crate::ie_coeffs!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            $crate::alg::synthesis_sv::iec::IECoeffs(_map)
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_arithmetic() {
        let a = ie_coeffs! { 1 => 1, 2 => 2 };
        let b = ie_coeffs! { 1 => 3, 2 => 4 };
        let c = ie_coeffs! { 2 => 3, 3 => 10, 4 => 8 };
        assert_eq!(c, &a * &b);
    }
}
