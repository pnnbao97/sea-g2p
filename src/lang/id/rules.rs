//! Rule-based Indonesian grapheme-to-phoneme: the out-of-vocabulary fallback.
//!
//! Indonesian spelling is close to phonemic, so rules carry most words on
//! their own. Measured against the 17.9k-word WikiPron gold lexicon: 74.4%
//! of words exactly right, and the misses are concentrated in one feature.
//!
//! # The one thing spelling does not say
//!
//! ⟨e⟩ writes both /ə/ and /e/, and nothing distinguishes them: /ə/ in 7,449
//! gold syllables against /e/ in 2,018. The default here is schwa, which is
//! right about two thirds of the time; the dictionary carries the exceptions,
//! sourced from KBBI, whose `pelafalan` field marks the schwa as ⟨ê⟩.
//!
//! Machine G2P does not help: espeak-ng scores 76.5% on the same lexicon, and
//! its errors run in *both* directions on the schwa, so generating entries
//! with it would bake the coin flip into the data.
//!
//! # Pre-1972 spellings
//!
//! The 1972 reform changed dj->j, tj->c, sj->sy, ch->kh and oe->u, but names
//! kept the old forms: Soeharto, Djakarta, Gadjah Mada, Achmad. Proper names
//! are 77% of what the dictionary does not cover, so these are read too.
//!
//! `nj` is deliberately NOT among them. In modern spelling it is nearly
//! always a morpheme boundary — menjadi is meN- + jadi, /mən.d͡ʒa.di/, not
//! /mə.ɲa.di/ — and adding it cost 0.89 points of gold accuracy against a
//! handful of old-spelling names.

/// Consonant digraphs, longest first so `ngg` beats `ng` and `ng` beats `n`.
const DIGRAPHS: &[(&str, &str)] = &[
    ("ngg", "ŋɡ"),
    ("ng", "ŋ"),
    ("ny", "ɲ"),
    ("sy", "ʃ"),
    ("kh", "x"),
    // pre-1972 spellings, kept alive by proper names
    ("dj", "d͡ʒ"),
    ("tj", "t͡ʃ"),
    ("sj", "ʃ"),
    ("ch", "x"),
    ("oe", "u"),
];

/// ⟨ai⟩ and ⟨au⟩ are diphthongs only at the END of a word. Counted over the
/// gold lexicon: word-final "ai" takes the glide 154 times against 39 in
/// hiatus, and "au" 72 against 3 — but before a consonant the counts invert,
/// 124 hiatus against 53, because those are two syllables (ma-in, ka-in,
/// la-in). Applying the diphthong everywhere read "main" as /majn/.
const DIPHTHONGS: &[(&str, &str)] = &[("ai", "aj"), ("au", "aw"), ("oi", "oj")];

fn single(c: char) -> Option<&'static str> {
    Some(match c {
        'a' => "a", 'i' => "i", 'u' => "u", 'o' => "o",
        'b' => "b", 'c' => "t͡ʃ", 'd' => "d", 'f' => "f", 'g' => "ɡ",
        'h' => "h", 'j' => "d͡ʒ", 'k' => "k", 'l' => "l", 'm' => "m",
        'n' => "n", 'p' => "p", 'q' => "k", 'r' => "r", 's' => "s",
        't' => "t", 'v' => "v", 'w' => "w", 'y' => "j", 'z' => "z",
        'x' => "ks",
        _ => return None,
    })
}

/// Every phoneme this module can emit, longest first so the tie-bar
/// affricates are matched before their first character.
const INVENTORY: &[&str] = &[
    "t͡ʃ", "d͡ʒ", "ŋ", "ɲ", "ʃ", "ɡ", "ʔ", "ə",
    "a", "i", "u", "e", "o",
    "p", "b", "t", "d", "k", "f", "v", "s", "z", "x", "h", "m", "n",
    "l", "r", "w", "j",
];

/// Split a mapping such as "ŋɡ", "ks" or "aj" into individual phonemes.
/// Writing the mappings as strings keeps the tables readable; this is what
/// turns them back into the phoneme sequence the caller needs.
fn push_phones(out: &mut Vec<String>, s: &str) {
    let mut rest = s;
    while !rest.is_empty() {
        match INVENTORY.iter().find(|p| rest.starts_with(**p)) {
            Some(p) => {
                out.push((*p).to_string());
                rest = &rest[p.len()..];
            }
            None => {
                // unknown symbol: skip one character rather than loop
                let mut it = rest.chars();
                it.next();
                rest = it.as_str();
            }
        }
    }
}

/// Read an Indonesian word by rule.
///
/// `e_is_e` holds the indices — counting only ⟨e⟩ letters, left to right — of
/// the ⟨e⟩s that are /e/; every other ⟨e⟩ is read /ə/.
pub fn g2p_word_with(word: &str, e_is_e: &[usize]) -> Vec<String> {
    let w: Vec<char> = word.to_lowercase().chars().collect();
    let mut out = Vec::with_capacity(w.len());
    let mut i = 0;
    let mut e_index = 0;
    while i < w.len() {
        if let Some((phones, len)) = match_at(&w, i) {
            push_phones(&mut out, phones);
            i += len;
            continue;
        }
        let c = w[i];
        if c == 'e' {
            out.push(if e_is_e.contains(&e_index) { "e".into() } else { "ə".into() });
            e_index += 1;
        } else if let Some(p) = single(c) {
            push_phones(&mut out, p);
        }
        i += 1;
    }
    out
}

/// [`g2p_word_with`] with every ⟨e⟩ read as schwa.
pub fn g2p_word(word: &str) -> Vec<String> {
    g2p_word_with(word, &[])
}

fn match_at(w: &[char], i: usize) -> Option<(&'static str, usize)> {
    for (src, dst) in DIPHTHONGS {
        let n = src.chars().count();
        if starts_with(w, i, src) && i + n == w.len() {
            return Some((dst, n));
        }
    }
    for (src, dst) in DIGRAPHS {
        if starts_with(w, i, src) {
            return Some((dst, src.chars().count()));
        }
    }
    None
}

fn starts_with(w: &[char], i: usize, s: &str) -> bool {
    let mut k = i;
    for c in s.chars() {
        if w.get(k) != Some(&c) {
            return false;
        }
        k += 1;
    }
    true
}
