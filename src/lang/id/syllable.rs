//! Syllabification of an Indonesian phoneme sequence.
//!
//! The library's output convention is **one space per syllable boundary**:
//! Vietnamese emits `t̪ˈoj ɗˈi hˈɔ6k` and Thai `kʰaw˩˩˦ tɕʰa˨˩ laːt̚˨˩`, both
//! one group per syllable. Indonesian phonemes arrive from the dictionary one
//! per token, so without this pass `saya makan` came out as nine groups
//! instead of four — the same library speaking two formats, which forces
//! every downstream consumer to special-case the language.
//!
//! # The rule
//!
//! An Indonesian syllable is (C)(C)V(C). Between two vowels:
//!
//!   - no consonant  -> the vowels are separate syllables (di-a, ma-in);
//!   - one consonant -> it opens the next syllable (ba-ca, sa-ya);
//!   - two or more   -> the last one opens the next syllable (man-di,
//!     bang-ku), unless the final two form a valid onset cluster, which in
//!     Indonesian happens only in loans (pra-, kla-, stra-).
//!
//! A glide needs no special case: in `sa.ja` the /j/ sits between vowels and
//! opens a syllable, while in `pan.daj` it trails the last vowel and closes
//! one.

/// Onset clusters Indonesian permits, all from Dutch, English or Sanskrit
/// loans. Native roots have no complex onsets.
const CLUSTERS: &[(&str, &str)] = &[
    ("p", "r"), ("p", "l"), ("b", "r"), ("b", "l"),
    ("t", "r"), ("d", "r"), ("k", "r"), ("k", "l"),
    ("ɡ", "r"), ("ɡ", "l"), ("f", "r"), ("f", "l"),
    ("s", "p"), ("s", "t"), ("s", "k"), ("s", "w"), ("s", "r"),
];

fn is_vowel(p: &str) -> bool {
    matches!(p, "a" | "i" | "u" | "e" | "o" | "ə")
}

fn is_cluster(a: &str, b: &str) -> bool {
    CLUSTERS.iter().any(|(x, y)| *x == a && *y == b)
}

/// Group a phoneme sequence into syllables, each returned as one string.
pub fn syllabify(phones: &[String]) -> Vec<String> {
    let nuclei: Vec<usize> = phones
        .iter()
        .enumerate()
        .filter(|(_, p)| is_vowel(p))
        .map(|(i, _)| i)
        .collect();
    if nuclei.len() <= 1 {
        return vec![phones.concat()];
    }

    // A syllable starts where the previous one ends; the first begins at 0
    // and the last runs to the end of the word.
    let mut starts = vec![0usize];
    for pair in nuclei.windows(2) {
        let (v1, v2) = (pair[0], pair[1]);
        let gap = v2 - v1 - 1; // consonants between the two nuclei
        let start = match gap {
            0 => v2,                       // di-a
            1 => v2 - 1,                   // ba-ca
            _ => {
                let (a, b) = (&phones[v2 - 2], &phones[v2 - 1]);
                if is_cluster(a, b) {
                    v2 - 2                 // in-fra
                } else {
                    v2 - 1                 // man-di
                }
            }
        };
        starts.push(start);
    }

    let mut out = Vec::with_capacity(starts.len());
    for (i, &s) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(phones.len());
        out.push(phones[s..end].concat());
    }
    out
}
