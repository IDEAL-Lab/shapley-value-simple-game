#[macro_use]
extern crate tracing;

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

pub mod alg;
pub mod dnf;
pub mod game;
pub mod owner;
pub mod product_tree;
pub mod union_combination;
pub mod utils;

pub mod table;
pub use table::*;

pub mod dataset;
pub use dataset::*;

pub mod join_plan;
pub use join_plan::*;

#[cfg(test)]
pub(crate) mod tests;

pub use dnf::Dnf;
pub use game::Game;
pub use owner::{OwnerId, OwnerSet};
pub type ShapleyValues = HashMap<OwnerId, f64>;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SVResult {
    #[serde(with = "serde_time")]
    pub avg_time: Duration,
    #[serde(with = "serde_time")]
    pub total_time: Duration,
    #[serde(with = "serde_time")]
    pub load_time: Duration,
    #[serde(with = "serde_time")]
    pub sv_cal_time: Duration,
    pub shapley_values: ShapleyValues,
    pub num_of_owners: usize,
}

mod serde_time {
    use super::*;
    use serde::{de::Deserializer, ser::Serializer};

    pub fn serialize<S: Serializer>(t: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_f64(t.as_secs_f64())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let t = <f64>::deserialize(d)?;
        Ok(Duration::from_secs_f64(t))
    }
}
