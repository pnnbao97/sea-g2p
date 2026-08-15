//! URLs, file paths and email addresses — used by pipeline stage 1.
//!
//! # Read in the language of the sentence
//!
//! Inside a Vietnamese sentence a path is spoken with Vietnamese names for its
//! separators ("gạch chéo", "gạch nối", "a còng"); inside an English one it
//! keeps English names. The `vi_ctx` and `en_ctx` flags carry that decision in.
//!
//! # Toneless syllable splitting
//!
//! Vietnamese identifiers are written without diacritics and often run together:
//! `thongbao`, `giaohang`, `truongminhkhai`. Reading them letter by letter is
//! unbearable, so `split_vi_syllables` searches for the best split using dynamic
//! programming over three kinds of piece:
//!
//!   - **Vietnamese syllables**, scored by frequency (2 for a top-3000 skeleton,
//!     1 for a dictionary hit, 0 otherwise) with a bonus when two adjacent
//!     pieces form a known bigram — this is what picks "tin học" over "khí
//!     tượng" for the same letters;
//!   - **English words** from the top-10k list, unpenalized;
//!   - **consonant runs** spelled out letter by letter, heavily penalized.
//!
//! The comparator prefers, in order: fewer spelled-out pieces, fewer spelled-out
//! letters, fewer pieces overall, higher score, and finally the rightmost-longest
//! split. A candidate with no Vietnamese piece at all is rejected outright.
//!
//! Dictionary lookup comes first throughout: a token already in the phoneme
//! dictionary is never split.

use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::OnceLock;
use crate::g2p::PhonemeDict;
use crate::lang::vi::num2vi::{n2w, n2w_single};
use crate::lang::vi::resources::{VI_LETTER_NAMES, COMMON_EMAIL_DOMAINS, DOMAIN_SUFFIX_MAP};

// The phoneme dictionary shared with G2P, memory-mapped so the lookup is cheap.
// The normalizer only asks "is this word known?" when deciding whether to keep
// a path or email token whole or split it into syllables.
static NORM_DICT: OnceLock<PhonemeDict> = OnceLock::new();

pub fn init_norm_dict(path: &str) {
    if NORM_DICT.get().is_some() { return; }
    if let Ok(d) = PhonemeDict::new(path) {
        let _ = NORM_DICT.set(d);
    }
}

pub fn dict_has(word: &str) -> bool {
    NORM_DICT.get()
        .map(|d| d.lookup_merged(word).is_some() || d.lookup_common(word).is_some())
        .unwrap_or(false)
}

/// Is `word` in the dictionary *as Vietnamese* — a merged entry whose
/// phonemes are not tagged `<en>`, or a common entry (which always carries a
/// Vietnamese side)? The acronym arbiter uses this instead of [`dict_has`]:
/// English entries ("vye", "us") must not license reading an all-caps token
/// as a Vietnamese word.
pub fn dict_has_vi(word: &str) -> bool {
    NORM_DICT.get()
        .map(|d| {
            d.lookup_merged(word).map(|p: &str| !p.starts_with("<en>")).unwrap_or(false)
                || d.lookup_common(word).is_some()
        })
        .unwrap_or(false)
}

/// Does the English side of the dictionary know `word`? A word it does not know
/// cannot be read in English no matter what markup wraps it, and one it does
/// know needs no markup — the stored phonemes already say whether the token is
/// a word ("json" -> dʒˈeɪsˈɑːn) or an initialism ("sql" -> ˌɛskjˌuːˈɛl).
pub fn dict_has_en(word: &str) -> bool {
    NORM_DICT.get().map(|d| d.has_english(word)).unwrap_or(false)
}

