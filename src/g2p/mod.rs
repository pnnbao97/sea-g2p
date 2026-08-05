//! Grapheme-to-phoneme conversion for mixed Vietnamese and English text.
//!
//! # Choosing a language per token
//!
//! Many tokens exist in both dictionaries, so the reading is decided by
//! context. `propagate_language` walks the sentence and attaches each ambiguous
//! token to its nearest unambiguous anchor — a word that can only be Vietnamese
//! or only English. Punctuation does not count toward the distance, so a comma
//! between a word and its anchor changes nothing. On a tie, real words go
//! English while single letters follow the anchor on their right. A sentence
//! made entirely of shared words defaults to English.
//!
//! # Out-of-vocabulary tokens
//!
//! `segment_oov` splits an unknown token by dynamic programming over dictionary
//! pieces, minimizing a cost: 1 per dictionary word, 4 + length for a
//! spelled-out fallback or a junk fragment. Ties are broken by preferring more
//! genuine English words, then the rightmost-longest split, then fewer pieces —
//! which is what makes "fine tune" win over "fin etune".

use std::io;
use regex::Regex;
use once_cell::sync::Lazy;

use crate::lang::en::top_words::EN_TOP_WORDS;

pub use crate::core::dict::PhonemeDict;

static RE_TOKEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(<en>.*?</en>)|(\w+(?:['’]\w+)*)|([^\w\s])").unwrap()
});

static RE_TAG_CONTENT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(\w+(?:['’]\w+)*)|([^\w\s])").unwrap()
});

static RE_TAG_STRIP: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)</?en>").unwrap()
});

static VI_ACCENTS: &str = "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ";

// English and Vietnamese vowels, lowercase, diacritics included.
static VOWELS: &str = "aeiouyàáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ";

/// Does the segment contain both a vowel and a consonant?
///
/// Rejects consonant-only pieces ("n", "st") and vowel-only ones ("e", "a").
/// Vietnamese words made purely of vowels, such as "ơi" and "ừ", are almost
/// always in the dictionary already and never reach `segment_oov`.
fn has_vowel_and_consonant(s: &str) -> bool {
    let mut has_v = false;
    let mut has_c = false;
    for c in s.chars() {
        let lc = c.to_lowercase().next().unwrap_or(c);
        if VOWELS.contains(lc) {
            has_v = true;
        } else if lc.is_alphabetic() {
            has_c = true;
        }
        if has_v && has_c { return true; }
    }
    false
}

/// Map a punctuation token to the form kept in the phoneme string, matching the
/// rules used by `Normalizer`. `None` means drop the mark entirely.
///
/// This is needed because content inside `<en>` tags never passes through
/// `Normalizer` — the normalizer preserves it verbatim — so marks such as `"`,
/// `(` and `-` would leak into the phoneme string without this. The rules mirror
/// `Normalizer`:
///   - `, . ! ?`            -> kept as-is
///   - `; :`                -> `,`
///   - `… ‥ ․` (ellipsis)   -> `.`
///   - everything else — quotes `"` `'` `«` `»`, brackets `(` `)` `{` `}`
///     `[` `]`, free-standing dashes `-` `–` `—` — is dropped
///
/// A punctuation token is always a single character (regex `[^\w\s]`).
fn map_punct(s: &str) -> Option<&'static str> {
    let mut it = s.chars();
    let c = match (it.next(), it.next()) {
        (Some(c), None) => c,
        _ => return None,
    };
    match c {
        ',' => Some(","),
        '.' => Some("."),
        '!' => Some("!"),
        '?' => Some("?"),
        ';' | ':' => Some(","),
        '\u{2026}' | '\u{2025}' | '\u{2024}' => Some("."),
        _ => None,
    }
}

#[derive(Clone)]
pub struct Token {
    pub lang: String,
    pub content: String,
    pub phone: Option<String>,
    pub is_explicit_en: bool,
}

use std::collections::HashMap;
use std::sync::RwLock;

pub struct G2PEngine {
    pub dict: PhonemeDict,
    merged_cache: RwLock<HashMap<String, String>>,
    common_cache: RwLock<HashMap<String, (String, String)>>,
    missing_merged: RwLock<std::collections::HashSet<String>>,
    missing_common: RwLock<std::collections::HashSet<String>>,
    /// Cache of `segment_oov` results, keyed by "{word}_{lang}". A `None` value
    /// records that the word could not be segmented, so the work is not repeated.
    segmentation_cache: RwLock<HashMap<String, Option<String>>>,
}

