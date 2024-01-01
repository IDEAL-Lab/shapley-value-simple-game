use crate::{
    utils::{binom, hashmap_reduce},
    Game, OwnerSet, ShapleyValues,
};
use rayon::prelude::*;

mod non_linear_comb;
mod non_linear_lookup;

pub fn synthesis_method(game: &Game) -> ShapleyValues {
    let syns: &Vec<&OwnerSet> = &game.to_syns();

    if let Some((count, k)) = is_linear(syns) {
        cal_sv_linear(syns, count, k)
    } else {
        cal_sv_non_linear(syns, &game.owner_set)
    }
}

fn is_linear(syns: &[&OwnerSet]) -> Option<(usize, usize)> {
    let mut count = 0;
    let mut k = 0;
    for syn in syns.iter() {
        if syn.len() > 1 {
            count += 1;
            k = syn.len();
        }

        if count > 1 {
            return None;
        }
    }

    Some((count, k))
}

fn cal_sv_linear(syns: &Vec<&OwnerSet>, count: usize, k: usize) -> ShapleyValues {
    let alpha = count;
    let beta = syns.len() - count;
    let sv_alpha = if alpha == 0 {
        0.
    } else {
        alpha as f64 / ((k + beta) * binom(k - 1, k + beta - 1)) as f64
    };
    let sv_beta = if beta == 0 {
        0.
    } else {
        (1. - k as f64 * sv_alpha) / beta as f64
    };
    let mut ans = ShapleyValues::new();
    for syn in syns.iter() {
        if syn.len() == 1 {
            let id = syn.iter().next().unwrap();
            ans.insert(*id, sv_beta);
        } else {
            for id in syn.iter() {
                ans.insert(*id, sv_alpha);
            }
        }
    }
    ans
}

fn cal_sv_non_linear(syns: &[&OwnerSet], owner_set: &OwnerSet) -> ShapleyValues {
    let scale = 1.0;
    owner_set
        .par_iter()
        .map(|&owner_id| {
            let mut ans = ShapleyValues::new();

            let syns_with_current_owner: Vec<_> = syns
                .iter()
                .copied()
                .filter(|s| s.contains(&owner_id))
                .collect();
            let syns_without_current_owner: Vec<_> = syns
                .iter()
                .copied()
                .filter(|s| !s.contains(&owner_id))
                .collect();

            let number_of_pow_for_syns = if syns_with_current_owner.is_empty() {
                syns_without_current_owner.len()
            } else if syns_without_current_owner.is_empty() {
                syns_with_current_owner.len()
            } else {
                syns_with_current_owner.len() * syns_without_current_owner.len()
            };

            if owner_set.len() as f64 <= scale * number_of_pow_for_syns as f64 {
                let u = non_linear_lookup::cal_sv_lookup_individual(
                    &syns_with_current_owner,
                    &syns_without_current_owner,
                    owner_set,
                    owner_id,
                );
                ans.insert(owner_id, u);
            } else {
                let u = non_linear_comb::cal_sv_non_linear_comb(
                    &syns_with_current_owner,
                    &syns_without_current_owner,
                );
                ans.insert(owner_id, u);
            }
            ans
        })
        .reduce(ShapleyValues::default, hashmap_reduce)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_method;

    #[test]
    fn test() {
        test_method(|game| synthesis_method(game), true);
    }
}
