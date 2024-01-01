use crate::{alg::subset_utility::subset_utility, Game, OwnerSet, ShapleyValues};
use itertools::Itertools;
use rayon::prelude::*;

pub fn traditional_method(game: &Game) -> ShapleyValues {
    // info!("traditional method...");
    let owner_len = game.owner_len();
    let shapley_values = game
        .owner_set
        .par_iter()
        .map(|&owner| {
            // info!("owner #{}", owner);
            let contribution: f64 = (0..owner_len)
                .into_par_iter()
                .map(move |k| {
                    let (utility, count) = game
                        .owner_set
                        .iter()
                        .copied()
                        .filter(|s| *s != owner)
                        .combinations(k)
                        .par_bridge()
                        .map(|subset| {
                            let mut subset = OwnerSet(subset.into_iter().collect());
                            let utility_without_owner = subset_utility(game, &subset);
                            subset.insert(owner);
                            let utility_with_owner = subset_utility(game, &subset);
                            (utility_with_owner - utility_without_owner, 1.)
                        })
                        .reduce(|| (0., 0.), |a, b| (a.0 + b.0, a.1 + b.1));
                    utility / count
                })
                .sum();
            // info!("owner #{} done", owner);
            (owner, contribution / owner_len as f64)
        })
        .collect::<ShapleyValues>();
    // info!("done in {:?}", total_time);

    shapley_values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_method;

    #[test]
    fn test() {
        test_method(|game| traditional_method(game), true);
    }
}
