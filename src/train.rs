use crate::norm;
use crate::protos::sentencepiece_model::{
    ModelProto, ModelProto_SentencePiece, ModelProto_SentencePiece_Type,
};
use anyhow::Result;
use clap::Clap;
use log;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::io::{self, prelude::*, BufReader};

#[derive(Clap)]
pub struct TrainOpts {
    #[clap(short, long, default_value = "100")]
    nstep: usize,
    #[clap(short, long)]
    model_prefix: String,
    input: String,
}

pub fn train(opts: TrainOpts) -> Result<()> {
    log::info!("Start train");
    let sentences = get_sentences(&opts.input)?;
    log::info!("Loaded texts from {}", &opts.input);

    let pieces = train_core(sentences, &opts);
    let path = opts.model_prefix.clone() + ".vocab";
    save_pieces_tsv(&pieces, &path)?;
    log::info!("Saved vocab to {}", path);

    let mut model = ModelProto::new();
    model.set_pieces(pieces.into());
    let path = opts.model_prefix + ".model";
    model.save(&path)?;
    log::info!("Saved model to {}", path);

    Ok(())
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

    fn all_words(&self) -> HashMap<&'a [char], usize> {
        let mut ret = HashMap::new();
        for (sentence, link) in self.sentences.iter().zip(self.links.iter()) {
            for (l, &(_, r)) in link.iter().enumerate() {
                if r <= sentence.len() && (r - l) > 1 {
                    *ret.entry(&sentence[l..r]).or_default() += 1;
                }
                ret.entry(&sentence[l..(l + 1)]).or_default();
            }
        }
        ret
    }

    #[inline]
    fn is_valid_pos(&self, pos: &(usize, usize)) -> bool {
        self.links[pos.0][pos.1].1 <= self.links[pos.0].len()
    }
}

fn is_valid_piece(piece: &[char]) -> bool {
    if piece.len() == 0 {
        return false;
    }
    if piece[piece.len() - 1] == norm::SPACE_REP {
        return false;
    }
    true
}

fn train_core(sentences: Vec<Vec<char>>, opts: &TrainOpts) -> Vec<ModelProto_SentencePiece> {
    let links: Vec<Vec<_>> = sentences
        .iter()
        .map(|s| (0..s.len()).map(|i| (i.wrapping_sub(1), i + 1)).collect())
        .collect();
    let (mut cand_pos, mut cand_pairs) = get_candidates(&sentences);
    let mut doc = Documents {
        sentences: &sentences,
        links,
    };

    log::info!("Start training loop");
    // buffer for pairs to be modified
    let mut processed = BTreeSet::new();
    let mut pairs_modified = vec![];
    'main: for i in 0..opts.nstep {
        processed.clear();
        pairs_modified.clear();
        if i % 20 == 0 {
            log::info!("Start {:<3} step", i);
        }
        let best_pair;
        loop {
            let pair = if let Some(last) = cand_pos.pop_last() {
                last.1
            } else {
                break 'main;
            };
            if is_valid_piece(pair) {
                best_pair = pair;
                break;
            }
        }
        log::trace!("best pair {:?}", &best_pair);
        // check all pairs
        let positions = cand_pairs.remove(best_pair).unwrap();
        for pos in positions {
            if let Some(prev) = doc.nth_from(pos, -1) {
                if processed.contains(&prev) {
                    // `pos` is no longer a valid pair
                    continue;
                }
            }
            processed.insert(pos);
        }

        // remove pairs
        let mut remove = |pair, pos| {
            if let Some(v) = cand_pairs.get_mut(pair) {
                cand_pos.remove(&(v.len(), pair));
                v.remove(&pos);
                pairs_modified.push(pair);
                if v.len() == 0 {
                    cand_pairs.remove(pair);
                }
            }
        };
        for &pos in &processed {
            // left
            if let Some((pair, pos)) = doc.pair_words(pos, -1, 1) {
                remove(pair, pos);
            }
            // right
            if let Some((pair, pos)) = doc.pair_words(pos, 1, 3) {
                remove(pair, pos);
            }
        }

        // Modify links
        for pos in &processed {
            doc.remove_node(doc.nth_from(*pos, 1).unwrap());
        }

        // Add new pairs
        let mut add = |pair, pos| {
            let v = cand_pairs.entry(pair).or_default();
            cand_pos.remove(&(v.len(), pair));
            v.insert(pos);
            pairs_modified.push(pair);
        };

        for &pos in &processed {
            // left
            if let Some((pair, pos)) = doc.pair_words(pos, -1, 1) {
                add(pair, pos);
            }
            // right
            if let Some((pair, pos)) = doc.pair_words(pos, 0, 2) {
                add(pair, pos);
            }
        }

        // re-compute freq for each pairs
        for pair in &pairs_modified {
            if let Some(l) = cand_pairs.get(pair).map(|s| s.len()) {
                if l > 0 {
                    cand_pos.insert((l, pair));
                } else {
                    cand_pairs.remove(pair);
                }
            }
        }
    }
    log::info!("End training loop");
    create_pieces(&doc)
}

fn save_pieces_tsv<P: AsRef<std::path::Path>>(
    pieces: &[ModelProto_SentencePiece],
    path: P,
) -> Result<()> {
    let mut f = BufWriter::new(File::create(path)?);
    for p in pieces {
        writeln!(f, "{}\t{}", p.get_piece(), p.get_score())?;
    }
    Ok(())
}

fn create_pieces(doc: &Documents) -> Vec<ModelProto_SentencePiece> {
    let mut ret = vec![];
    for (s, t) in &[
        ("<unk>", ModelProto_SentencePiece_Type::UNKNOWN),
        ("<s>", ModelProto_SentencePiece_Type::CONTROL),
        ("</s>", ModelProto_SentencePiece_Type::CONTROL),
    ] {
        let mut p = ModelProto_SentencePiece::new();
        p.set_piece(s.to_string());
        p.set_score(0.0);
        p.set_field_type(*t);
        ret.push(p);
    }
    let mut words: Vec<_> = doc.all_words().into_iter().collect();
    // Note: sort by reverse order
    words.sort_by(|a, b| b.1.cmp(&a.1));
    for (i, (s, _)) in words.iter().enumerate() {
        let mut p = ModelProto_SentencePiece::new();
        p.set_piece(s.into_iter().collect());
        p.set_score(-(i as f32));
        ret.push(p);
    }
    ret
}

fn get_candidates<'a>(
    sentences: &'a [Vec<char>],
) -> (
    BTreeSet<(usize, &[char])>,
    HashMap<&[char], BTreeSet<(usize, usize)>>,
) {
    let mut candidates = HashMap::<_, BTreeSet<_>>::new();
    for (i, line) in sentences.iter().enumerate() {
        for j in 0..(line.len() - 1) {
            let key = &line[j..j + 2];
            candidates.entry(key).or_default().insert((i, j));
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
    let mut ret = vec![];
    for line in BufReader::new(f).lines() {
        let line = norm::to_chars(&line?);
        if line.len() > 0 {
            ret.push(line);
        }
    }
    Ok(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use norm;
    fn run_train(fname: &str) {
        let opts = TrainOpts {
            input: fname.into(),
            nstep: 100,
            model_prefix: "/tmp/foo".into(),
        };
        train(opts).unwrap();
    }
    #[test]
    fn samples() {
        for fname in &[
            "tests/sample0.txt",
            "tests/sample1.txt",
            "tests/sample2.txt",
        ] {
            run_train(fname);
        }
    }
}
