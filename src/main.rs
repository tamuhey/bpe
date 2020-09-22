#![feature(map_first_last)]
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;
mod train;

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

    match opts.subcmd {
        SubCmd::Train(opts) => train::train(opts)?,
        SubCmd::Encode(opts) => {}
        SubCmd::Decode(opts) => {}
    }
    Ok(())
}
