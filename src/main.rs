#![feature(map_first_last)]
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
mod train;
use env_logger::{self, Builder, Target};
use log::{self, LevelFilter};

use anyhow::Result;
use clap::Clap;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, prelude::*, BufReader},
};

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCmd,
    #[clap(short, long, parse(from_occurrences))]
    verbose: u32,
}

#[derive(Clap)]
enum SubCmd {
    Train(train::TrainOpts),
    Encode(EncodeOpts),
    Decode(DecodeOpts),
}

#[derive(Clap)]
struct EncodeOpts {
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    input: String,
}

#[derive(Clap)]
struct DecodeOpts {
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    #[clap(short, long, default_value = "output.txt")]
    out: String,
    input: String,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let level = match opts.verbose {
        0 => LevelFilter::Warn,
        1 => LevelFilter::Info,
        2 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    let mut builder = Builder::from_default_env();
    builder.filter_level(level);
    builder.init();

    match opts.subcmd {
        SubCmd::Train(opts) => train::train(opts)?,
        SubCmd::Encode(opts) => {}
        SubCmd::Decode(opts) => {}
    }
    Ok(())
}
