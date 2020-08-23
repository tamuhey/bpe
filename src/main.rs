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
}

fn main() {
    let _text = "Hello, world!";
    let mut vocab = Vocab::default();
    let mut text: Vec<u32> = vec![];
    for c in _text.chars() {
        let c = Prime(c);
        text.push(vocab.push(c));
    }

    let n = 10;
    let mut counter = HashMap::new();
    let mut new_text = vec![];
    for _ in 0..n {
        for (&s, &t) in text.iter().zip(text.iter().skip(1)) {
            let c = Pair((s, t));
            *counter.entry(c).or_insert(0usize) += 1;
        }
        let best = *counter
            .iter()
            .max_by_key(|(_, v)| **v)
            .map(|(k, _)| k)
            .unwrap();
        let besti = vocab.push(best);
        let mut i = 0;
        while i < text.len() {
            if i != text.len() - 1 && Pair((text[i], text[i + 1])) == best {
                new_text.push(besti);
                i += 1;
            } else {
                new_text.push(text[i]);
            }
            i += 1;
        }
        let tmp = text;
        text = new_text;
        new_text = tmp;
        new_text.clear();
        counter.clear();
        println!("{:?}", text);
    }

    let mut s = String::new();
    vocab.decode(&text, &mut s);
    println!("{}", s);
}
