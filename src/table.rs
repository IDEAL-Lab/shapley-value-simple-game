use crate::{OwnerId, OwnerSet};
use anyhow::{Context, Result};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fs::File, io::BufReader, path::Path};

pub const ROW_ID_COL_NAME: &str = "_row_id";

#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    derive_more::Display,
    derive_more::Constructor,
    derive_more::Deref,
    derive_more::DerefMut,
    derive_more::AsRef,
    derive_more::AsMut,
    derive_more::From,
    derive_more::Into,
)]
#[as_ref(forward)]
#[as_mut(forward)]
pub struct RowId(pub u64);

#[derive(Debug, Clone)]
pub struct Table {
    pub name: String,
    pub df: DataFrame,
    pub owner_map: HashMap<RowId, OwnerSet>,
}

impl Table {
    pub fn load(
        name: impl Into<String>,
        csv_path: impl AsRef<Path>,
        row_id_path: impl AsRef<Path>,
        owner_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let mut df = CsvReader::new(File::open(csv_path)?).finish()?;
        let row_id: Vec<u64> = serde_json::from_reader(BufReader::new(File::open(row_id_path)?))?;
        df.with_column(Series::new(ROW_ID_COL_NAME, row_id))?;

        #[derive(Debug, Deserialize)]
        struct Owner {
            index: HashMap<String, RowId>,
            owner: HashMap<String, OwnerId>,
        }

        let owner: Owner = serde_json::from_reader(BufReader::new(File::open(owner_path)?))?;
        let mut owner_map: HashMap<RowId, OwnerSet> = HashMap::new();
        for (i_k, i_v) in owner.index {
            let s_v = *owner
                .owner
                .get(&i_k)
                .context("failed to read -owner.json")?;
            owner_map.entry(i_v).or_default().insert(s_v);
        }

        Ok(Self {
            name: name.into(),
            df,
            owner_map,
        })
    }

    pub fn load_without_assignment(
        name: impl Into<String>,
        csv_path: impl AsRef<Path>,
    ) -> Result<Self> {
        let df = CsvReader::new(File::open(csv_path)?).finish()?;

        Ok(Self {
            name: name.into(),
            df,
            owner_map: HashMap::default(),
        })
    }
}