/// Mark the single LETTERS in `s` as English and leave every word bare.
///
/// The normalizer does not decide a word's language; only letters. Three
/// reasons, in the order they matter:
///
/// 1. A letter genuinely needs the marker. Stage 16 turns a bare single letter
///    into a Vietnamese letter name, so "MI5" without it comes out "mờ i five"
///    instead of ˈɛm ˈaɪ fˈaɪv.
/// 2. A word does not. The dictionary already carries the language —
///    "megapascal" and "nasa" read English with or without markup — and for a
///    genuine homograph G2P picks from the nearest language anchor, which is
///    usually right: "con voi to lắm" -> t̪ˈɔ, "i want to go home" -> tuː. A
///    marked letter beside a word is itself such an anchor, which is why
///    "<en>j</en> son" still gives dʒˈeɪ sˈʌn.
/// 3. Where the anchor loses, the Vietnamese reading is acceptable anyway —
///    "kỳ thi SAT" as "sát", "thẻ SIM" as "sim" — and a caller who wants
///    otherwise can write the `<en>` tag by hand. That escape hatch is the
///    user's, not the normalizer's.
///
/// Adjacent letters share one marker pair, so "washington d c" is marked once
/// rather than twice and "b two b" keeps its "two" bare.
pub fn en_marked(s: &str) -> String {
    let toks: Vec<&str> = s.split_whitespace().collect();
    if toks.is_empty() { return s.to_string(); }
    let need: Vec<bool> = toks.iter()
        .map(|t: &&str| t.chars().count() == 1)
        .collect();
    if !need.iter().any(|b: &bool| *b) { return toks.join(" "); }

    let mut out: Vec<String> = Vec::new();
    let mut run: Vec<&str> = Vec::new();
    for (t, n) in toks.iter().zip(need.iter()) {
        if *n {
            run.push(t);
        } else {
            if !run.is_empty() {
                out.push(format!("__start_en__{}__end_en__", run.join(" ")));
                run.clear();
            }
            out.push(t.to_string());
        }
    }
    if !run.is_empty() {
        out.push(format!("__start_en__{}__end_en__", run.join(" ")));
    }
    out.join(" ")
}

// ── Vietnamese-style reading of paths, URLs and emails ──────────────────────
// Vietnamese onsets without diacritics ("đ" folded to "d"). Longer strings come
// first so they are tried first; the empty string last covers syllables with no
// onset at all ("an", "uong").
static VI_ONSETS: &[&str] = &[
    "ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr",
    "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x", "",
];

// Vietnamese rimes without diacritics (ă/â -> a, ê -> e, ô/ơ -> o, ư -> u, all
// tone variants collapsing to one skeleton). Accuracy only has to reach
// "plausibly a Vietnamese syllable".
static VI_RHYMES: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "a", "ac", "ach", "ai", "am", "an", "ang", "anh", "ao", "ap", "at", "au", "ay",
        "e", "ec", "ech", "em", "en", "eng", "enh", "eo", "ep", "et", "eu",
        "i", "ia", "ich", "iec", "iem", "ien", "ieng", "iep", "iet", "ieu",
        "im", "in", "inh", "ip", "it", "iu",
        "o", "oa", "oac", "oach", "oai", "oan", "oang", "oanh", "oap", "oat", "oay",
        "oc", "oe", "oen", "oeo", "oi", "om", "on", "ong", "ooc", "oong", "op", "ot",
        "u", "ua", "uan", "uat", "uay", "uc", "ue", "uech", "uenh", "ui", "um", "un",
        "ung", "uo", "uoc", "uoi", "uom", "uon", "uong", "uot", "uou", "up", "ut",
        "uu", "uy", "uya", "uych", "uyen", "uyet", "uynh", "uyt", "uyu",
        "y", "yem", "yen", "yet", "yeu",
    ].into_iter().collect()
});

// File extensions: after "chấm" they keep the established English-style
// reading even inside a Vietnamese sentence — "chấm p y", "chấm jpg".
static FILE_EXTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "txt", "log", "tar", "gz", "zip", "rar", "sh", "py", "js", "ts", "cpp",
        "c", "h", "rs", "go", "java", "php", "json", "xml", "yaml", "yml", "md",
        "csv", "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "exe", "dll",
        "so", "config", "ini", "bat", "jpg", "jpeg", "png", "gif", "bmp", "svg",
        "webp", "wav", "mp3", "mp4", "avi", "mkv", "html", "css", "sql", "db",
        "iso", "apk",
    ].into_iter().collect()
});

fn is_vi_syllable(s: &str) -> bool {
    for onset in VI_ONSETS {
        if let Some(rhyme) = s.strip_prefix(onset) {
            if VI_RHYMES.contains(rhyme) { return true; }
        }
    }
    false
}

