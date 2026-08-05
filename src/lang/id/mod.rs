//! Indonesian: normalization and pronunciation.
//!
//! The pipeline is the simplest of the three languages, because the script
//! is: Latin with spaces, so no segmentation stage is needed.
//!
//!   raw text -> normalize (6 stages) -> per-token lookup -> rules for the rest
//!            -> syllabify
//!
//! The last step is what keeps the output format the same across languages:
//! one space per syllable boundary, as Vietnamese and Thai already emit.
//!
//! Pronunciation comes from the dictionary section in the phoneme binary
//! (`SECTION_ID`, 172k words covering 86% of corpus tokens), built from the
//! WikiPron gold lexicon, KBBI's `pelafalan` field, affix analysis and
//! compound splitting — see `indo/` for how, and why no machine G2P was used
//! to generate entries.
//!
//! # Why Indonesian words cannot share the English table
//!
//! "air", "dia", "media" and "an" are words in both languages with different
//! readings, so a single Latin keyspace cannot hold both. The Indonesian
//! entries therefore live in their own section, and a token is resolved
//! against the language the caller asked for; anything the Indonesian
//! dictionary does not have and the rules should not claim — English words in
//! a code-switched sentence — goes to the English engine through the same
//! callback the Thai module uses.

pub mod normalizer;
pub mod num2id;
pub mod resources;
pub mod rules;
pub mod syllable;

use crate::core::dict::{PhonemeDict, SECTION_ID};

/// Indonesian front end.
pub struct Indonesian;

impl Indonesian {
    pub fn new() -> Self {
        Self
    }

    /// Phonemize Indonesian text, sending tokens the dictionary does not have
    /// to `read_latin` when they look English, and to the rules otherwise.
    pub fn phonemize_with<F>(&self, text: &str, dict: &PhonemeDict, read_latin: F) -> String
    where
        F: Fn(&str) -> String,
    {
        let text = normalizer::normalize(text);
        let mut out: Vec<String> = Vec::new();
        for tok in text.split_whitespace() {
            let word: String = tok
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '\'')
                .collect::<String>()
                .to_lowercase();
            let trailing: String = tok
                .chars()
                .rev()
                .take_while(|c| matches!(c, ',' | '.' | '!' | '?'))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect();
            if word.is_empty() {
                if !trailing.is_empty() {
                    attach(&mut out, &trailing);
                }
                continue;
            }
            let phones = match dict.lookup_section(SECTION_ID, &word) {
                // Dictionary entries are stored one phoneme per token; the
                // output convention is one group per syllable, as it is for
                // Vietnamese and Thai.
                Some(p) => {
                    let seq: Vec<String> = p.split_whitespace().map(str::to_string).collect();
                    syllable::syllabify(&seq).join(" ")
                }
                // Not Indonesian vocabulary. Route to English only when the
                // English DICTIONARY has the word — asking the engine instead
                // always says yes, since it segments any unknown string, and
                // that sent the Indonesian name Gadjah to be read "gad jah".
                None if dict.has_english(&word) => read_latin(&word),
                // Otherwise the rules read it, which is what carries proper
                // names — 77% of the words the dictionary does not cover.
                None => syllable::syllabify(&rules::g2p_word(&word)).join(" "),
            };
            out.push(phones);
            if !trailing.is_empty() {
                attach(&mut out, &trailing);
            }
        }
        out.join(" ")
    }

    /// [`phonemize_with`] with no English reader: unknown tokens are read by
    /// rule. Used for pure-Indonesian input and by tests.
    pub fn phonemize(&self, text: &str, dict: &PhonemeDict) -> String {
        self.phonemize_with(text, dict, |_| String::new())
    }
}

impl Default for Indonesian {
    fn default() -> Self {
        Self::new()
    }
}

/// Punctuation attaches to the phoneme group before it, the convention the
/// Vietnamese and Thai front ends already use.
fn attach(out: &mut Vec<String>, mark: &str) {
    match out.last_mut() {
        Some(last) => last.push_str(mark),
        None => out.push(mark.to_string()),
    }
}
