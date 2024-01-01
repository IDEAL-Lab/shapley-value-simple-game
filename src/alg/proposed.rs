use crate::{Game, ShapleyValues};

use super::synthesis_sv::recursive_decompose::cal_sv_recursive_decompose;

pub fn proposed_method(game: &Game) -> ShapleyValues {
    // info!("proposed method ({})...");
    cal_sv_recursive_decompose(game)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::test_method;

    #[test]
    fn test_recursive_decompose() {
        test_method(|game| proposed_method(game), true);
    }
}
