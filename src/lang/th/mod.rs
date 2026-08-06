//! Thai: segmentation, spelling normalization and pronunciation.
//!
//! The pipeline differs in shape from the Vietnamese one because the script
//! does: Thai has no spaces, so text must be **segmented before anything
//! else can look a word up**.
//!
//!   raw text -> normalize (numbers, dates, abbreviations, symbols)
//!            -> segment -> per-token pronunciation
//!
//! Pronunciation comes from the dictionary section built into the phoneme
//! binary (`SECTION_TH`, 44k words covering >99% of corpus tokens). Words
//! the dictionary does not have — new names, transliterations, typos — are
//! read by rule; the reference implementation of those rules lives in
//! `thai/rule_g2p.py` and is being ported here.

pub mod normalizer;
pub mod num2th;
pub mod resources;
pub mod rules;
pub mod segment;

use crate::core::dict::{PhonemeDict, SECTION_TH, SECTION_TH_FREQ};
use segment::{segment as split, Token, WordTrie};

/// Join phoneme groups with spaces, but attach a punctuation mark to the
/// group before it. Without this "…maj˩˩˦ ?" leaves the mark as its own
/// whitespace token while the Vietnamese engine emits "…ɗˈiɛ4m." — a shared
/// TTS front end would then see two conventions from one library.
fn join_phonemes(parts: &[String]) -> String {
    let mut out = String::new();
    for p in parts {
        let is_punct = !p.is_empty() && p.chars().all(|c| matches!(c, ',' | '.' | '!' | '?'));
        if !out.is_empty() && !is_punct {
            out.push(' ');
        }
        out.push_str(p);
    }
    out
}

/// Thai front-end: owns the word trie built from the dictionary section.
pub struct Thai {
    trie: WordTrie,
}

impl Thai {
    /// Build the segmenter wordlist from the dictionary's Thai section, so
    /// the segmenter and the pronunciation table can never drift apart, and
    /// weight it with the corpus frequencies stored alongside.
    ///
    /// Without frequencies the objective degenerates to "fewest pieces",
    /// which ties on real ambiguities (สากล|คน vs สาก|ลคน) and picks wrong.
    pub fn new(dict: &PhonemeDict) -> Self {
        let freqs = dict.section_entries(SECTION_TH_FREQ);
        if freqs.is_empty() {
            return Self { trie: WordTrie::from_words(dict.section_keys(SECTION_TH)) };
        }
        // The 2,767 keys carrying an empty pronunciation stay in the
        // segmenter's vocabulary on purpose. Dropping them looks right — a
        // word the pipeline cannot pronounce is a poor segmentation target,
        // and 98.5% of them are absent from PyThaiNLP's human-compiled
        // `thai_words()`, so they really are fragments of transliterated
        // names (มปิก from โอลิมปิก, ทเทิล from Seattle). Two independent
        // judges still say leave them:
        //
        //                    F1 vs newmm   curated kept whole   F1 vs BEST2009
        //   keep (current)   0.9854        74.76%               0.8702
        //   drop from trie   0.9850        74.69%               0.8700
        //
        // The first judge is compromised — `thai/build/freq_count.py` counts
        // with newmm, so newmm produced these fragments and agreeing with it
        // cannot penalise them. The third is not: BEST2009 is human-annotated
        // (`nectec/best2009` on Hugging Face, train split, 4,800 records over
        // its four genres), and it says the same thing to four decimal places.
        // All three reject the change: the fragments almost never win a
        // segmentation, because the cost model already prefers a whole word
        // wherever one exists. Their harm was in the pronunciation lookup,
        // and that is fixed in `phonemize_with`, one layer down.
        let pairs = freqs
            .into_iter()
            .map(|(w, n)| (w, n.parse::<u64>().unwrap_or(1)));
        Self { trie: WordTrie::from_frequencies(pairs) }
    }

    pub fn segment(&self, text: &str) -> Vec<Token> {
        split(text, &self.trie)
    }

    /// Phonemes of the last dictionary word inside `word`, when `word` is a
    /// compound of two dictionary words. `None` when it is atomic — the
    /// caller then repeats the token whole.
    fn final_word(word: &str, dict: &PhonemeDict) -> Option<String> {
        let chars: Vec<char> = word.chars().collect();
        // Both halves must be at least two characters: a single Thai letter
        // is in the dictionary as a *letter name*, not as a word, and
        // accepting it splits ต่าง into ต่า + ง and repeats "ngo".
        const MIN: usize = 2;
        if chars.len() < MIN * 2 {
            return None;
        }
        for split in MIN..=chars.len() - MIN {
            let prefix: String = chars[..split].iter().collect();
            let suffix: String = chars[split..].iter().collect();
            if dict.lookup_section(SECTION_TH, &prefix).is_some() {
                if let Some(p) = dict.lookup_section(SECTION_TH, &suffix) {
                    return Some(p.to_string());
                }
            }
        }
        None
    }

