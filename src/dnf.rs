//! Ref: Jan C. Bioch, Modular Decomposition of Boolean Functions, 2002
#![allow(clippy::module_inception)]

mod decompose;
mod dnf;
mod implicant;
mod modular_closure;
pub(crate) mod recursive_decompose;
mod unionfind;
mod utils;

pub use decompose::{decompose, Decompose, SubExp};
pub use dnf::Dnf;
pub use implicant::Implicant;
pub use recursive_decompose::{recursive_decompose, RecursiveDecompose};

/// Trait for boolean expression variable.
pub trait Var: Clone + Ord + Eq + Sync + Send {}
impl<T: Clone + Ord + Eq + Sync + Send> Var for T {}
