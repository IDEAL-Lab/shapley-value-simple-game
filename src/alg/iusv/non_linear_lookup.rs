use crate::{utils::binom_coeffs, OwnerId, OwnerSet};
use rayon::prelude::*;

#[derive(Clone)]
struct Subset {
    next_id: usize,
    set: OwnerSet,
    with_flag: bool,
}

impl Subset {
    fn utility_with_current_owner(
        &mut self,
        owner: OwnerId,
        syns_with_current_owner: &[&OwnerSet],
    ) -> bool {
        if self.with_flag {
            return true;
        }

        let syn_len_upper_bound = self.set.len() + 1;
        self.with_flag = syns_with_current_owner.par_iter().any(|syn| {
            if syn.len() > syn_len_upper_bound {
                return false;
            }
            let mut syn = (*syn).clone();
            syn.remove(&owner);
            self.set.is_superset(&syn)
        });
        self.with_flag
    }

    fn utility_without_current_owner(&self, syns_without_current_owner: &[&OwnerSet]) -> bool {
        syns_without_current_owner
            .par_iter()
            .any(|syn| self.set.is_superset(syn))
    }
}

pub fn cal_sv_lookup_individual(
    syns_with_current_owner: &[&OwnerSet],
    syns_without_current_owner: &[&OwnerSet],
    owners: &OwnerSet,
    current_owner: OwnerId,
) -> f64 {
    let number_of_owners = owners.len();
    let rest_of_owners: Vec<_> = owners
        .iter()
        .copied()
        .filter(|s| *s != current_owner)
        .collect();
    let rest_of_owners_len = rest_of_owners.len();

    let mut marginal_contribution_for_current_owner = 0.;
    let mut init_subset = Subset {
        next_id: 0,
        set: OwnerSet::default(),
        with_flag: false,
    };

    if init_subset.utility_with_current_owner(current_owner, syns_with_current_owner) {
        // when subset is empty; number_of_sub_combination = 1 and without_flag = false
        marginal_contribution_for_current_owner += 1.;
    }

    let binom_coeffs = binom_coeffs(rest_of_owners_len);
    let mut subsets: Vec<Subset> = vec![init_subset];
    let mut chosen = 1;

    while !subsets.is_empty() {
        let (marginal_contribution_in_sub_combination, new_subsets): (usize, Vec<Subset>) = subsets
            .par_iter()
            .flat_map(|old_s| {
                (old_s.next_id..rest_of_owners_len)
                    .into_par_iter()
                    .filter_map(|next_id| {
                        let mut new_s = old_s.clone();
                        new_s.next_id = next_id + 1;
                        new_s.set.insert(rest_of_owners[next_id]);

                        if new_s.utility_with_current_owner(current_owner, syns_with_current_owner)
                        {
                            if new_s.utility_without_current_owner(syns_without_current_owner) {
                                // early stop
                                return None;
                            } else {
                                return Some((1, new_s));
                            }
                        }

                        Some((0, new_s))
                    })
            })
            .fold(
                || (0, Vec::new()),
                |mut acc, input| {
                    acc.0 += input.0;
                    acc.1.push(input.1);
                    acc
                },
            )
            .reduce(
                || (0, Vec::new()),
                |mut a, mut b| -> (usize, Vec<Subset>) {
                    a.0 += b.0;
                    a.1.append(&mut b.1);
                    a
                },
            );

        if marginal_contribution_in_sub_combination != 0 {
            let number_of_sub_combination = binom_coeffs[chosen];
            marginal_contribution_for_current_owner +=
                marginal_contribution_in_sub_combination as f64 / number_of_sub_combination as f64;
        }

        subsets = new_subsets;
        chosen += 1;
    }

    marginal_contribution_for_current_owner / number_of_owners as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::assert_f64_eq;
    use crate::ShapleyValues;

    fn cal_sv_lookup(syns: &[&OwnerSet], owners: &OwnerSet) -> ShapleyValues {
        owners
            .par_iter()
            .map(|&owner_id| {
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
                let u = cal_sv_lookup_individual(
                    &syns_with_current_owner,
                    &syns_without_current_owner,
                    owners,
                    owner_id,
                );
                (owner_id, u)
            })
            .collect()
    }

    #[test]
    fn test_lookup() {
        let syns: Vec<OwnerSet> = vec![
            OwnerSet::from_iter([1, 3]),
            OwnerSet::from_iter([2, 3]),
            OwnerSet::from_iter([4]),
            OwnerSet::from_iter([5]),
        ];
        let syns_ref: Vec<_> = syns.iter().collect();
        let owners = OwnerSet::from_iter(1..=5);
        let sv = cal_sv_lookup(&syns_ref, &owners);
        assert_f64_eq(0.05, sv[&OwnerId(1)]);
        assert_f64_eq(0.13333333333, sv[&OwnerId(3)]);
        assert_f64_eq(0.3833333333333335, sv[&OwnerId(5)]);
    }
}