impl G2PEngine {
    pub fn new(dict_path: &str) -> io::Result<Self> {
        Ok(Self {
            dict: PhonemeDict::new(dict_path)?,
            merged_cache: RwLock::new(HashMap::with_capacity(2048)),
            common_cache: RwLock::new(HashMap::with_capacity(1024)),
            missing_merged: RwLock::new(std::collections::HashSet::new()),
            missing_common: RwLock::new(std::collections::HashSet::new()),
            segmentation_cache: RwLock::new(HashMap::with_capacity(512)),
        })
    }

    fn cached_lookup_merged(&self, word: &str) -> Option<String> {
        {
            let r = self.merged_cache.read().unwrap();
            if let Some(v) = r.get(word) { return Some(v.clone()); }
        }
        {
            let m = self.missing_merged.read().unwrap();
            if m.contains(word) { return None; }
        }
        match self.dict.lookup_merged(word) {
            Some(s) => {
                let val = s.to_string();
                let mut w = self.merged_cache.write().unwrap();
                if w.len() >= 10_000 { w.clear(); }
                w.insert(word.to_string(), val.clone());
                Some(val)
            }
            None => {
                let mut m = self.missing_merged.write().unwrap();
                if m.len() < 50_000 { m.insert(word.to_string()); }
                None
            }
        }
    }

    fn cached_lookup_common(&self, word: &str) -> Option<(String, String)> {
        {
            let r = self.common_cache.read().unwrap();
            if let Some(v) = r.get(word) { return Some(v.clone()); }
        }
        {
            let m = self.missing_common.read().unwrap();
            if m.contains(word) { return None; }
        }
        match self.dict.lookup_common(word) {
            Some((v, e)) => {
                let val = (v.to_string(), e.to_string());
                let mut w = self.common_cache.write().unwrap();
                if w.len() >= 5_000 { w.clear(); }
                w.insert(word.to_string(), val.clone());
                Some(val)
            }
            None => {
                let mut m = self.missing_common.write().unwrap();
                if m.len() < 50_000 { m.insert(word.to_string()); }
                None
            }
        }
    }

    /// Resolve the phonemes of a single segment from the dictionary, honouring
    /// the language context.
    fn resolve_segment_phone(&self, segment: &str, lang: &str) -> Option<String> {
        let lw = segment.to_lowercase();

        if let Some(p) = self.cached_lookup_merged(&lw) {
            return Some(p.replace("<en>", "").trim().to_string());
        }

        if let Some((vi, en)) = self.cached_lookup_common(&lw) {
            let phone = if lang == "en" && !en.is_empty() {
                en.replace("<en>", "").trim().to_string()
            } else if !vi.is_empty() {
                vi.trim().to_string()
            } else {
                en.replace("<en>", "").trim().to_string()
            };
            return Some(phone);
        }

        None
    }

