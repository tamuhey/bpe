#![feature(map_first_last)]
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
mod decode;
mod model;
mod norm;
mod protos;
mod train;
mod util;
use env_logger;
mod spec;
use log::{self, LevelFilter};

use anyhow::Result;
use chrono::Local;
use clap::Clap;
use std::io::prelude::*;

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCmd,
    #[clap(short, long, parse(from_occurrences))]
    verbose: u32,
}

#[derive(Clap)]
enum SubCmd {
    Train(spec::TrainSpec),
    Encode(EncodeOpts),
    Decode(decode::DecodeOpts),
}

#[derive(Clap)]
struct EncodeOpts {
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    input: String,
}

fn main() -> Result<()> {
    let spec: Opts = Opts::parse();
    let level = match spec.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::builder()
        .filter(None, level)
        .format(|buf, record| {
            writeln!(
                buf,
                "{}:{} {} [{}] - {}",
                record.file().unwrap(),
                record.line().unwrap(),
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                record.args()
            )
        })
        .init();

    match spec.subcmd {
        SubCmd::Train(spec) => train::train(spec)?,
        SubCmd::Encode(spec) => {}
        SubCmd::Decode(spec) => decode::decode(spec)?,
    }
    Ok(())
}
