use unicode_normalization::UnicodeNormalization;

/// 1. normalize wiht NFKD
/// 2. replace whitespace to U+2581
pub fn to_chars(s: &str) -> Vec<char> {
    s.nfkd()
        .map(|c| if c == ' ' { '\u{2581}' } else { c })
        .collect()
}