/// How Vietnamese a syllable looks:
///   2 = skeleton of a FREQUENT Vietnamese syllable, per the frequency table
///       ("tin", "hoc");
///   1 = present in the dictionary with a Vietnamese reading, merged VI or
///       common ("nhoc");
///   0 = anything else: an English entry, or absent from the dictionary.
///
/// This scoring is what makes "tin|hoc" (2+2) beat "ti|nhoc" (2+1), and
/// "khi|tuong" beat "khit|uong".
fn syllable_vi_score(w: &str) -> u32 {
    if crate::lang::vi::vi_top_syllables::VI_TOP_SYLLABLES.contains(w) {
        return 2;
    }
    if let Some(d) = NORM_DICT.get() {
        if let Some(p) = d.lookup_merged(w) {
            if !p.starts_with("<en>") { return 1; }
        }
        if d.lookup_common(w).is_some() { return 1; }
    }
    0
}

/// Split a lowercase ASCII run into pieces: toneless Vietnamese syllables
/// (`is_vi = true`) interleaved with FOREIGN pieces of at least three
/// characters (`is_vi = false`) for mixed compounds
/// ("blogcongnghe" -> blog|cong|nghe, "tapdoanxyz" -> tap|doan|xyz).
/// Dynamic programming picks the split by these criteria, in order:
///   1. fewest foreign pieces, so a fully Vietnamese split always wins;
///   2. fewest foreign characters in total ("blog|cong|nghe" beats one block);
///   3. fewest pieces ("luu|tru" beats "lu|u|tru"; whole "blog" beats fragments);
///   4. fewest dictionary syllables that are not read the Vietnamese way
///      ("tra|cuu" beats "trac|uu", because "tra" is a VI entry while
///      "trac" and "uu" are junk EN entries);
///   5. longer FINAL piece ("tin|hoc" beats "tinh|oc").
///
/// Returns `None` when no Vietnamese syllable is found at all, i.e. the token is
/// entirely foreign.
fn split_vi_syllables(s: &str) -> Option<Vec<(String, bool)>> {
    if s.is_empty() || !s.is_ascii() { return None; }

    #[derive(Clone)]
    struct P {
        jsegs: u32,
        jletters: u32,
        score: u32,
        lens: Vec<u8>,
        parts: Vec<(String, bool)>,
    }
    fn better(a: &P, b: &P) -> bool {
        if a.jsegs != b.jsegs { return a.jsegs < b.jsegs; }
        if a.jletters != b.jletters { return a.jletters < b.jletters; }
        if a.lens.len() != b.lens.len() { return a.lens.len() < b.lens.len(); }
        if a.score != b.score { return a.score > b.score; }
        for (x, y) in a.lens.iter().rev().zip(b.lens.iter().rev()) {
            if x != y { return x > y; }
        }
        false
    }

    let n = s.len();
    let mut dp: Vec<Option<P>> = vec![None; n + 1];
    dp[0] = Some(P { jsegs: 0, jletters: 0, score: 0, lens: Vec::new(), parts: Vec::new() });
    for i in 0..n {
        let Some(base) = dp[i].clone() else { continue };
        // Vietnamese syllable piece, at most seven characters.
        for j in (i + 1)..=n.min(i + 7) {
            let seg = &s[i..j];
            if !is_vi_syllable(seg) { continue; }
            let mut cand = base.clone();
            let mut sc = syllable_vi_score(seg);
            // Adjacent pieces forming a real compound ("tin hoc", "khi tuong")
            // get a large bonus. This is the tie-breaker that makes "tin|hoc"
            // win over "ti|nhoc".
            if let Some((prev, prev_is_vi)) = base.parts.last() {
                if *prev_is_vi {
                    let key = format!("{} {}", prev, seg);
                    if crate::lang::vi::vi_bigrams::VI_BIGRAMS.contains(key.as_str()) {
                        sc += 3;
                    }
                }
            }
            cand.score += sc;
            cand.lens.push((j - i) as u8);
            cand.parts.push((seg.to_string(), true));
            if dp[j].as_ref().map_or(true, |old: &P| better(&cand, old)) {
                dp[j] = Some(cand);
            }
        }
        // Pieces that are common English words (top wordlist) escape the
        // foreign-piece penalty: "smart|home" beats "smart|ho|me", and
        // "blog|cong|nghe" survives intact.
        for j in (i + 3)..=n {
            let seg = &s[i..j];
            if !crate::lang::en::top_words::EN_TOP_WORDS.contains(seg) { continue; }
            let mut cand = base.clone();
            cand.lens.push((j - i).min(255) as u8);
            cand.parts.push((seg.to_string(), false));
            if dp[j].as_ref().map_or(true, |old: &P| better(&cand, old)) {
                dp[j] = Some(cand);
            }
        }
        // Consonant-only foreign pieces ("xyz", "pnn", "tsn"; "y" counts as a
        // consonant so the piece can be spelled out): at least three
        // characters, penalised on both counters, and therefore used only when
        // nothing else fits. A foreign piece containing a vowel but absent from
        // the top wordlist is rejected outright ("smar", "ldserver"), which
        // keeps words like "buildserver" whole for G2P.
        for j in (i + 3)..=n {
            let seg = &s[i..j];
            if seg.chars().any(|c: char| "aeiou".contains(c)) { continue; }
            let mut cand = base.clone();
            cand.jsegs += 1;
            cand.jletters += (j - i) as u32;
            cand.lens.push((j - i).min(255) as u8);
            cand.parts.push((seg.to_string(), false));
            if dp[j].as_ref().map_or(true, |old: &P| better(&cand, old)) {
                dp[j] = Some(cand);
            }
        }
    }
    let best = dp[n].take()?;
    // No Vietnamese syllable at all: leave the token to another code path.
    if !best.parts.iter().any(|(_, is_vi): &(String, bool)| *is_vi) { return None; }
    Some(best.parts)
}

