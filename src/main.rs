use clap::Clap;
use std::{
    collections::HashMap,
    io::{self, Read},
};

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
enum Data {
    Prime(char),
    Pair((u32, u32)),
}
use Data::*;

enum EncodeOpt {
    NTimes(u32),
}

#[derive(Debug, Default, Clone)]
struct Vocab {
    table: Vec<Data>,
    inv: HashMap<Data, u32>,
}

impl Vocab {
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn push(&mut self, c: Data) -> u32 {
        if !self.inv.contains_key(&c) {
            let i = self.len() as u32;
            self.inv.insert(c, i);
            self.table.push(c);
            i
        } else {
            self.inv[&c]
        }
    }

    pub fn decode(&self, data: &[u32], s: &mut String) {
        for &x in data {
            self.decode_unit(x, s)
        }
    }

    fn decode_unit(&self, x: u32, s: &mut String) {
        match self.table[x as usize] {
            Prime(c) => s.push(c),
            Pair((y, z)) => {
                self.decode_unit(y, s);
                self.decode_unit(z, s);
            }
        }
    }

    pub fn encode(&mut self, text: &str, opt: EncodeOpt) -> Vec<u32> {
        let n = match opt {
            EncodeOpt::NTimes(n) => n,
        };
        let mut dat = self.encode_first_step(text);
        let mut counter = HashMap::new();
        let mut new_dat = vec![];
        for _ in 0..n {
            if !self.encode_step(&mut dat, &mut new_dat, &mut counter) {
                break;
            }
        }
        dat
    }

    /// Encode `dat` for one step
    /// Returns whether the step proceeded or not.
    fn encode_step<'a>(
        &mut self,
        dat: &'a mut Vec<u32>,
        new_dat: &'a mut Vec<u32>,
        counter: &mut HashMap<Data, u32>,
    ) -> bool {
        new_dat.clear();
        counter.clear();

        // Find most frequently occuring pair
        for (&s, &t) in dat.iter().zip(dat.iter().skip(1)) {
            let c = Pair((s, t));
            *counter.entry(c).or_insert(0) += 1;
        }
        let best: Data = if let Some(v) = counter.iter().max_by_key(|(_, v)| **v).map(|(k, _)| k) {
            *v
        } else {
            return false;
        };

        // Encode the pair
        let code = self.push(best);
        let mut i = 0;
        while i < dat.len() {
            if i != dat.len() - 1 && Pair((dat[i], dat[i + 1])) == best {
                new_dat.push(code);
                i += 1;
            } else {
                new_dat.push(dat[i]);
            }
            i += 1;
        }

        std::mem::swap(dat, new_dat);
        true
    }

    /// Convert chars to code
    fn encode_first_step(&mut self, text: &str) -> Vec<u32> {
        let mut dat: Vec<u32> = vec![];
        for c in text.chars() {
            let c = Prime(c);
            dat.push(self.push(c));
        }
        dat
    }
}

#[derive(Clap)]
struct Opts {
    #[clap(short, long)]
    ntimes: u32,
}

fn main() {
    let opts: Opts = Opts::parse();
    let mut text = String::new();
    io::stdin().lock().read_to_string(&mut text).unwrap();
    println!("{:?}", text); // DEBUG
    let mut vocab = Vocab::default();
    let dat = vocab.encode(&text, EncodeOpt::NTimes(opts.ntimes));
    println!("{:?}", dat);
    let mut s = String::new();
    vocab.decode(&dat, &mut s);
    println!("{}", s);
}
