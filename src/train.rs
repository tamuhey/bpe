use anyhow::Result;
use clap::Clap;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};
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

#[derive(Debug)]
struct Documents {
    sentences: Vec<Vec<char>>,
    links: Vec<Vec<(usize, usize)>>,
}

impl Documents {
    fn chars(&self, sid: usize, i: usize) -> &[char] {
        &self.sentences[sid][i..self.links[sid][i].1]
    }
    fn pair_chars(
        &self,
        sid: usize,
        i: usize,
        left: isize,
        right: isize,
    ) -> Option<(&[char], (usize, usize))> {
        let mut l = i;
        let links = &self.links[sid];
        for _ in left..0 {
            if l >= links.len() {
                return None;
            }
            l = links[l].0;
        }
        let mut r = links[i].1;
        for _ in 0..(right - 1) {
            if r >= links.len() {
                return None;
            }
            r = links[r].1;
        }
        Some((&self.sentences[sid][l..r], (sid, l)))
    }
    fn remove_node(&mut self, sid: usize, i: usize) {
        let links = &mut self.links[sid];
        let (l, r) = links[i];
        if let Some(v) = links.get_mut(l) {
            v.1 = r;
        }
        if let Some(v) = links.get_mut(r) {
            v.0 = l;
        }
        links[i] = (!0, !0);
    }
}
// 1. すべてのペアを抽出
// 2. 最頻ペアを求める
// 3. オーバーラップしている候補ペアについて処理
pub fn train(opts: TrainOpts) -> Result<()> {
    let sentences = get_sentences(&opts.input)?;
    let mut links: Vec<Vec<_>> = sentences
        .iter()
        .map(|s| (0..s.len()).map(|i| (i.wrapping_sub(1), i + 1)).collect())
        .collect();
    let mut doc = Documents { sentences, links };
    let (mut cand_pos, mut cand_pairs) = get_candidates(&doc.sentences);

    // buffer for pairs to be modified
    let mut staging_rm = vec![];
    let mut staging_add = vec![];
    for _ in 0..10 {
        let (_, best_pair) = cand_pos.pop_last().unwrap();
        let positions = cand_pairs.remove(best_pair).unwrap();
        for (sid, i) in positions {
            // left
            if let Some((pair, pos)) = doc.pair_chars(sid, i, -1, 1) {
                staging_rm.push((pair, pos));
                staging_add.push((doc.pair_chars(sid, pos.1, 0, 2).unwrap(), pos))
            };
            // right
            if let Some((pair, pos)) = doc.pair_chars(sid, i, 1, 2) {
                staging_rm.push((pair, pos));
                staging_add.push((doc.pair_chars(sid, pos.1, -1, 1).unwrap(), pos))
            };
            doc.remove_node(sid, i);
        }
    }
    Ok(())
}

fn get_candidates<'a>(
    sentences: &'a [Vec<char>],
) -> (
    BTreeSet<(usize, &[char])>,
    HashMap<&[char], HashSet<(usize, usize)>>,
) {
    let mut candidates = HashMap::new();
    for (i, line) in sentences.iter().enumerate() {
        for j in 0..(line.len() - 1) {
            let key = &line[j..j + 2];
            candidates
                .entry(key)
                .or_insert(HashSet::new())
                .insert((i, j));
        }
    }
    let mut positions = BTreeSet::new();
    let mut pairs = HashMap::new();
    for (key, pos) in candidates {
        positions.insert((pos.len(), key));
        pairs.insert(key, pos);
    }
    (positions, pairs)
}

fn get_sentences(path: &str) -> Result<Vec<Vec<char>>> {
    let f = File::open(path)?;
    let sentences: Vec<Vec<char>> = BufReader::new(f)
        .lines()
        .map(|line| line.map(|line| line.chars().map(|c| c).collect::<Vec<_>>()))
        .collect::<Result<_, _>>()?;
    Ok(sentences)
}