    /// Segment an out-of-vocabulary word by dynamic programming, minimising cost:
    ///   - a segment that is a REAL dictionary word (vowel plus consonant, and
    ///     phonemes that are not a spelled-out form) costs 1, so the search
    ///     naturally prefers few long pieces — "vietinbank" becomes
    ///     "viet in bank" rather than "vi eti en bank";
    ///   - a short segment of at most four characters whose phonemes carry two
    ///     or more stresses is a spelled-out acronym entry ("mbo" -> em bi ô).
    ///     It is priced high and used only when nothing else fits;
    ///   - a run of at most three characters absent from the dictionary may be
    ///     spelled letter by letter, again at a high price ("vpbank" -> spelled
    ///     "vp" plus "bank"; "chunkr" -> "chunk" plus "r"), which still beats
    ///     dropping the whole word to `char_fallback`;
    ///   - on equal cost, prefer the longer LEADING piece (leftmost-longest).
    fn segment_oov(&self, word: &str, lang: &str) -> Option<String> {
        // Consult the cache first.
        let cache_key = format!("{}_{}", word, lang);
        {
            let r = self.segmentation_cache.read().unwrap();
            if let Some(cached) = r.get(&cache_key) {
                return cached.clone();
            }
        }

        const JUNK_COST: u32 = 4;

        #[derive(Clone)]
        struct Path {
            cost: u32,
            top: u32,
            lens: Vec<u8>,
            phones: Vec<String>,
        }
        // True when a beats b. Lower cost first; on a tie, MORE common English
        // words ("fine|tune" beats "fin|etune", "family|app" beats "famil|yapp",
        // because junk dictionary entries are absent from the top wordlist);
        // then a longer FINAL piece ("vin|homes" beats "vinho|mes"); then fewer
        // pieces. A "balanced split" criterion was tried and rejected: English
        // morphemes are not balanced, and it broke "vin|homes" into
        // "vinh|omes".
        fn better(a: &Path, b: &Path) -> bool {
            if a.cost != b.cost { return a.cost < b.cost; }
            if a.top != b.top { return a.top > b.top; }
            for (x, y) in a.lens.iter().rev().zip(b.lens.iter().rev()) {
                if x != y { return x > y; }
            }
            a.lens.len() < b.lens.len()
        }

        let chars: Vec<char> = word.chars().collect();
        let n = chars.len();
        let mut dp: Vec<Option<Path>> = vec![None; n + 1];
        dp[0] = Some(Path { cost: 0, top: 0, lens: Vec::new(), phones: Vec::new() });

        for i in 0..n {
            let Some(base) = dp[i].clone() else { continue };
            for j in (i + 1)..=n {
                let segment: String = chars[i..j].iter().collect();
                let seg_len = j - i;

                let mut phone: Option<String> = None;
                let mut cost = 1u32;
                if has_vowel_and_consonant(&segment) {
                    if let Some(p) = self.resolve_segment_phone(&segment, lang) {
                        let primary = p.matches('ˈ').count();
                        let total = primary + p.matches('ˌ').count();
                        // Junk dictionary entries: a short piece whose phonemes
                        // carry many stresses (a spelled-out form, "mbo" -> em
                        // bi ô), or two or more PRIMARY stresses (a glued entry,
                        // "enbank" -> en-bank). Long real words with a secondary
                        // stress (ˈ plus ˌ) are not caught by this.
                        if (seg_len <= 4 && total >= 2) || primary >= 2 {
                            cost = JUNK_COST + seg_len as u32;
                        }
                        phone = Some(p);
                    }
                }
                if phone.is_none() && seg_len <= 3 {
                    // Spelling letter by letter, priced by length: a single
                    // trailing letter is cheap ("chunk r"), a three-consonant
                    // run mid-word is expensive.
                    let spelled = self.char_fallback(&segment, lang);
                    if !spelled.trim().is_empty() {
                        phone = Some(spelled);
                        cost = JUNK_COST + seg_len as u32;
                    }
                }
                let Some(p) = phone else { continue };

                let mut cand = base.clone();
                cand.cost += cost;
                if EN_TOP_WORDS.contains(segment.as_str()) {
                    cand.top += 1;
                }
                cand.lens.push(seg_len as u8);
                cand.phones.push(p);
                if dp[j].as_ref().map_or(true, |old: &Path| better(&cand, old)) {
                    dp[j] = Some(cand);
                }
            }
        }

        let result = dp[n].take().map(|p: Path| p.phones.join(" "));

        // Cache the result, `None` included, so the work is not repeated.
        {
            let mut w = self.segmentation_cache.write().unwrap();
            if w.len() >= 5_000 { w.clear(); }
            w.insert(cache_key, result.clone());
        }

        result
    }

    /// Character-by-character fallback, the last resort when `segment_oov` also
    /// fails.
    fn char_fallback(&self, content: &str, lang: &str) -> String {
        content.chars().map(|c| {
            let cl = c.to_lowercase().to_string();
            if let Some(cp) = self.cached_lookup_merged(&cl) {
                cp.replace("<en>", "").trim().to_string()
            } else if let Some((v, e)) = self.cached_lookup_common(&cl) {
                let p = if lang == "en" && !e.is_empty() { e } else {
                    if !v.is_empty() { v } else { e }
                };
                p.replace("<en>", "").trim().to_string()
            } else {
                cl
            }
        }).collect::<Vec<String>>().join("")
    }

