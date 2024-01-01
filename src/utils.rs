use crate::{Dnf, OwnerId, OwnerSet};
use anyhow::{Error, Result};
use ref_cast::RefCast;
#[cfg(test)]
use std::path::PathBuf;
use std::{cmp, collections::HashMap, hash::Hash, ops::AddAssign};
use tracing_subscriber::EnvFilter;

pub fn init_tracing_subscriber(default_filter: &str) -> Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_filter));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init()
        .map_err(Error::msg)
}

pub fn setup_rayon(num_threads: Option<usize>) -> Result<()> {
    if let Some(num_threads) = num_threads {
        rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()?;
    }
    Ok(())
}

#[cfg(test)]
pub fn test_data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

#[inline]
pub fn binom(k: usize, n: usize) -> usize {
    let k = cmp::min(k, n - k);
    let mut res = 1;
    let mut n = n;

    for d in 1..=k {
        res *= n;
        res /= d;
        n -= 1;
    }

    res
}

#[inline]
pub fn binom_coeffs(n: usize) -> Vec<usize> {
    let mut ans = Vec::with_capacity(n + 1);
    let mut v = 1;
    let mut l = n;
    ans.push(v);
    for d in 1..=n {
        v *= l;
        v /= d;
        l -= 1;
        ans.push(v);
    }
    ans
}

#[inline]
pub fn hashmap_fold<K, V>(mut acc: HashMap<K, V>, (k, v): (K, V)) -> HashMap<K, V>
where
    K: Eq + Hash,
    V: Default + AddAssign,
{
    *acc.entry(k).or_default() += v;
    acc
}

#[inline]
pub fn hashmap_reduce<K, V>(a: HashMap<K, V>, b: HashMap<K, V>) -> HashMap<K, V>
where
    K: Eq + Hash,
    V: Default + AddAssign,
{
    let (to_consume, mut to_mutate) = if a.len() < b.len() { (a, b) } else { (b, a) };
    for (k, v) in to_consume {
        *to_mutate.entry(k).or_default() += v;
    }
    to_mutate
}

#[inline]
pub fn dnf_to_syns(exp: &Dnf<OwnerId>) -> Vec<&'_ OwnerSet> {
    exp.iter().map(|imp| OwnerSet::ref_cast(&imp.0)).collect()
}

#[inline]
pub fn dnf_to_syns_with_filter(
    exp: &Dnf<OwnerId>,
    mut f: impl FnMut(&OwnerSet) -> bool,
) -> Vec<&'_ OwnerSet> {
    exp.iter()
        .map(|imp| OwnerSet::ref_cast(&imp.0))
        .filter(|s| f(s))
        .collect()
}

pub fn cartesian_product(owner_sets: &[OwnerSet]) -> Vec<OwnerSet> {
    match owner_sets.split_first() {
        Some((first, rest)) => {
            let init: Vec<OwnerSet> = first.iter().cloned().map(|n| OwnerSet::from([n])).collect();

            rest.iter().cloned().fold(init, partial_cartesian)
        }
        None => {
            vec![]
        }
    }
}

pub fn partial_cartesian(a: Vec<OwnerSet>, b: OwnerSet) -> Vec<OwnerSet> {
    a.into_iter()
        .flat_map(|xs| {
            b.iter()
                .cloned()
                .map(|y| {
                    let mut set = xs.clone();
                    set.insert(y);
                    set
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binom() {
        let n = 10;
        let coeffs = binom_coeffs(n);
        for k in 0..=n {
            assert_eq!(binom(k, n), coeffs[k]);
        }
    }

    #[test]
    fn test_cartesian_product() {
        let a = OwnerSet::from_iter([1, 2, 3]);
        let b = OwnerSet::from_iter([4, 5]);
        let c = OwnerSet::from_iter([6, 7, 8]);

        let products = cartesian_product(&vec![a, b, c]);
        dbg!(&products);
    }
}
