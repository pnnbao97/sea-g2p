//! Thai word segmentation and spelling normalization.
//!
//! Thai is written without spaces, so every downstream stage depends on this
//! one. Two passes:
//!
//! 1. **Spelling normalization** — real corpora carry sequences that are
//!    invalid orthography and must be folded before anything else: decomposed
//!    sara am, แ typed as two เ, and tone marks typed out of order. See
//!    [`normalize_spelling`].
//! 2. **Segmentation** — TCC (Thai Character Cluster) grouping to fix the
//!    boundaries no split may cross, then a dynamic program over those
//!    boundaries that minimises the total **unigram cost** `-ln P(word)` over
//!    a trie of dictionary words, charging unknown runs per character.
//!    Frequency weighting is what settles genuine ambiguities: under a plain
//!    "fewest pieces" objective สากลคน ties between สากล|คน and สาก|ลคน and
//!    picks the wrong one. Measured boundary F1 against PyThaiNLP `newmm`
//!    rose from 0.983 to 0.988, and fully-identical runs from 90.3% to 93.7%
//!    (see `thai/README.md`).
//!
//! Non-Thai runs (Latin, digits, punctuation) are natural boundaries and pass
//! through untouched — that is where English code-switching hooks in.

use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;

/// Vowels written *before* the consonant they are pronounced after.
const PREPOSED: [char; 5] = ['เ', 'แ', 'โ', 'ใ', 'ไ'];

/// Marks that can never begin a cluster: they attach to a preceding consonant.
fn is_dependent(c: char) -> bool {
    matches!(c,
        'ะ' | 'ั' | 'า' | 'ำ' | 'ิ' | 'ี' | 'ึ' | 'ื' | 'ุ' | 'ู' | '็'
        | '่' | '้' | '๊' | '๋' | '์' | 'ํ' | '๎' | 'ฺ')
}

pub fn is_thai(c: char) -> bool {
    ('\u{0E01}'..='\u{0E4E}').contains(&c)
}

static RE_SARA_AM_TONE: Lazy<Regex> = Lazy::new(|| Regex::new(r"ํ([่้๊๋])า").unwrap());
static RE_TONE_AFTER_AA: Lazy<Regex> = Lazy::new(|| Regex::new(r"([าำ])([่้๊๋])").unwrap());
static RE_TONE_BEFORE_VOWEL: Lazy<Regex> = Lazy::new(|| Regex::new(r"([่้๊๋])([ิีึืัุู็])").unwrap());

/// Fold spelling quirks found in real wiki, news and web corpora.
///
/// Every rewrite here targets a sequence that is *impossible* in correct Thai
/// orthography, which is what makes the swaps safe:
///
/// - decomposed sara am: nikhahit ํ + า -> ำ (ทํา -> ทำ), including the
///   variant with a tone mark wedged in (นํ้า -> น้ำ);
/// - แ typed as two เ (เเละ -> และ);
/// - tone typed after า/ำ (นำ้ -> น้ำ) or before an above/below vowel
///   (ท่ี -> ที่).
pub fn normalize_spelling(text: &str) -> String {
    let mut s = RE_SARA_AM_TONE.replace_all(text, "${1}ำ").into_owned();
    s = s.replace("ํา", "ำ");
    s = s.replace("เเ", "แ");
    s = RE_TONE_AFTER_AA.replace_all(&s, "$2$1").into_owned();
    s = RE_TONE_BEFORE_VOWEL.replace_all(&s, "$2$1").into_owned();
    s
}

/// Split a run of Thai text into clusters no boundary may fall inside:
/// `[preposed vowel]* consonant [dependent mark]*`. Returns byte offsets of
/// the cluster edges, starting at 0 and ending at `s.len()`.
pub fn tcc_bounds(s: &str) -> Vec<usize> {
    let mut bounds = vec![0usize];
    let chars: Vec<(usize, char)> = s.char_indices().collect();
    let n = chars.len();
    let mut i = 0;
    while i < n {
        let mut j = i;
        while j < n && PREPOSED.contains(&chars[j].1) {
            j += 1;
        }
        if j < n {
            j += 1; // the base consonant
        }
        while j < n && is_dependent(chars[j].1) {
            j += 1;
        }
        let end = if j < n { chars[j].0 } else { s.len() };
        bounds.push(end);
        i = j;
    }
    bounds
}

#[derive(Default)]
struct Node {
    children: HashMap<char, Node>,
    /// `-ln(P(word))` for a word ending here; `None` if no word does.
    cost: Option<f32>,
}

/// Prefix trie of dictionary words carrying each word's unigram cost.
///
/// The cost is what makes segmentation prefer the *likely* parse rather than
/// merely the parse with fewest pieces. Under the old "fewest words" rule
/// สากลคน tied between สากล|คน and สาก|ลคน and lost the tie; weighted by
/// corpus frequency it is not close.
#[derive(Default)]
pub struct WordTrie {
    root: Node,
    /// Cost charged per character of a run no word covers. Must exceed any
    /// real word's cost so unknown text is a last resort.
    unknown_per_char: f32,
}

impl WordTrie {
    pub fn new() -> Self {
        Self { root: Node::default(), unknown_per_char: 30.0 }
    }

