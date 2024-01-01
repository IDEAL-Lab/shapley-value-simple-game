use once_cell::sync::Lazy;
use polars_core::prelude::JoinType;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct JoinStep {
    pub table_to_join: &'static str,
    pub left_join_keys: Vec<&'static str>,
    pub right_join_keys: Vec<&'static str>,
    pub join_type: &'static JoinType,
}

#[derive(Debug, Clone)]
pub struct JoinPlan {
    pub init_table: &'static str,
    pub steps: Vec<JoinStep>,
}

pub static PLANS: Lazy<HashMap<&'static str, JoinPlan>> = Lazy::new(|| {
    let mut plans = HashMap::new();
    plans.insert(
        "tpch",
        JoinPlan {
            init_table: "region",
            steps: vec![
                JoinStep {
                    table_to_join: "nation",
                    left_join_keys: vec!["r_regionkey"],
                    right_join_keys: vec!["n_regionkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "supplier",
                    left_join_keys: vec!["n_nationkey"],
                    right_join_keys: vec!["s_nationkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "partsupp",
                    left_join_keys: vec!["s_suppkey"],
                    right_join_keys: vec!["ps_suppkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "part",
                    left_join_keys: vec!["ps_partkey"],
                    right_join_keys: vec!["p_partkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "lineitem",
                    left_join_keys: vec!["p_partkey", "ps_suppkey"],
                    right_join_keys: vec!["l_partkey", "l_suppkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "orders",
                    left_join_keys: vec!["l_orderkey"],
                    right_join_keys: vec!["o_orderkey"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "customer",
                    left_join_keys: vec!["o_custkey"],
                    right_join_keys: vec!["c_custkey"],
                    join_type: &JoinType::Inner,
                },
            ],
        },
    );

    plans.insert(
        "soccer",
        JoinPlan {
            init_table: "Match",
            steps: vec![
                JoinStep {
                    table_to_join: "Country",
                    left_join_keys: vec!["country_id"],
                    right_join_keys: vec!["id"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "HomeTeam",
                    left_join_keys: vec!["home_team_api_id"],
                    right_join_keys: vec!["team_api_id"],
                    join_type: &JoinType::Inner,
                },
                JoinStep {
                    table_to_join: "AwayTeam",
                    left_join_keys: vec!["away_team_api_id"],
                    right_join_keys: vec!["team_api_id"],
                    join_type: &JoinType::Left,
                },
                JoinStep {
                    table_to_join: "League",
                    left_join_keys: vec!["league_id"],
                    right_join_keys: vec!["id"],
                    join_type: &JoinType::Inner,
                },
            ],
        },
    );

    plans
});
