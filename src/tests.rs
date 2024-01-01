use super::*;
use once_cell::sync::Lazy;

static FIXTURE_GAME: Lazy<Game> = Lazy::new(|| {
    let exp = dnf!(1 2 4 + 1 2 5 + 2 3 4 + 2 3 5 + 4 5);
    Game::new(exp.map_variable(|owner_id| OwnerId(*owner_id)))
});

static FIXTURE_RESULT: Lazy<ShapleyValues> = Lazy::new(|| {
    ShapleyValues::from([
        (OwnerId(1), 0.06666666666),
        (OwnerId(2), 0.23333333333),
        (OwnerId(3), 0.06666666666),
        (OwnerId(4), 0.31666666666),
        (OwnerId(5), 0.31666666666),
    ])
});

pub(crate) fn test_method(f: impl Fn(&Game) -> ShapleyValues, is_accurate: bool) {
    let actual = f(Lazy::force(&FIXTURE_GAME));
    let expect = Lazy::force(&FIXTURE_RESULT);

    assert_eq!(actual.len(), expect.len());
    if is_accurate {
        for (o, u) in actual {
            let u_e = expect[&o];
            assert_f64_eq(u_e, u);
        }
    } else {
        let sum_sv: f64 = actual.values().copied().sum();
        assert_f64_eq(1., sum_sv);
    }
}

pub(crate) fn assert_f64_eq(expect: f64, actual: f64) {
    if (expect - actual).abs() > 1e-5 {
        panic!("assert failed. expect: {expect}, actual: {actual}.");
    }
}
