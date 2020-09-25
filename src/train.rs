use crate::norm;
use crate::protos::sentencepiece_model::{
    ModelProto, ModelProto_SentencePiece, ModelProto_SentencePiece_Type,
};
use anyhow::{anyhow, Result};
use clap::Clap;
use log;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs::File;
use std::io::BufWriter;
use std::io::{prelude::*, BufReader};

#[derive(Clap, Debug)]
pub struct TrainOpts {
    #[clap(short, long, default_value = "8000")]
    vocab_size: usize,
    #[clap(short, long)]
    model_prefix: String,
    input: String,
    #[cfg(debug_assertions)]
    #[clap(long)]
    slow: bool,
}

pub fn train(opts: TrainOpts) -> Result<()> {
    log::info!("Start train");
    log::debug!("Config: {:?}", opts);

    let pieces = if cfg!(debug_assertions) && opts.slow {
        log::warn!("Running with slow bpe");
        slow_bpe(&opts)?
    } else {
        train_core(&opts)?
    };
    let path = opts.model_prefix.clone() + ".vocab";
    pieces.save_pieces_tsv(&path)?;
    log::info!("Saved vocab to {}", path);

    let mut model = ModelProto::new();
    model.set_pieces(pieces.to_vec().into());
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

fn train_core(opts: &TrainOpts) -> Result<Pieces> {
    let sentences = get_sentences(&opts.input)?;
    log::info!("Loaded texts from {}", &opts.input);

    let mut pieces = Pieces::new(&sentences);
    log::info!("Created {} pieces", pieces.len());
    if opts.vocab_size < pieces.len() {
        let msg = format!("vocab_size must be larger than {}", pieces.len());
        log::error!("{}", &msg);
        return Err(anyhow!(msg));
    }

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
    let mut fixed_pieces = HashMap::<&[char], usize>::new();

    let mut counter = 0;
    let mut max_piece_count = 0;
    let mut size;
    while {
        counter += 1;
        size = fixed_pieces.len() + pieces.len();
        max_piece_count = std::cmp::max(max_piece_count, size);
        size < opts.vocab_size
    } {
        processed.clear();
        pairs_modified.clear();
        if cfg!(debug_assertions) || counter % 20 == 0 {
            log::info!("Start {:<3} step. piece size: {}", counter, size);
        }

        // pop best pair
        let best_pair = loop {
            let pair = if let Some(last) = cand_pos.pop_last() {
                last.1
            } else {
                let msg = format!(
                    "vocab_size must be less than or equal to {}",
                    max_piece_count
                );
                log::error!("{}", &msg);
                return Err(anyhow!(msg));
            };
            if is_valid_piece(pair) {
                break pair;
            }
        };
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

        // fix best_pair
        *fixed_pieces.entry(best_pair).or_default() += processed.len();

        // modify fixed word counts
        for &pos in &processed {
            for i in 0..=1 {
                if let Some((p, _)) = doc.pair_words(pos, i, i + 1) {
                    if let Some(v) = fixed_pieces.get_mut(p) {
                        *v -= 1;
                        if *v == 0 {
                            fixed_pieces.remove(p);
                        }
                    }
                }
            }
        }

        // remove candidate pairs
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

        // Add new candidate pairs
        let mut ret = |pair, pos| {
            let v = cand_pairs.entry(pair).or_default();
            cand_pos.remove(&(v.len(), pair));
            v.insert(pos);
            pairs_modified.push(pair);
        };

        for &pos in &processed {
            // left
            if let Some((pair, pos)) = doc.pair_words(pos, -1, 1) {
                ret(pair, pos);
            }
            // right
            if let Some((pair, pos)) = doc.pair_words(pos, 0, 2) {
                ret(pair, pos);
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
    log::debug!("Created {:?} pieces", fixed_pieces.len());
    debug_assert_eq!(
        doc.all_words().iter().filter(|x| x.0.len() > 1).count(),
        fixed_pieces.len()
    );
    log::info!("End training loop");
    pieces.add_pieces(
        fixed_pieces
            .into_iter()
            .map(|(k, v)| (k.into_iter().collect(), v))
            .collect(),
    );
    debug_assert_eq!(pieces.len(), opts.vocab_size);
    Ok(pieces)
}

struct Pieces {
    predefined: Vec<ModelProto_SentencePiece>,
    chars: Vec<ModelProto_SentencePiece>,
    pieces: Vec<ModelProto_SentencePiece>,
}

impl Pieces {
    fn new(sentences: &[Vec<char>]) -> Self {
        Self {
            predefined: Self::get_predefined_pieces(),
            chars: Self::init_pieces(sentences),
            pieces: vec![],
        }
    }
    fn len(&self) -> usize {
        self.predefined.len() + self.chars.len() + self.pieces.len()
    }

    fn get_predefined_pieces() -> Vec<ModelProto_SentencePiece> {
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
        ret
    }

    fn init_pieces(sentences: &[Vec<char>]) -> Vec<ModelProto_SentencePiece> {
        let mut ret = HashSet::new();
        for line in sentences {
            for c in line {
                ret.insert(c);
            }
        }
        ret.into_iter()
            .map(|k| {
                let mut p = ModelProto_SentencePiece::new();
                p.set_piece(k.to_string());
                p.set_field_type(ModelProto_SentencePiece_Type::NORMAL);
                p
            })
            .collect()
    }

    fn add_pieces(&mut self, mut words: Vec<(String, usize)>) {
        // Note: sort by reverse order
        words.sort_by(|a, b| b.1.cmp(&a.1));
        self.pieces
            .extend(words.into_iter().enumerate().map(|(i, (s, _))| {
                let mut p = ModelProto_SentencePiece::new();
                p.set_piece(s);
                p.set_score(-(i as f32));
                p
            }));
    }

    fn iter(&self) -> impl Iterator<Item = &ModelProto_SentencePiece> {
        self.predefined
            .iter()
            .chain(self.pieces.iter())
            .chain(self.chars.iter())
    }

    fn to_vec(self) -> Vec<ModelProto_SentencePiece> {
        let Self {
            mut predefined,
            mut pieces,
            mut chars,
        } = self;
        predefined.append(&mut pieces);
        predefined.append(&mut chars);
        predefined
    }

    fn save_pieces_tsv<P: AsRef<std::path::Path>>(&self, path: P) -> Result<()> {
        let mut f = BufWriter::new(File::create(path)?);
        for p in self.iter() {
            writeln!(f, "{}\t{}", p.get_piece(), p.get_score())?;
        }
        Ok(())
    }
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

#[cfg(debug_assertions)]
fn slow_bpe(opts: &TrainOpts) -> Result<Pieces> {
    let sentences = get_sentences(&opts.input)?;
    let mut pieces = Pieces::new(&sentences);
    let mut encoded: Vec<Vec<String>> = sentences
        .iter()
        .map(|line| line.into_iter().map(|c| c.to_string()).collect())
        .collect();
    fn get_freq<'a>(encoded: &'a [Vec<String>]) -> HashMap<(&'a String, &'a String), usize> {
        let mut freq = HashMap::<_, usize>::new();
        for line in encoded {
            for (a, b) in line.iter().zip(line.iter().skip(1)) {
                *freq.entry((a, b)).or_default() += 1;
            }
        }
        freq
    };
    let mut max_size = 0;
    let freq = loop {
        let mut freq: Vec<_> = get_freq(&encoded).into_iter().collect();
        freq.sort_by_key(|x| x.1);
        let pair = 'outer: loop {
            while let Some(((a, b), _)) = freq.pop() {
                if !b.ends_with(norm::SPACE_REP) {
                    break 'outer (a.clone(), b.clone());
                }
            }
            return Err(anyhow!(
                "max_size {:?}, but vocab_size {:?}",
                max_size,
                opts.vocab_size
            ));
        };
        let (a, b) = pair;
        let p = format!("{}{}", a, b);
        encoded = encoded
            .into_iter()
            .map(|line| {
                let mut i = 0;
                let mut next_line = vec![];
                while i < line.len() {
                    if i < line.len() - 1 && (&line[i], &line[i + 1]) == (&a, &b) {
                        next_line.push(p.clone());
                        i += 2;
                    } else {
                        next_line.push(line[i].clone());
                        i += 1;
                    }
                }
                next_line
            })
            .collect();
        let mut freq = HashMap::new();
        for line in &encoded {
            for word in line {
                if word.chars().count() > 1 {
                    *freq.entry(word.clone()).or_default() += 1;
                }
            }
        }
        let size = freq.len() + pieces.len();
        max_size = std::cmp::max(size, max_size);
        if size == opts.vocab_size {
            break freq;
        }
    };
    assert_eq!(pieces.len() + freq.len(), opts.vocab_size);
    pieces.add_pieces(freq.into_iter().collect());
    assert_eq!(opts.vocab_size, pieces.len());
    Ok(pieces)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn run_samples() {
        for (fname, vocab_size) in &[
            ("tests/sample0.txt", 8),
            ("tests/sample1.txt", 100),
            ("tests/sample2.txt", 6),
            ("tests/sample4.txt", 9),
        ] {
            let opts = TrainOpts {
                input: fname.to_string(),
                vocab_size: *vocab_size,
                model_prefix: "/tmp/foo".into(),
                slow: false,
            };
            train(opts).unwrap();
            println!("ok {:?}", fname); // DEBUG
        }
    }

    #[test]
    fn check_with_slow_algorithm() {
        for (fname, vocab_size) in &[
            ("tests/sample0.txt", 7),
            ("tests/sample1.txt", 80),
            ("tests/sample2.txt", 6),
        ] {
            let opts = TrainOpts {
                input: fname.to_string(),
                vocab_size: *vocab_size,
                model_prefix: "/tmp/main".into(),
                slow: false,
            };
            let a: BTreeSet<_> = train_core(&opts)
                .unwrap()
                .pieces
                .into_iter()
                .map(|x| x.get_piece().to_string())
                .collect();
            let b: BTreeSet<_> = slow_bpe(&opts)
                .unwrap()
                .pieces
                .into_iter()
                .map(|x| x.get_piece().to_string())
                .collect();
            assert_eq!(a, b, "failed with {}", fname,);
            println!("OK {}", fname);
        }
    }
}
