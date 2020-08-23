use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash, Copy)]
enum Data {
    Prime(char),
    Pair((u32, u32)),
}
use Data::*;

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

    pub fn encode(&mut self, text: &str) -> Vec<u32> {
        let n = 10;

        let mut dat = self.encode_first_step(text);

        let mut counter = HashMap::new();
        let mut new_dat = vec![];
        for _ in 0..n {
            self.encode_step(&mut dat, &mut new_dat, &mut counter)
        }
        dat
    }

    fn encode_step<'a>(
        &mut self,
        dat: &'a mut Vec<u32>,
        new_dat: &'a mut Vec<u32>,
        counter: &mut HashMap<Data, u32>,
    ) {
        new_dat.clear();
        counter.clear();

        // Find most frequently occuring pair
        for (&s, &t) in dat.iter().zip(dat.iter().skip(1)) {
            let c = Pair((s, t));
            *counter.entry(c).or_insert(0) += 1;
        }
        let best: Data = *counter
            .iter()
            .max_by_key(|(_, v)| **v)
            .map(|(k, _)| k)
            .unwrap();

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

fn main() {
    let text = "Hello, world!";
    let mut vocab = Vocab::default();
    let dat = vocab.encode(text);
    println!("{:?}", dat);
    let mut s = String::new();
    vocab.decode(&dat, &mut s);
    println!("{}", s);
}
