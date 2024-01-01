use crate::{
    alg::join::join,
    dnf::{Dnf, Implicant},
    owner::{OwnerId, OwnerSet},
    utils::{cartesian_product, dnf_to_syns},
    DataSet, RowId, PLANS, ROW_ID_COL_NAME,
};
use anyhow::{Context, Error, Ok, Result};
use polars_core::{
    prelude::{AnyValue, DataFrame, NamedFrom},
    series::{ChunkCompare, Series},
};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, str::FromStr};

/// A simple game among data owners.
#[derive(Debug, Clone)]
pub struct Game {
    pub dnf: Dnf<OwnerId>,
    pub owner_set: OwnerSet,
}

impl Game {
    pub fn new(exp: Dnf<OwnerId>) -> Self {
        let owner_set = exp.all_variables().into();
        Self {
            dnf: exp,
            owner_set,
        }
    }

    pub fn owner_len(&self) -> usize {
        self.owner_set.len()
    }

    pub fn to_syns(&self) -> Vec<&'_ OwnerSet> {
        dnf_to_syns(&self.dnf)
    }

    pub fn generate_games(dataset: &DataSet) -> Result<Vec<Self>> {
        if dataset.owner_set.is_empty() {
            Self::generate_games_without_assignment(dataset)
        } else {
            Self::generate_games_with_assignment(dataset)
        }
    }

    fn generate_games_with_assignment(dataset: &DataSet) -> Result<Vec<Self>> {
        info!("join...");
        let join_df = join(
            |table_name| dataset.tables.get(table_name).map(|t| &t.df),
            PLANS
                .get(dataset.name.as_str())
                .context("cannot find join plan")?,
            true,
        )?;

        info!("extract row_id_columns...");
        let row_id_columns: Vec<(String, Vec<RowId>)> = join_df
            .columns(
                dataset
                    .tables
                    .keys()
                    .map(|t| format!("{}:{}", ROW_ID_COL_NAME, t)),
            )?
            .into_iter()
            .map(|column| {
                let table_name = column.name().rsplit(':').next().unwrap().to_string();
                let row_ids = column
                    .u64()
                    .unwrap()
                    .into_iter()
                    .map(|row_id| RowId::new(row_id.unwrap()))
                    .collect();
                (table_name, row_ids)
            })
            .collect();
        let rows = join_df.shape().0;
        let cols = row_id_columns.len();
        drop(join_df);

        info!("create games...");
        let row_id_columns_ref = &row_id_columns;
        let games: Vec<_> = (0..rows)
            .into_par_iter()
            .map(move |i| {
                let owner_sets = (0..cols)
                    .map(move |j| {
                        let (table_name, row_ids) = &row_id_columns_ref[j];
                        let row_id = row_ids[i];
                        let owner_set = &dataset.tables[table_name].owner_map[&row_id];
                        owner_set.clone()
                    })
                    .collect::<Vec<OwnerSet>>();

                let owner_sets_per_tuple = cartesian_product(&owner_sets);

                let mut exp_set: BTreeSet<Implicant<OwnerId>> = BTreeSet::default();
                for owner_set in owner_sets_per_tuple {
                    let set = Implicant::from_iter(owner_set);
                    exp_set.insert(set);
                }

                let mut exp = Dnf::from(exp_set);
                exp.minimize();

                let owner_set = exp.all_variables().into();
                Self {
                    dnf: exp,
                    owner_set,
                }
            })
            .collect();

        drop(row_id_columns);

        Ok(games)
    }

    fn generate_games_without_assignment(dataset: &DataSet) -> Result<Vec<Self>> {
        let df = Self::join_df(dataset)?;
        let rows = df.height();
        let cols = df.width();
        let games: Vec<Self> = (0..rows)
            .into_par_iter()
            .map(|row_idx| {
                // Create a vector to store the Series for the current row
                let mut row_series: Vec<Series> = Vec::with_capacity(cols);

                // Iterate through each column
                for col_idx in 0..cols {
                    // Select the column by index
                    let cell = df.select_at_idx(col_idx).unwrap().get(row_idx);

                    let owner_list = match cell {
                        AnyValue::List(rows) => Ok(rows),
                        _ => Err(Error::msg("unexpected col in the aggregated df")),
                    };

                    row_series.push(owner_list.unwrap());
                }
                Self::generate_games_with_agg_helper(&row_series).unwrap()
            })
            .collect();
        Ok(games)
    }

    fn join_df(dataset: &DataSet) -> Result<DataFrame> {
        info!("join...");
        let mut join_df = join(
            |table_name| dataset.tables.get(table_name).map(|t| &t.df),
            PLANS
                .get(dataset.name.as_str())
                .context("cannot find join plan")?,
            false,
        )?;

        Self::win(&mut join_df);

        let mask_season = join_df
            .column("season")
            .expect("season:2015/2016")
            .equal("2015/2016")
            .unwrap();
        let mask_winner = join_df
            .column("winner")
            .expect("Not a draw!")
            .not_equal("DRAW")
            .unwrap();
        let combined_mask = mask_season & mask_winner;

        join_df = join_df.filter(&combined_mask).unwrap();

        let cols = join_df
            .get_column_names()
            .into_par_iter()
            .filter_map(|col| {
                if col.contains("api_id") {
                    Some(col.to_string())
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        join_df = join_df
            .groupby(["winner"])
            .unwrap()
            .select(cols)
            .agg_list()
            .unwrap();
        join_df = join_df.sort(["winner"], false)?;
        join_df = join_df.drop("winner")?;
        Ok(join_df)
    }

    fn win(df: &mut DataFrame) {
        let home_score = df.column("home_team_goal").unwrap().i64().unwrap();
        let away_score = df.column("away_team_goal").unwrap().i64().unwrap();

        //TODO: get the team names
        let home_team_name = df.column("team_long_name").unwrap().utf8().unwrap();
        let away_team_name = df
            .column("team_long_name:AwayTeam")
            .unwrap()
            .utf8()
            .unwrap();
        let draw = "DRAW";

        let winner = home_score
            .into_iter()
            .zip(away_score.into_iter())
            .zip(home_team_name.into_iter())
            .zip(away_team_name.into_iter())
            .map(
                |(((home_score, away_score), home_team_name), away_team_name)| {
                    if home_score > away_score {
                        home_team_name.unwrap()
                    } else if home_score < away_score {
                        away_team_name.unwrap()
                    } else {
                        draw
                    }
                },
            )
            .collect::<Vec<_>>();

        let _ = df.with_column(Series::new("winner", winner));
    }

    pub fn generate_games_with_agg_helper(row_series: &[Series]) -> Result<Self> {
        let rows = row_series[0].len(); // Assuming all RowSeries have the same length
        let cols = row_series.len();

        let owner_sets: Vec<OwnerSet> = (0..rows)
            .into_par_iter()
            .map(|i| {
                let owner_set = (0..cols)
                    .filter_map(|j| match row_series[j].get(i) {
                        AnyValue::UInt32(owner_id) => Some(OwnerId::from(owner_id)),
                        AnyValue::UInt64(owner_id) => Some(OwnerId::from(owner_id as u32)),
                        AnyValue::Int32(owner_id) => Some(OwnerId::from(owner_id as u32)),
                        AnyValue::Int64(owner_id) => Some(OwnerId::from(owner_id as u32)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                OwnerSet::from_iter(owner_set)
            })
            .collect();

        let mut exp_set = BTreeSet::new();
        exp_set.extend(owner_sets.into_iter().map(Implicant::from_iter));

        let mut exp = Dnf::from(exp_set);
        exp.minimize();

        let owner_set = exp.all_variables().into();
        let game = Self {
            dnf: exp,
            owner_set,
        };

        Ok(game)
    }
}

/// A boolean expression
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum BoolExp<T> {
    And { and: Vec<BoolExp<T>> },
    Or { or: Vec<BoolExp<T>> },
    Elm(T),
}

impl<T> FromStr for BoolExp<T>
where
    T: for<'de> Deserialize<'de>,
{
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str(s)?)
    }
}
