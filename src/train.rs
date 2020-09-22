use anyhow::Result;
use clap::Clap;
use log;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

#[derive(Clap)]
pub struct TrainOpts {
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    #[clap(short, long, default_value = "100")]
    nstep: usize,
    input: String,
}

#[derive(Debug)]
struct Documents<'a> {
    sentences: &'a Vec<Vec<char>>,
    links: Vec<Vec<(usize, usize)>>,
}

impl<'a> Documents<'a> {
    fn nth_from(&self, pos: (usize, usize), offset: isize) -> Option<(usize, usize)> {
        let (sid, mut i) = pos;
        let links = &self.links[sid];
        for _ in offset..0 {
            i = links.get(i)?.0;
        }
        for _ in 0..offset {
            i = links.get(i)?.1;
        }
        if i > links.len() {
            None
        } else {
            Some((sid, i))
        }
    }

    fn pair_words(
        &self,
        pos: (usize, usize),
        left: isize,
        right: isize,
    ) -> Option<(&'a [char], (usize, usize))> {
        let l = self.nth_from(pos, left)?;
        let r = self.nth_from(pos, right)?;
        Some((&self.sentences[l.0][l.1..r.1], l))
    }

    fn remove_node(&mut self, pos: (usize, usize)) -> Option<()> {
        let (sid, l) = self.nth_from(pos, -1)?;
        let (_sid, r) = self.nth_from(pos, 1)?;
        debug_assert_eq!(sid, _sid);
        let links = &mut self.links[sid];
        links[pos.1] = (!0, !0);
        links[l].1 = r;
        links.get_mut(r)?.0 = l;
        Some(())
    }

    fn all_words(&self) -> HashSet<&'a [char]> {
        let mut ret = HashSet::new();
        for (sentence, link) in self.sentences.iter().zip(self.links.iter()) {
            for (l, &(_, r)) in link.iter().enumerate() {
                if r <= sentence.len() {
                    ret.insert(&sentence[l..r]);
                }
            }
        }
        ret
    }
}

pub fn train(opts: TrainOpts) -> Result<()> {
    log::info!("start train");
    let sentences = get_sentences(&opts.input)?;
    let links: Vec<Vec<_>> = sentences
        .iter()
        .map(|s| (0..s.len()).map(|i| (i.wrapping_sub(1), i + 1)).collect())
        .collect();
    let (mut cand_pos, mut cand_pairs) = get_candidates(&sentences);
    let mut doc = Documents {
        sentences: &sentences,
        links,
    };

    // buffer for pairs to be modified
    let mut pairs_rm = HashMap::<&[char], Vec<(usize, usize)>>::new();
    let mut pairs_add = HashMap::<&[char], Vec<(usize, usize)>>::new();
    let mut nodes_rm = HashSet::<(usize, usize)>::new();
    for _ in 0..opts.nstep {
        log::trace!("cand pairs {:?}", &cand_pairs);
        log::trace!("links {:?}", doc.links);
        let (_, best_pair) = if let Some(last) = cand_pos.pop_last() {
            last
        } else {
            break;
        };
        log::trace!("best pair {:?}", &best_pair);
        let positions = cand_pairs.remove(best_pair).unwrap();
        for pos in positions {
            let (sid, i) = pos;
            log::trace!("pos {:?}", pos);
            if nodes_rm.contains(&pos) {
                continue;
            }
            // left
            if let Some((pair, pos)) = doc.pair_words((sid, i), -1, 1) {
                pairs_rm.entry(pair).or_default().push(pos);
                let (pair, pos) = doc.pair_words(pos, 0, 3).unwrap();
                pairs_add.entry(pair).or_default().push(pos);
            };
            // right
            if let Some((pair, pos)) = doc.pair_words((sid, i), 1, 3) {
                pairs_rm.entry(pair).or_default().push(pos);
                let (pair, pos) = doc.pair_words(pos, -1, 2).unwrap();
                pairs_add.entry(pair).or_default().push(pos);
            };
            nodes_rm.insert(pos);
        }

        log::trace!("pairs_rm {:?}", &pairs_rm);
        log::trace!("pairs_add {:?}", &pairs_add);
        log::trace!("nodes_rm {:?}", &nodes_rm);

        for (pair, positions) in &pairs_rm {
            let v = cand_pairs.get_mut(pair).unwrap();
            cand_pos.remove(&(v.len(), pair));
            for pos in positions {
                v.remove(pos);
            }
        }
        for (pair, positions) in &pairs_add {
            let v = cand_pairs.entry(pair).or_default();
            cand_pos.remove(&(v.len(), pair));
            for &pos in positions {
                debug_assert!(v.insert(pos));
            }
        }

        // re-compute freq for each pairs
        for (pair, _) in &pairs_rm {
            let l = cand_pairs[pair].len();
            if l > 0 {
                cand_pos.insert((l, pair));
            } else {
                cand_pairs.remove(pair);
            }
        }
        for (pair, _) in &pairs_add {
            let l = cand_pairs[pair].len();
            if l > 0 {
                cand_pos.insert((l, pair));
            } else {
                cand_pairs.remove(pair);
            }
        }
        pairs_rm.clear();
        pairs_add.clear();
        // modify links
        for pos in &nodes_rm {
            log::trace!("{:?}", pos);
            doc.remove_node(doc.nth_from(*pos, 1).unwrap());
        }
        nodes_rm.clear();
    }
    eprintln!("{:?}", doc.all_words()); // DEBUG
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
