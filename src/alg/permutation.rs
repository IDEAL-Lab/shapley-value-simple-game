use crate::{
    alg::subset_utility::subset_utility_with_cache, utils::hashmap_reduce, Game, OwnerId, OwnerSet,
    ShapleyValues,
};
use dashmap::DashMap;
use rand::prelude::*;
use rayon::prelude::*;

pub fn permutation_method(game: &Game, sample_size: usize) -> ShapleyValues {
    let cache: DashMap<OwnerSet, f64> = DashMap::new();
    let cache_ref = &cache;

    let mut shapley_values = (0..sample_size)
        .into_par_iter()
        .map(|_| {
            // info!("sample #{}", i);
            let mut rng = thread_rng();
            let mut owners: Vec<OwnerId> = game.owner_set.iter().copied().collect();
            owners.shuffle(&mut rng);

            let mut last_utility = 0.;
            let mut owner_set = OwnerSet::default();
            let mut ans = ShapleyValues::new();

            for owner in owners {
                owner_set.insert(owner);
                let subset_utility = subset_utility_with_cache(game, owner_set.clone(), cache_ref);
                ans.insert(owner, subset_utility - last_utility);
                last_utility = subset_utility;
            }

            // info!("sample #{} done", i);
            ans
        })
        .reduce(ShapleyValues::new, hashmap_reduce);

    shapley_values.par_iter_mut().for_each(|(_, v)| {
        *v /= sample_size as f64;
    });

    shapley_values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_method;

    #[test]
    fn test() {
        test_method(|game| permutation_method(game, 100), false);
    }
}