fn vi_letter_names(s: &str) -> String {
    s.chars().map(|c: char| {
        let cl = c.to_lowercase().to_string();
        VI_LETTER_NAMES.get(cl.as_str()).map(|v| v.to_string()).unwrap_or(cl)
    }).collect::<Vec<String>>().join(" ")
}

/// Render the output of `split_vi_syllables` as readable text: Vietnamese
/// syllables are emitted bare; consonant-only foreign pieces are spelled with
/// Vietnamese letter names ("xyz" -> "ích y dét"); foreign pieces containing a
/// vowel stay bare for G2P to read from the dictionary ("blog").
fn render_vi_split(pieces: &[(String, bool)]) -> String {
    pieces.iter().map(|(txt, is_vi): &(String, bool)| {
        if *is_vi {
            txt.clone()
        } else if !txt.chars().any(|c: char| "aeiou".contains(c)) {
            vi_letter_names(txt)
        } else {
            txt.clone()
        }
    }).collect::<Vec<String>>().join(" ")
}

/// English-style reading of a letter cluster: short all-caps runs and clusters
/// of at most two characters are spelled out; anything longer is read as a word.
fn en_chunk(t: &str) -> String {
    let mut val = t.to_lowercase();
    if (t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4) || t.len() <= 2 {
        val = val.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
    }
    en_marked(&val)
}

/// Email variant: local parts are always read as English words, short tokens
/// included, so the Vietnamese branch is added only when `vi_ctx` holds.
fn norm_letter_chunk_email(t: &str, vi_ctx: bool, _en_ctx: bool) -> String {
    let lw = t.to_lowercase();
    if !vi_ctx { return en_marked(&lw); }
    // Single letters and consonant-only runs take Vietnamese letter names,
    // checked before the dictionary, exactly as in paths.
    if lw.chars().count() == 1 || !lw.chars().any(|c: char| "aeiouy".contains(c)) {
        return vi_letter_names(&lw);
    }
    if dict_has(&lw) { return lw; }
    if let Some(pieces) = split_vi_syllables(&lw) {
        return render_vi_split(&pieces);
    }
    // Unknown word: leave it bare for G2P to look up, or read as English OOV.
    lw
}