    pub fn phonemize(&self, text: &str) -> String {
        // Curly to straight apostrophe, so "i’m" finds the dictionary entry
        // "i'm" when a caller invokes G2P directly, bypassing the Normalizer.
        let text: std::borrow::Cow<str> = if text.contains('\u{2019}') || text.contains('\u{2018}') {
            std::borrow::Cow::Owned(text.replace(['\u{2019}', '\u{2018}'], "'"))
        } else {
            std::borrow::Cow::Borrowed(text)
        };
        let text = text.as_ref();
        let mut tokens = Vec::new();

        for cap in RE_TOKEN.captures_iter(text) {
            if let Some(en_tag) = cap.get(1) {
                let content = RE_TAG_STRIP.replace_all(en_tag.as_str(), "").trim().to_string();
                for scall in RE_TAG_CONTENT.captures_iter(&content) {
                    if let Some(sw) = scall.get(1) {
                        let word = sw.as_str().to_string();
                        let lw = word.to_lowercase();
                        let mut phone_val = None;

                        if let Some(p) = self.cached_lookup_merged(&lw) {
                            phone_val = Some(p.replace("<en>", "").trim().to_string());
                        } else if let Some((_, en)) = self.cached_lookup_common(&lw) {
                            if !en.is_empty() {
                                phone_val = Some(en.replace("<en>", "").trim().to_string());
                            }
                        }

                        tokens.push(Token {
                            lang: "en".to_string(),
                            content: word,
                            phone: phone_val,
                            is_explicit_en: true,
                        });
                    } else if let Some(sp) = scall.get(2) {
                        tokens.push(Token {
                            lang: "punct".to_string(),
                            content: sp.as_str().to_string(),
                            phone: Some(sp.as_str().to_string()),
                            is_explicit_en: true,
                        });
                    }
                }
            } else if let Some(word) = cap.get(2) {
                let lw = word.as_str().to_lowercase();
                if let Some(p) = self.cached_lookup_merged(&lw) {
                    let lang = if p.contains("<en>") { "en" } else { "vi" };
                    tokens.push(Token {
                        lang: lang.to_string(),
                        content: word.as_str().to_string(),
                        phone: Some(p.replace("<en>", "").trim().to_string()),
                        is_explicit_en: false,
                    });
                } else if let Some((vi, en)) = self.cached_lookup_common(&lw) {
                    tokens.push(Token {
                        lang: "common".to_string(),
                        content: word.as_str().to_string(),
                        phone: Some(format!("\x1F{}\x1F{}\x1F",
                            vi.trim(),
                            en.replace("<en>", "").trim()
                        )),
                        is_explicit_en: false,
                    });
                } else {
                    let has_vi_accent = lw.chars().any(|c| VI_ACCENTS.contains(c));
                    tokens.push(Token {
                        lang: if has_vi_accent { "vi".to_string() } else { "en".to_string() },
                        content: word.as_str().to_string(),
                        phone: None,
                        is_explicit_en: false,
                    });
                }
            } else if let Some(punct) = cap.get(3) {
                tokens.push(Token {
                    lang: "punct".to_string(),
                    content: punct.as_str().to_string(),
                    phone: Some(punct.as_str().to_string()),
                    is_explicit_en: false,
                });
            }
        }

        self.propagate_language(&mut tokens);

        let mut result = Vec::new();
        for t in tokens {
            if t.lang == "punct" {
                // Map punctuation by the Normalizer's rules, dropping quotes,
                // brackets and free-standing dashes.
                if let Some(p) = map_punct(&t.content) {
                    result.push(p.to_string());
                }
            } else {
                let phone = if let Some(p) = t.phone {
                    if p.starts_with('\x1F') && p.ends_with('\x1F') {
                        let inner = &p[1..p.len()-1];
                        let sep = inner.find('\x1F').unwrap_or(inner.len());
                        if t.lang == "en" {
                            let mut p_val = if sep + 1 <= inner.len() { inner[sep+1..].to_string() } else { String::new() };
                            // Rule for 'a': if English style but not in <en> tag, use 'ɐ'
                            if t.content.to_lowercase() == "a" && !t.is_explicit_en {
                                p_val = "ɐ".to_string();
                            }
                            p_val
                        } else {
                            inner[..sep].to_string()
                        }
                    } else {
                        let mut p_val = p;
                        // Also check for 'a' that was pre-resolved as 'en' (from merged dict with <en> tag in content)
                        if t.lang == "en" && t.content.to_lowercase() == "a" && !t.is_explicit_en {
                            p_val = "ɐ".to_string();
                        }
                        p_val
                    }
                } else {
                    // Fallback chain:
                    // 1. Dynamic-programming segmentation with the vowel filter.
                    // 2. Char-by-char (last resort)
                    let lw = t.content.to_lowercase();
                    self.segment_oov(&lw, &t.lang)
                        .unwrap_or_else(|| self.char_fallback(&t.content, &t.lang))
                };
                result.push(phone.trim().to_string());
            }
        }

        let mut joined = result.join(" ")
            .replace(" .", ".")
            .replace(" ,", ",")
            .replace(" !", "!")
            .replace(" ?", "?")
            .replace(" ;", ";")
            .replace(" :", ":");
        // Collapse repeated punctuation, mirroring the Normalizer: "..." and
        // "…" become ".", ",," becomes ",". Safe because phoneme strings never
        // contain '.' or ',' themselves.
        while joined.contains("..") { joined = joined.replace("..", "."); }
        while joined.contains(",,") { joined = joined.replace(",,", ","); }
        joined
    }

