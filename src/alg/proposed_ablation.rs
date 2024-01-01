use crate::{Game, ShapleyValues};

use super::synthesis_sv::recursive_decompose_ablation::{
    cal_sv_recursive_decompose_ablation, AblationType,
};

pub fn proposed_ablation_method(game: &Game, ablation_type: AblationType) -> ShapleyValues {
    // info!("proposed method ({})...");
    cal_sv_recursive_decompose_ablation(game, ablation_type)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_method;

    #[test]
    fn test_recursive_decompose() {
        test_method(
            |game| proposed_ablation_method(game, AblationType::NoVertical),
            true,
        );
        test_method(
            |game| proposed_ablation_method(game, AblationType::NoHorizontal),
            true,
        );
        test_method(
            |game| proposed_ablation_method(game, AblationType::NoHybrid),
            true,
        );
    }
}