/// Read one letter cluster from a path, URL or email.
///
/// With `vi_ctx` — the sentence contains Vietnamese words — the Vietnamese
/// reading is preferred: toneless syllable splitting ("thongbao" -> "thong
/// bao") and letter names for consonant-only runs ("mn" -> "mờ nờ"). Familiar
/// English words keep their English reading regardless.
fn norm_letter_chunk(t: &str, vi_ctx: bool, after_dot: bool) -> String {
    if !vi_ctx { return en_chunk(t); }
    let lw = t.to_lowercase();
    // Known file extensions: those with a real vowel are read as words ("zip",
    // "yaml"); consonant-only ones, where "y" does not count as a vowel, take
    // Vietnamese letter names ("py" -> "phê y", "jpg" -> "giây phê gờ").
    if after_dot && FILE_EXTS.contains(lw.as_str()) {
        if lw.chars().any(|c: char| "aeiou".contains(c)) { return lw; }
        return vi_letter_names(&lw);
    }
    // A short all-caps run is an acronym (TTS, GPU), spelled with Vietnamese
    // letter names: "tê tê ét".
    if t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4 && t.len() >= 2 {
        return vi_letter_names(&lw);
    }
    // Single letters ("v" in v2, "c" in C:) and consonant-only runs ("www",
    // "mn", "db") take Vietnamese letter names. Checked BEFORE the dictionary,
    // otherwise a dictionary entry would swallow "www" or "v" instead of
    // yielding "vê kép vê kép vê kép", "vê", "xê", "mờ nờ".
    if lw.chars().count() == 1 || !lw.chars().any(|c: char| "aeiouy".contains(c)) {
        return vi_letter_names(&lw);
    }
    // camelCase already encodes syllable boundaries ("CanHoMau" ->
    // Can|Ho|Mau), so if every piece is a Vietnamese syllable, use that split
    // directly. Checked BEFORE the dictionary so a junk entry such as "canhan"
    // cannot swallow "CaNhan".
    if t.chars().any(|c: char| c.is_uppercase()) && t.chars().any(|c: char| c.is_lowercase()) {
        let mut pieces: Vec<String> = Vec::new();
        let mut cur = String::new();
        for c in t.chars() {
            if c.is_uppercase() && !cur.is_empty() {
                pieces.push(cur.to_lowercase());
                cur = String::new();
            }
            cur.push(c);
        }
        if !cur.is_empty() { pieces.push(cur.to_lowercase()); }
        if pieces.len() > 1 && pieces.iter().all(|p: &String| is_vi_syllable(p)) {
            return pieces.join(" ");
        }
    }
    // Present in the sea-g2p dictionary: leave it bare and untagged so G2P
    // reads it from the dictionary — merged English entries in English, common
    // entries following the surrounding Vietnamese context. This is what stops
    // familiar English words ("home", "data") from being split into Vietnamese
    // syllables.
    if dict_has(&lw) { return lw; }
    if let Some(pieces) = split_vi_syllables(&lw) {
        return render_vi_split(&pieces);
    }
    // Unknown word ("pnnbao"): left bare for dictionary lookup or English OOV
    // reading.
    lw
}

static RE_TECH_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([./:?&=/_ \-\\#@])").unwrap());
static RE_EMAIL_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([._\-+])").unwrap());
static RE_SUB_TOKENS: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"[a-zA-Z]+|\d+").unwrap());

pub static RE_TECHNICAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)
    \b(?:https?|ftp)://[\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b(?:www\.)[\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b[A-Za-z0-9.\-]+(?:\.com|\.vn|\.net|\.org|\.gov|\.edu|\.io|\.biz|\.info|\.dev|\.shop|\.app|\.tech|\.studio|\.online|\.store|\.ai|\.ly|\.me|\.gle|\.cc|\.co|\.tv|\.xyz|\.site|\.link|\.page|\.blog|\.news|\.pro)(?:[/?#][\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]*)?\b
    |
    (?<![\w\\])\\\\[a-zA-Z0-9._\-]+(?:\\[\p{L}0-9._\-]+)*\\?
    |
    (?<![\w\\])\\[a-zA-Z0-9._\-]+(?:\\[\p{L}0-9._\-]+)+\\?
    |
    (?<!\w)/[a-zA-Z0-9._\-/]{2,}\b
    |
    \b[a-zA-Z]:\\[a-zA-Z0-9._\\\-]+\b
    |
    \b[a-zA-Z0-9._\-]+\.(?:txt|log|tar|gz|zip|sh|py|js|cpp|h|json|xml|yaml|yml|md|csv|pdf|docx|xlsx|exe|dll|so|config)\b
    |
    \b[a-zA-Z][a-zA-Z0-9]*(?:[._\-][a-zA-Z0-9]+){2,}\b
    |
    \b[a-fA-F0-9]{1,4}(?::[a-fA-F0-9]{1,4}){3,7}\b
    ").unwrap()
});

