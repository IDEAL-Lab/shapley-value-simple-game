use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::Display,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
    derive_more::Into,
)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct OwnerId(pub u32);

#[derive(
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
    derive_more::IntoIterator,
    ref_cast::RefCast,
)]
#[from(forward)]
#[as_ref(forward)]
#[as_mut(forward)]
#[repr(transparent)]
pub struct OwnerSet(pub BTreeSet<OwnerId>);

impl FromIterator<u32> for OwnerSet {
    fn from_iter<T: IntoIterator<Item = u32>>(iter: T) -> Self {
        Self(iter.into_iter().map(OwnerId).collect())
    }
}

impl FromIterator<OwnerId> for OwnerSet {
    fn from_iter<T: IntoIterator<Item = OwnerId>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