    pub fn insert_with_cost(&mut self, word: &str, cost: f32) {
        let mut node = &mut self.root;
        for c in word.chars() {
            node = node.children.entry(c).or_default();
        }
        node.cost = Some(match node.cost {
            Some(prev) if prev < cost => prev,
            _ => cost,
        });
    }

    pub fn insert(&mut self, word: &str) {
        self.insert_with_cost(word, 1.0);
    }

    /// Build from words alone: every word costs the same, which reproduces
    /// the older "fewest pieces" objective. Used by tests and by any caller
    /// without frequency data.
    pub fn from_words<I: IntoIterator<Item = S>, S: AsRef<str>>(words: I) -> Self {
        let mut t = Self::new();
        for w in words {
            t.insert(w.as_ref());
        }
        t
    }

    /// Build from `(word, corpus_count)` pairs. Costs are `-ln(count/total)`,
    /// and the unknown-run charge is set just above the cost of the rarest
    /// possible word so that unknown text never wins against a real one.
    pub fn from_frequencies<I: IntoIterator<Item = (S, u64)>, S: AsRef<str>>(pairs: I) -> Self {
        let pairs: Vec<(S, u64)> = pairs.into_iter().collect();
        let total: u64 = pairs.iter().map(|(_, n)| (*n).max(1)).sum::<u64>().max(1);
        let ln_total = (total as f32).ln();
        let mut t = Self::new();
        for (w, n) in &pairs {
            let cost = ln_total - ((*n).max(1) as f32).ln();
            t.insert_with_cost(w.as_ref(), cost);
        }
        t.unknown_per_char = ln_total * 1.5;
        t
    }
}

/// A segmented token. `known` is false for a run no dictionary word covered,
/// and `None` for a non-Thai run passed through verbatim.
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub text: String,
    pub known: Option<bool>,
}

fn segment_thai_run(s: &str, trie: &WordTrie) -> Vec<Token> {
    let bounds = tcc_bounds(s);
    let n = bounds.len() - 1; // cluster count
    let idx_of: HashMap<usize, usize> = bounds.iter().enumerate().map(|(i, b)| (*b, i)).collect();

    let mut best = vec![f32::INFINITY; n + 1];
    let mut back: Vec<Option<(usize, usize, bool)>> = vec![None; n + 1]; // (prev, byte_start, known)
    best[0] = 0.0;

    for k in 0..n {
        if !best[k].is_finite() {
            continue;
        }
        let start = bounds[k];
        // dictionary walk from this boundary; only accept matches that land
        // on another cluster boundary
        let mut node = &trie.root;
        for (off, c) in s[start..].char_indices() {
            match node.children.get(&c) {
                Some(next) => node = next,
                None => break,
            }
            let pos = start + off + c.len_utf8();
            if let Some(word_cost) = node.cost {
                if let Some(&k2) = idx_of.get(&pos) {
                    let cand = best[k] + word_cost;
                    if cand < best[k2] {
                        best[k2] = cand;
                        back[k2] = Some((k, start, true));
                    }
                }
            }
        }
        // or take one cluster as unknown
        let cluster_chars = s[start..bounds[k + 1]].chars().count() as f32;
        let cand = best[k] + trie.unknown_per_char * cluster_chars;
        if cand < best[k + 1] {
            best[k + 1] = cand;
            back[k + 1] = Some((k, start, false));
        }
    }

    let mut pieces: Vec<Token> = Vec::new();
    let mut k = n;
    while k > 0 {
        let (prev, start, known) = back[k].expect("reachable by construction");
        pieces.push(Token { text: s[start..bounds[k]].to_string(), known: Some(known) });
        k = prev;
    }
    pieces.reverse();

    // merge neighbouring unknown chunks into one token
    let mut out: Vec<Token> = Vec::with_capacity(pieces.len());
    for p in pieces {
        match out.last_mut() {
            Some(last) if last.known == Some(false) && p.known == Some(false) => {
                last.text.push_str(&p.text);
            }
            _ => out.push(p),
        }
    }
    out
}

/// Segment mixed text. Thai runs are split with the dictionary; everything
/// else is emitted verbatim with `known: None`.
pub fn segment(text: &str, trie: &WordTrie) -> Vec<Token> {
    let text = normalize_spelling(text);
    let mut out = Vec::new();
    let mut buf = String::new();
    let mut buf_is_thai = false;

    let flush = |buf: &mut String, is_thai: bool, out: &mut Vec<Token>| {
        if buf.is_empty() {
            return;
        }
        if is_thai {
            out.extend(segment_thai_run(buf, trie));
        } else {
            out.push(Token { text: std::mem::take(buf), known: None });
            return;
        }
        buf.clear();
    };

    for c in text.chars() {
        // ๆ (repeat) and ฯ (abbreviation) are their own tokens: expanding
        // them is the normalizer's job, exactly like Vietnamese "v.v".
        if c == 'ๆ' || c == 'ฯ' {
            flush(&mut buf, buf_is_thai, &mut out);
            out.push(Token { text: c.to_string(), known: Some(true) });
            continue;
        }
        let t = is_thai(c);
        if t != buf_is_thai {
            flush(&mut buf, buf_is_thai, &mut out);
            buf_is_thai = t;
        }
        buf.push(c);
    }
    flush(&mut buf, buf_is_thai, &mut out);
    out
}