pub static RE_EMAIL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap()
});

pub static RE_SLASH_NUMBER: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![a-zA-Z\d,.])(\d+)/(\d+)(?![\d,.])").unwrap()
});

static RE_NEG_FRAC: Lazy<regex::Regex> = Lazy::new(|| {
    regex::Regex::new(r"(?:=|\s)-((\d+)/(\d+))").unwrap()
});

// Denominator immediately followed by a letter: 225/45R17, 195/65R15
static RE_SLASH_ALNUM: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?<![a-zA-Z\d,.])(\d+)/(\d+[a-zA-Z][a-zA-Z0-9]*)").unwrap()
});

pub fn normalize_technical(text: &str, vi_ctx: bool, en_ctx: bool) -> String {
    let slash_name = if en_ctx { "slash" } else if vi_ctx { "gạch chéo" } else { "gạch" };
    let hyphen_name = if en_ctx { "dash" } else if vi_ctx { "gạch nối" } else { "gạch ngang" };
    let dot_name = if en_ctx { "dot" } else { "chấm" };
    let underscore_name = if en_ctx { "underscore" } else { "gạch dưới" };
    let colon_name = if en_ctx { "colon" } else { "hai chấm" };
    RE_TECHNICAL.replace_all(text, |caps: &Captures| {
        let orig = caps.get(0).unwrap().as_str();
        let mut rest = orig;
        let mut res = Vec::new();

        if let Some(p_idx) = orig.to_lowercase().find("://") {
            let protocol = &orig[..p_idx];
            if vi_ctx {
                // "https://" -> "hát tê tê phê ét hai chấm gạch chéo gạch chéo"
                res.push(vi_letter_names(&protocol.to_lowercase()));
                res.push("hai chấm gạch chéo gạch chéo".to_string());
            } else {
                let p_norm = if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                    protocol.to_lowercase().chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ")
                } else {
                    protocol.to_lowercase()
                };
                res.push(en_marked(&p_norm));
                if en_ctx {
                    res.push("colon slash slash".to_string());
                }
            }
            rest = &orig[p_idx + 3..];
        } else if orig.starts_with('/') {
            res.push(slash_name.to_string());
            rest = &orig[1..];
        }

        let re_split = &*RE_TECH_SPLIT;
        let mut segments_vec = Vec::new();
        let mut last = 0;
        for mat in re_split.find_iter(rest) {
            segments_vec.push(&rest[last..mat.start()]);
            segments_vec.push(mat.as_str());
            last = mat.end();
        }
        segments_vec.push(&rest[last..]);

        let mut idx = 0;
        let mut after_dot = false;
        while idx < segments_vec.len() {
            let s = segments_vec[idx];
            if s.is_empty() { idx += 1; continue; }

            let mut next_after_dot = false;
            match s {
                "." => {
                    let mut next_seg = "";
                    for j in idx + 1..segments_vec.len() {
                        let sj = segments_vec[j];
                        if !sj.is_empty() && !("./:?&=/_ -\\".contains(sj)) {
                            next_seg = sj;
                            break;
                        }
                    }
                    // The suffix table ("com", "o rờ gờ") applies outside English sentences.
                    if !en_ctx && !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                        res.push("chấm".to_string());
                        res.push(DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap().to_string());
                        idx += 1;
                        while idx < segments_vec.len() && (segments_vec[idx].is_empty() || segments_vec[idx].to_lowercase() != next_seg.to_lowercase()) {
                            idx += 1;
                        }
                        idx += 1;
                        continue;
                    }
                    res.push(dot_name.to_string());
                    next_after_dot = true;
                }
                "/" | "\\" => res.push(slash_name.to_string()),
                "-" => res.push(hyphen_name.to_string()),
                "_" => res.push(underscore_name.to_string()),
                ":" => res.push(colon_name.to_string()),
                "?" => res.push(if en_ctx { "question mark" } else { "hỏi chấm" }.to_string()),
                "&" => res.push(if en_ctx { "and" } else { "và" }.to_string()),
                "=" => res.push(if en_ctx { "equals" } else { "bằng" }.to_string()),
                "#" => res.push(if en_ctx { "hash" } else { "thăng" }.to_string()),
                "@" => res.push(if en_ctx { "at" } else { "a còng" }.to_string()),
                _ => {
                    // A path segment containing diacritic Vietnamese is read as
                    // Vietnamese words rather than spelled out character by
                    // character (".../báo-cáo" -> "báo" "cáo").
                    if s.chars().any(|c: char| c.is_alphabetic() && !c.is_ascii()) {
                        res.push(s.to_lowercase());
                    } else if !en_ctx && DOMAIN_SUFFIX_MAP.contains_key(s.to_lowercase().as_str()) {
                        // Domain suffixes follow the table ("i ô", "vi en").
                        // English sentences skip it and fall through to the
                        // English letter branch below.
                        res.push(DOMAIN_SUFFIX_MAP.get(s.to_lowercase().as_str()).unwrap().to_string());
                    } else if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                        // In English sentences digits are read individually in
                        // English ("127" -> "one two seven").
                        let digits = |d: &str| -> String {
                            if en_ctx {
                                crate::lang::vi::num2en::n2w_en_digits(d)
                            } else {
                                d.chars().map(|c: char| n2w_single(&c.to_string())).collect::<Vec<String>>().join(" ")
                            }
                        };
                        if s.chars().all(|c: char| c.is_ascii_digit()) {
                            res.push(digits(s));
                        } else {
                            let re_sub = &*RE_SUB_TOKENS;
                            let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m: regex::Match| m.as_str()).collect();
                            if sub_tokens.len() > 1 {
                                for t in sub_tokens {
                                    if t.chars().all(|c: char| c.is_ascii_digit()) {
                                        res.push(digits(t));
                                    } else {
                                        res.push(norm_letter_chunk(t, vi_ctx, after_dot));
                                    }
                                }
                            } else {
                                res.push(norm_letter_chunk(s, vi_ctx, after_dot));
                            }
                        }
                    } else {
                        for char in s.to_lowercase().chars() {
                            if char.is_alphanumeric() {
                                if char.is_ascii_digit() {
                                    res.push(n2w_single(&char.to_string()));
                                } else {
                                    res.push(VI_LETTER_NAMES.get(char.to_string().as_str()).cloned().unwrap_or(char.to_string().as_str()).to_string());
                                }
                            } else {
                                res.push(char.to_string());
                            }
                        }
                    }
                }
            }
            after_dot = next_after_dot;
            idx += 1;
        }
        res.join(" ").replace("  ", " ").trim().to_string()
    }).to_string()
}

