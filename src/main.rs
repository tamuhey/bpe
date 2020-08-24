#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;

use clap::Clap;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{self, prelude::*},
};

enum EncodeOpt {
    NTimes(u32),
}

#[derive(Debug, Default, Clone)]
struct Vocab {
    table: Vec<String>,
    inv: HashMap<String, u32>,
}

impl Vocab {
    pub fn len(&self) -> usize {
        self.table.len()
    }
    pub fn push(&mut self, c: String) -> u32 {
        if !self.inv.contains_key(&c) {
            let i = self.len() as u32;
            self.table.push(c.clone());
            self.inv.insert(c, i);
            i
        } else {
            self.inv[c.as_str()]
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
    fn encode_step(
        &mut self,
        dat: &mut Vec<u32>,
        new_dat: &mut Vec<u32>,
        counter: &mut HashMap<(u32, u32), u32>,
    ) -> bool {
        new_dat.clear();
        counter.clear();

        // Find most frequently occuring pair
        for (&s, &t) in dat.iter().zip(dat.iter().skip(1)) {
            let c = (s, t);
            *counter.entry(c).or_insert(0) += 1;
        }
        let best: (u32, u32) =
            if let Some(v) = counter.iter().max_by_key(|(_, v)| **v).map(|(k, _)| k) {
                *v
            } else {
                return false;
            };

        // Encode the pair
        let mut s = String::new();
        self.decode_unit(best.0, &mut s);
        self.decode_unit(best.1, &mut s);
        let code = self.push(s);
        let mut i = 0;
        while i < dat.len() {
            if i != dat.len() - 1 && (dat[i], dat[i + 1]) == best {
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
            dat.push(self.push(c.to_string()));
        }
        dat
    }

    pub fn decode(&self, data: impl Iterator<Item = u32>, s: &mut String) {
        for x in data {
            self.decode_unit(x, s)
        }
    }

    fn decode_unit(&self, x: u32, s: &mut String) {
        s.push_str(self.table[x as usize].as_str())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut ret = vec![];
        for s in &self.table {
            ret.extend(s.as_bytes());
            ret.push(0);
        }
        ret
    }
    pub fn from_bytes(mut bytes: impl Iterator<Item = u8>) -> Self {
        let mut table = vec![];
        let mut inv = HashMap::new();
        loop {
            let dat = bytes.by_ref().take_while(|c| *c != 0).collect::<Vec<_>>();
            if dat.len() == 0 {
                break;
            }
            let s = std::str::from_utf8(&dat).unwrap().to_string();
            inv.insert(s.clone(), table.len() as u32);
            table.push(s);
        }
        Self { table, inv }
    }
}

const B7: u32 = (1 << 7) - 1;
const P8: u8 = 1 << 7;

fn codes_to_bytes(codes: &[u32]) -> Vec<u8> {
    let mut ret = vec![];
    for &code in codes {
        ret.push((code & B7) as u8);
        let mut code = code >> 7;
        while code > 0 {
            ret.push((code & B7) as u8 | P8);
            code >>= 7;
        }
    }
    ret
}

fn bytes_to_codes(bytes: &[u8]) -> Vec<u32> {
    let mut ret: Vec<u32> = vec![];
    let mut first = true;
    let mut cur = 0;
    for &byte in bytes {
        if byte & P8 == 0 {
            if first {
                first = false
            } else {
                ret.push(cur);
            }
            cur = byte as u32;
        } else {
            cur <<= 7;
            cur |= (byte & !P8) as u32;
        }
    }
    if !first {
        ret.push(cur);
    }
    ret
}

#[derive(Clap)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCmd,
}

#[derive(Clap)]
enum SubCmd {
    Encode(CmdEncode),
    Decode(CmdDecode),
}
#[derive(Clap)]
struct CmdEncode {
    #[clap(short, long)]
    nstep: u32,
    #[clap(short, long, default_value = "out.bin")]
    out: String,
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    input: String,
}

#[derive(Clap)]
struct CmdDecode {
    #[clap(short, long, default_value = "vocab.bpe")]
    vocab_path: String,
    #[clap(short, long, default_value = "output.txt")]
    out: String,
    input: String,
}

fn main() -> io::Result<()> {
    let opts: Opts = Opts::parse();

    match opts.subcmd {
        SubCmd::Encode(opts) => {
            // read from file
            let mut text = String::new();
            File::open(&opts.input)?.read_to_string(&mut text)?;

            // encode
            let mut vocab = Vocab::default();
            let codes = vocab.encode(&text, EncodeOpt::NTimes(opts.nstep));

            // output
            File::create(opts.vocab_path)?.write_all(&vocab.as_bytes())?;
            File::create(opts.out)?.write_all(&codes_to_bytes(&codes))?;
            Ok(())
        }
        SubCmd::Decode(opts) => {
            let vocab =
                Vocab::from_bytes(File::open(&opts.vocab_path)?.bytes().map(|x| x.unwrap()));
            let codes = File::open(&opts.input)?
                .bytes()
                .map(|x| x.unwrap())
                .collect::<Vec<_>>();
            let codes = bytes_to_codes(&codes);
            let mut s = String::new();
            vocab.decode(codes.into_iter(), &mut s);

            // output
            File::create(opts.out)?.write_all(s.as_bytes())?;
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bytes_and_codes_handmade() {
        let cases = vec![(vec![0, 22], vec![0, 22]), (vec![0], vec![0])];
        for (codes, expected) in cases {
            let bytes = codes_to_bytes(&codes);
            assert_eq!(bytes, expected);
            assert_eq!(bytes_to_codes(&bytes), codes);
        }
    }

    #[quickcheck]
    fn test_bytes_and_codes_quick(codes: Vec<u32>) {
        assert_eq!(codes, bytes_to_codes(&codes_to_bytes(&codes)));
    }
}