    fn propagate_language(&self, tokens: &mut Vec<Token>) {
        let n = tokens.len();
        // With no Vietnamese token anywhere, shared words default to English:
        // "I can do it" is entirely common words and must not fall back to a
        // Vietnamese reading.
        let default_lang = if tokens.iter().any(|t: &Token| t.lang == "vi") {
            "vi"
        } else {
            "en"
        };
        let mut i = 0;
        while i < n {
            if tokens[i].lang == "common" {
                let start = i;
                while i < n && tokens[i].lang == "common" { i += 1; }
                let end = i - 1;

                let is_stop_punct = |t: &Token| -> bool {
                    t.content.chars().next()
                        .map(|c| t.content.len() == c.len_utf8() && ".!?;:()[]{}".contains(c))
                        .unwrap_or(false)
                };

                // Distance to an anchor counts WORD tokens only; non-blocking
                // punctuation such as commas and apostrophes is skipped. In
                // "OK, go thôi", "go" is one word from "ok", tying with "thôi",
                // and follows the English anchor instead of being pushed away
                // from it by the comma.
                let mut left_anchor = None;
                let mut left_dist = 999;
                let mut d = 0;
                for l in (0..start).rev() {
                    if is_stop_punct(&tokens[l]) { break; }
                    if tokens[l].lang == "punct" { continue; }
                    d += 1;
                    if tokens[l].lang == "vi" || tokens[l].lang == "en" {
                        left_anchor = Some(tokens[l].lang.clone());
                        left_dist = d;
                        break;
                    }
                }

                let mut right_anchor = None;
                let mut right_dist = 999;
                let mut d = 0;
                for r in (end + 1)..n {
                    if is_stop_punct(&tokens[r]) { break; }
                    if tokens[r].lang == "punct" { continue; }
                    d += 1;
                    if tokens[r].lang == "vi" || tokens[r].lang == "en" {
                        right_anchor = Some(tokens[r].lang.clone());
                        right_dist = d;
                        break;
                    }
                }

                let final_lang = if let (Some(l), Some(r)) = (left_anchor.as_ref(), right_anchor.as_ref()) {
                    if right_dist < left_dist {
                        r.clone()
                    } else if left_dist < right_dist {
                        l.clone()
                    } else {
                        // On a tie: a shared word that is a REAL word sitting
                        // next to an English word usually belongs to that
                        // English phrase ("let's go ăn" -> "go" is English;
                        // "muốn go to market" -> "go to" is English). Single
                        // letters are the exception ("a" in "a còng", "i" in
                        // "core i chín"): they keep the right-anchor preference
                        // so they are not dragged into English.
                        let run_is_bare_letters = (start..=end)
                            .all(|k| tokens[k].content.chars().count() == 1);
                        if !run_is_bare_letters && (l == "en" || r == "en") {
                            "en".to_string()
                        } else {
                            r.clone()
                        }
                    }
                } else if let Some(l) = left_anchor {
                    l
                } else if let Some(r) = right_anchor {
                    r
                } else {
                    default_lang.to_string()
                };

                for idx in start..=end {
                    tokens[idx].lang = final_lang.clone();
                }
            } else {
                i += 1;
            }
        }
    }
}