pub fn normalize_emails(text: &str, vi_ctx: bool, en_ctx: bool) -> String {
    let hyphen_name = if en_ctx { "dash" } else if vi_ctx { "gạch nối" } else { "gạch ngang" };
    let dot_name = if en_ctx { "dot" } else { "chấm" };
    let at_name = if en_ctx { "at" } else { "a còng" };
    RE_EMAIL.replace_all(text, |caps: &Captures| {
        let email = caps.get(0).unwrap().as_str();
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() != 2 { return email.to_string(); }

        let user_part = parts[0];
        let domain_part = parts[1];

        let norm_segment = |s: &str| {
            if s.is_empty() { return String::new(); }
            if s.chars().all(|c: char| c.is_ascii_digit()) {
                return if en_ctx { crate::lang::vi::num2en::n2w_en(s) } else { n2w(s) };
            }
            if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                let re_sub = &*RE_SUB_TOKENS;
                let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m: regex::Match| m.as_str()).collect();
                if sub_tokens.len() > 1 {
                    let mut res_parts = Vec::new();
                    for t in sub_tokens {
                        if t.chars().all(|c: char| c.is_ascii_digit()) {
                            res_parts.push(if en_ctx { crate::lang::vi::num2en::n2w_en(t) } else { n2w(t) });
                        } else {
                            res_parts.push(norm_letter_chunk_email(t, vi_ctx, en_ctx));
                        }
                    }
                    return res_parts.join(" ");
                }
                return norm_letter_chunk_email(s, vi_ctx, en_ctx);
            }

            let mut res = Vec::new();
            for char in s.to_lowercase().chars() {
                if char.is_alphanumeric() {
                    if char.is_ascii_digit() {
                        res.push(n2w_single(&char.to_string()));
                    } else {
                        res.push(VI_LETTER_NAMES.get(char.to_string().as_str()).cloned().unwrap_or(char.to_string().as_str()).to_string());
                    }
                } else {
                    res.push(char.to_string());
                }
            }
            res.join(" ")
        };

        let process_part = |p: &str, is_domain: bool| {
            let re_split = &*RE_EMAIL_SPLIT;
            let mut segments_vec = Vec::new();
            let mut last = 0;
            for mat in re_split.find_iter(p) {
                segments_vec.push(&p[last..mat.start()]);
                segments_vec.push(mat.as_str());
                last = mat.end();
            }
            segments_vec.push(&p[last..]);

            let mut res = Vec::new();
            let mut idx = 0;
            while idx < segments_vec.len() {
                let s = segments_vec[idx];
                if s.is_empty() { idx += 1; continue; }
                match s {
                    "." => {
                        if is_domain {
                            let mut next_seg = "";
                            let mut peek_idx = -1;
                            for j in idx + 1..segments_vec.len() {
                                let sj = segments_vec[j];
                                if !sj.is_empty() && !("._-+".contains(sj)) {
                                    next_seg = sj;
                                    peek_idx = j as i32;
                                    break;
                                }
                            }
                            if !en_ctx && !next_seg.is_empty() && DOMAIN_SUFFIX_MAP.contains_key(next_seg.to_lowercase().as_str()) {
                                res.push("chấm".to_string());
                                res.push(DOMAIN_SUFFIX_MAP.get(next_seg.to_lowercase().as_str()).unwrap().to_string());
                                idx = peek_idx as usize + 1;
                                continue;
                            }
                        }
                        res.push(dot_name.to_string());
                    }
                    "_" => res.push(if en_ctx { "underscore" } else { "gạch dưới" }.to_string()),
                    "-" => res.push(hyphen_name.to_string()),
                    "+" => res.push(if en_ctx { "plus" } else { "cộng" }.to_string()),
                    _ => res.push(norm_segment(s)),
                }
                idx += 1;
            }
            res.join(" ")
        };

        let user_norm = process_part(user_part, false);
        let domain_part_lower = domain_part.to_lowercase();
        // The familiar-domain table spells "chấm" in Vietnamese, so it applies only
    // outside pure-English sentences.
        let domain_norm = if !en_ctx {
            if let Some(dn) = COMMON_EMAIL_DOMAINS.get(domain_part_lower.as_str()) {
                dn.to_string()
            } else {
                process_part(domain_part, true)
            }
        } else {
            process_part(domain_part, true)
        };

        format!("{} {} {}", user_norm, at_name, domain_norm).replace("  ", " ").trim().to_string()
    }).to_string()
}

