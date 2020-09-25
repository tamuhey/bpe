use crate::spec::TrainSpec;
use unicode_normalization::UnicodeNormalization;

pub const SPACE_REP: char = '\u{2581}';

/// 1. normalize wiht NFKD
/// 2. replace whitespace to U+2581
pub fn to_chars(mut s: &str, spec: &TrainSpec) -> Vec<char> {
    let mut ret = vec![SPACE_REP];
    let mut is_prev_space = !spec.keep_extra_whitespaces;

    if !spec.keep_extra_whitespaces {
        s = s.trim();
    }

    for c in s.nfkd() {
        if c.is_whitespace() {
            if !is_prev_space {
                ret.push(SPACE_REP);
                is_prev_space = !spec.keep_extra_whitespaces;
            }
        } else {
            ret.push(c);
            is_prev_space = false;
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_to_chars() {
        let mut spec = TrainSpec::default();
        spec.keep_extra_whitespaces = false;
        let s = "  ab \t c\td  ";
        assert_eq!(
            to_chars(s, &spec),
            vec![SPACE_REP, 'a', 'b', SPACE_REP, 'c', SPACE_REP, 'd']
        );
    }
}
