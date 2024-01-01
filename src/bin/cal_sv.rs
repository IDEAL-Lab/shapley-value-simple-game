#[macro_use]
extern crate tracing;

use anyhow::{Context, Ok, Result};
use clap::{Parser, ValueEnum};
use rayon::prelude::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde_json::json;
use shapley_value_decomposition::{utils::hashmap_reduce, *};
use std::{fs::File, io::BufWriter, path::PathBuf, time::Instant};

#[derive(Debug, Parser)]
struct Args {
    /// Input dataset
    #[clap(short = 'd', long, value_parser)]
    dataset: String,

    /// Input dataset file
    #[clap(short = 'c', long, value_parser)]
    csv_dir: PathBuf,

    /// Input owner assignment file
    #[clap(short = 'a', long, value_parser)]
    assignment_dir: Option<PathBuf>,

    /// Output file
    #[clap(short, long, value_parser)]
    output: PathBuf,

    /// Method
    #[clap(short, long, value_enum)]
    method: Method,

    /// Sample size (for permutation method)
    #[clap(short, long)]
    sample_size: Option<usize>,

    /// Number of threads
    #[clap(short = 't', long)]
    num_threads: Option<usize>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Method {
    /// Traditional method
    #[clap(alias("trad"))]
    Traditional,
    /// Permutation method
    #[clap(alias("perm"))]
    Permutation,
    /// IUSV method
    #[clap(alias("iusv"))]
    IUSV,
    /// Proposed method with recursive decompose
    #[clap(alias("rdsv"))]
    RDSV,
}

fn main() -> Result<()> {
    utils::init_tracing_subscriber("info")?;
    let args = Args::from_args();
    info!("args: {:#?}", args);
    utils::setup_rayon(args.num_threads)?;

    let begin = Instant::now();

    let (result, load_time, sv_cal_time) = polars_core::POOL.install(|| {
        let begin_load = Instant::now();
        let dataset = DataSet::load(&args.dataset, &args.csv_dir, &args.assignment_dir).unwrap();
        let load_time = Instant::now() - begin_load;
        let games = Game::generate_games(&dataset).unwrap();

        println!(" # of games: {}", &games.len());

        let begin_cal = Instant::now();
        let shapley_values = games
            .into_par_iter()
            .enumerate()
            .map(|(i, game)| {
                if i % 100_000 == 0 {
                    info!("game: #{}", i);
                }
                match args.method {
                    Method::Traditional => alg::traditional::traditional_method(&game),
                    Method::Permutation => alg::permutation::permutation_method(
                        &game,
                        args.sample_size.context("need sample size").unwrap(),
                    ),
                    Method::IUSV => alg::iusv::synthesis_method(&game),
                    Method::RDSV => alg::proposed::proposed_method(&game),
                }
            })
            .reduce(ShapleyValues::default, hashmap_reduce);

        let sv_cal_time = Instant::now() - begin_cal;
        info!("time in sv_cal {:?}", sv_cal_time);

        (shapley_values, load_time, sv_cal_time)
    });

    let total_time = Instant::now() - begin;
    let num_of_owners = result.len();
    let avg_time = total_time / num_of_owners as u32;

    let sv_result = SVResult {
        shapley_values: result,
        total_time,
        avg_time,
        load_time,
        sv_cal_time,
        num_of_owners,
    };

    let mut result_json = serde_json::to_value(sv_result)?;
    result_json.as_object_mut().unwrap().append(
        json!({
            "method": format!("{:?}", args.method).to_lowercase(),
            "csv_dir": args.csv_dir,
            "assignment_dir": args.assignment_dir,
            "num_threads": args.num_threads,
            "sample_size": args.sample_size,
        })
        .as_object_mut()
        .unwrap(),
    );

    let out = BufWriter::new(File::create(&args.output)?);
    serde_json::to_writer(out, &result_json)?;

    Ok(())
}
