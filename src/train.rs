use anyhow::Result;
use clap::Clap;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

#[derive(Clap)]
pub struct TrainOpts {
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    input: String,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
struct Pair<'a> {
    freq: usize,
    // (sentence idx, character idx)
    positions: Vec<(usize, usize)>,
    chars: &'a [char],
}

impl<'a> Pair<'a> {
    fn new(positions: Vec<(usize, usize)>, chars: &'a [char]) -> Self {
        Self {
            freq: positions.len(),
            positions,
            chars,
        }
    }
}

pub fn train(opts: TrainOpts) -> Result<()> {
    let sentences = get_sentences(&opts.input)?;
    let links: Vec<Vec<_>> = sentences
        .iter()
        .map(|s| (0..s.len()).map(|i| (i.wrapping_sub(1), i + 1)).collect())
        .collect();
    let mut candidates = get_candidates(&sentences);
    let mut next = HashMap::new();
    for _ in 0..10 {
        if let Some(Pair {
            positions,
            chars,
            freq: _,
        }) = candidates.pop()
        {
            next.clear();
            for (i, j) in positions {
                let key = &sentences[i][j..(j + chars.len())];
            }
        }
    }
    Ok(())
}

fn get_candidates<'a>(sentences: &'a [Vec<char>]) -> BinaryHeap<Pair<'a>> {
    let mut candidates = HashMap::new();
    for (i, line) in sentences.iter().enumerate() {
        for j in 0..(line.len() - 1) {
            let key = &line[j..j + 2];
            candidates.entry(key).or_insert(vec![]).push((i, j));
        }
    }
    candidates
        .into_iter()
        .map(|(k, v)| Pair::new(v, k))
        .collect()
}

fn get_sentences(path: &str) -> Result<Vec<Vec<char>>> {
    let f = File::open(path)?;
    let sentences: Vec<Vec<char>> = BufReader::new(f)
        .lines()
        .map(|line| line.map(|line| line.chars().map(|c| c).collect::<Vec<_>>()))
        .collect::<Result<_, _>>()?;
    Ok(sentences)
}
