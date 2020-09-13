use clap::Clap;
use std::io::{self, prelude::*};

#[derive(Clap)]
pub struct TrainOpts {
    #[clap(short, long)]
    nstep: u32,
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    input: String,
}

pub fn train(opts: TrainOpts) {}

fn get_sentences(path: ) -> Vec<String> {
}
