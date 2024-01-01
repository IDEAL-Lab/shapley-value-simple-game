use crate::{Game, OwnerSet};
use dashmap::DashMap;

pub(crate) fn subset_utility(game: &Game, subset: &OwnerSet) -> f64 {
    if game.dnf.eval(subset, true) {
        1.
    } else {
        0.
    }
}

#[inline]
pub(crate) fn subset_utility_with_cache(
    game: &Game,
    subset: OwnerSet,
    cache: &DashMap<OwnerSet, f64>,
) -> f64 {
    if let Some(u) = cache.get(&subset) {
        return *u;
    }

    let u = subset_utility(game, &subset);
    cache.insert(subset, u);
    u
}
