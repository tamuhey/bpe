use unicode_normalization::UnicodeNormalization;

pub const SPACE_REP: char = '\u{2581}';

/// 1. normalize wiht NFKD
/// 2. replace whitespace to U+2581
pub fn to_chars(s: &str) -> Vec<char> {
    s.nfkd()
        .map(|c| if c == ' ' { SPACE_REP } else { c })
        .collect()
}
