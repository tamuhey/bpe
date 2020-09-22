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
    fn nth_from(&self, pos: (usize, usize), offset: isize) -> Option<(usize, usize)> {
        let (sid, mut i) = pos;
        let links = &self.links[sid];
        for _ in offset..0 {
            i = links.get(i)?.0;
        }
        for _ in 0..offset {
            i = links.get(i)?.1;
        }
        Some((sid, i))
    }
    fn pair_chars(
        &self,
        pos: (usize, usize),
        left: isize,
        right: isize,
    ) -> Option<(&[char], (usize, usize))> {
        let l = self.nth_from(pos, left)?;
        let r = self.nth_from(pos, right)?;
        Some((&self.sentences[l.0][l.1..r.1], l))
    }

    fn remove_node(&mut self, pos: (usize, usize)) {
        let (sid, l) = self.nth_from(pos, -1).unwrap();
        let (_sid, r) = self.nth_from(pos, 1).unwrap();
        debug_assert_eq!(sid, _sid);
        let links = &mut self.links[sid];
        links[l].1 = r;
        links[r].0 = l;
    }
}
// 1. すべてのペアを抽出
// 2. 最頻ペアを求める
// 3. オーバーラップしている候補ペアについて処理
pub fn train(opts: TrainOpts) -> Result<()> {
    let sentences = get_sentences(&opts.input)?;
    let links: Vec<Vec<_>> = sentences
        .iter()
        .map(|s| (0..s.len()).map(|i| (i.wrapping_sub(1), i + 1)).collect())
        .collect();
    let (mut cand_pos, mut cand_pairs) = get_candidates(&sentences);
    let mut doc = Documents { sentences, links };

    // buffer for pairs to be modified
    let mut pairs_rm = HashMap::<&[char], Vec<(usize, usize)>>::new();
    let mut pairs_add = HashMap::<&[char], Vec<(usize, usize)>>::new();
    let mut nodes_rm = HashSet::<(usize, usize)>::new();
    for _ in 0..10 {
        let (_, best_pair) = cand_pos.pop_last().unwrap();
        let positions = cand_pairs.remove(best_pair).unwrap();
        for pos in positions {
            let (sid, i) = pos;
            if nodes_rm.contains(&pos) {
                continue;
            }
            // left
            if let Some((pair, pos)) = doc.pair_chars((sid, i), -1, 1) {
                pairs_rm.entry(pair).or_default().push(pos);
                let (pair, pos) = doc.pair_chars(pos, 0, 3).unwrap();
                pairs_add.entry(pair).or_default().push(pos);
            };
            // right
            if let Some((pair, pos)) = doc.pair_chars((sid, i), 1, 3) {
                pairs_rm.entry(pair).or_default().push(pos);
                let (pair, pos) = doc.pair_chars(pos, -1, 2).unwrap();
                pairs_add.entry(pair).or_default().push(pos);
            };
            nodes_rm.insert(pos);
        }

        for (pair, positions) in &pairs_rm {
            let v = cand_pairs.get_mut(pair).unwrap();
            cand_pos.remove(&(v.len(), pair));
            for pos in positions {
                v.remove(pos);
            }
        }
        for (pair, positions) in &pairs_add {
            let v = cand_pairs.get_mut(pair).unwrap();
            cand_pos.remove(&(v.len(), pair));
            for &pos in positions {
                debug_assert!(v.insert(pos));
            }
        }

        // re-compute freq for each pairs
        for (pair, _) in &pairs_rm {
            cand_pos.insert((cand_pairs[pair].len(), pair));
        }
        for (pair, _) in &pairs_add {
            cand_pos.insert((cand_pairs[pair].len(), pair));
        }

        // modify links
        for pos in &nodes_rm {
            doc.remove_node(*pos);
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