    /// Phonemize Thai text: dictionary first, then [`rules`] for anything
    /// it does not have, so every string comes out pronounceable.
    ///
    /// Latin runs are handed to `read_latin`. Thai text code-switches with
    /// English constantly — brand names, tech terms, whole English clauses —
    /// and passing those through verbatim leaves raw letters in the phoneme
    /// string. The callback keeps this module free of a dependency on the
    /// Latin-script engine; [`crate::G2P`] wires the two together.
    pub fn phonemize_with<F>(&self, text: &str, dict: &PhonemeDict, read_latin: F) -> String
    where
        F: Fn(&str) -> String,
    {
        let text = normalizer::normalize(text);
        let mut out: Vec<String> = Vec::new();
        let mut prev_word: Option<String> = None;
        for tok in self.segment(&text) {
            // ๆ (mai yamok) repeats the word before it. Applied here, not in
            // the normalizer, because only after segmentation is it known
            // where that word starts: คนต่างๆ is คน-ต่าง-ต่าง, not
            // คน-ต่าง-คน-ต่าง.
            //
            // When the preceding token is a lexicalised compound the mark
            // still repeats only its final word — เด็กเล็กๆ is
            // "dèk lék lék", never "dèk-lék dèk-lék" — so a compound is
            // split back into (prefix, last word) when both halves are
            // themselves dictionary words.
            if tok.text == "ๆ" {
                match prev_word.as_deref().and_then(|w| Self::final_word(w, dict)) {
                    Some(tail) => out.push(tail),
                    None => {
                        if let Some(prev) = out.last().cloned() {
                            out.push(prev);
                        }
                    }
                }
                continue;
            }
            // ฯ (paiyannoi) marks an elided remainder — กรุงเทพฯ stands for
            // the full ceremonial name of Bangkok — and is itself silent. No
            // rule reads it, so the "never give up" fallback below pushed the
            // raw character into the phoneme string: กรุงเทพฯ came out as
            // "kruŋ˧ tʰeːp̚˥˩ ฯ", handing a TTS a token no voice was trained
            // on. ฯลฯ is not affected; the normalizer expands it earlier.
            if tok.text.trim() == "ฯ" {
                continue;
            }
            prev_word = Some(tok.text.clone());
            match tok.known {
                None => {
                    let trimmed = tok.text.trim();
                    // Only alphabetic runs are words; pure punctuation or
                    // digits left by the normalizer pass through as they are.
                    if trimmed.chars().any(|c| c.is_alphabetic()) {
                        let read = read_latin(trimmed);
                        if !read.trim().is_empty() {
                            out.push(read);
                        }
                    } else if !trimmed.is_empty() {
                        out.push(trimmed.to_string());
                    }
                }
                Some(_) => {
                    // An entry with an EMPTY pronunciation is not a
                    // pronunciation. 2,767 words in the shipped Thai section
                    // carry one — segmentation fragments of transliterated
                    // names such as มปิก (from โอลิมปิก) or ทเทิล (Seattle) —
                    // and taking the lookup at its word deleted every one of
                    // them from the phoneme stream, 0.08% of corpus tokens
                    // vanishing with nothing to hear. Treat it as a miss and
                    // let the rules read the word, which they can.
                    let listed = dict.lookup_section(SECTION_TH, &tok.text).unwrap_or("");
                    if !listed.trim().is_empty() {
                        out.push(listed.to_string());
                    } else {
                        let syls = rules::g2p_word(&tok.text);
                        if !syls.is_empty() {
                            out.push(syls.join(" "));
                        }
                        // Nothing left to say: every consonant carries
                        // thanthakhat, so the token really is silent (ร์, น์).
                        // Emitting the raw characters here is what put ฯ into
                        // the phonemes — a token no voice was trained on.
                    }
                }
            }
        }
        join_phonemes(&out)
    }

    /// [`phonemize_with`] without a Latin reader: non-Thai runs are kept
    /// verbatim. Useful for pure-Thai input and for tests.
    pub fn phonemize(&self, text: &str, dict: &PhonemeDict) -> String {
        self.phonemize_with(text, dict, |s| s.to_string())
    }
}