pub fn normalize_slashes(text: &str) -> String {
    let res = RE_NEG_FRAC.replace_all(text, |caps: &regex::Captures| {
        let matched = caps.get(0).unwrap().as_str();
        let frac = caps.get(1).unwrap().as_str();
        let prefix = if matched.starts_with('=') { "= âm " } else { " âm " };
        format!("{}{}", prefix, frac)
    }).into_owned();

    // Handle patterns like 225/45R17: split denominator at letter/digit boundaries,
    // read digit groups as full numbers, letter groups as letter names.
    let res2 = RE_SLASH_ALNUM.replace_all(&res, |caps: &Captures| {
        let n1 = caps.get(1).unwrap().as_str();
        let alnum = caps.get(2).unwrap().as_str(); // e.g. "45R17"
        let sub_tokens = RE_SUB_TOKENS.find_iter(alnum);
        let alnum_spoken: Vec<String> = sub_tokens.map(|m: regex::Match| {
            let t = m.as_str();
            if t.chars().all(|c| c.is_ascii_digit()) {
                n2w(t)
            } else {
                t.chars().map(|c: char| {
                    crate::lang::vi::resources::VI_LETTER_NAMES
                        .get(c.to_lowercase().to_string().as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| c.to_lowercase().to_string())
                }).collect::<Vec<String>>().join(" ")
            }
        }).collect();
        format!("{} trên {}", n2w(n1), alnum_spoken.join(" "))
    }).to_string();

    RE_SLASH_NUMBER.replace_all(&res2, |caps: &Captures| {
        let n1 = caps.get(1).unwrap().as_str();
        let n2 = caps.get(2).unwrap().as_str();
        format!("{} trên {}", n2w(n1), n2w(n2))
    }).to_string()
}
