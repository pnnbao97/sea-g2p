//! Rule-based Thai grapheme-to-phoneme: the out-of-vocabulary fallback.
//!
//! The dictionary covers >99% of corpus tokens; this reads the rest — new
//! proper names, transliterations, typos — so the pipeline never has to give
//! up on a string. Measured against the 14.8k-word gold Wiktionary lexicon:
//! 84% of syllables and 77% of short words exactly right (`thai/rule_g2p.py`
//! is the reference implementation these rules were validated with).
//!
//! # Writing order is not reading order
//!
//! Every Thai syllable is written in one frame:
//!
//! ```text
//! [pre-vowel] C [cluster] [above/below vowel] [tone mark] [post-vowel] [final]
//!    เแโใไ                   ◌ั◌ิ◌ี◌ึ◌ื◌ุ◌ู◌็       ◌่◌้◌๊◌๋        าะอวย
//! ```
//!
//! so the tone mark always sits after the consonant and any above/below
//! vowel, and BEFORE า/อ/ะ. Getting that slot wrong misreads ข้าว as
//! "kʰa-wa" instead of /kʰaːw˥˩/.
//!
//! # The two rules that carry the accuracy
//!
//! - **อักษรนำ (leading consonant)**: a high/mid-class consonant with no
//!   vowel of its own, directly before a low-class sonorant, is read /Ca/
//!   AND lends its class to the next syllable — this is why สวัสดี is
//!   sa-wàt-dii and not sa-wát-dii.
//! - **Tone is computed**, never written: (class × liveness × mark).
//!
//! # Parsing
//!
//! Syllable boundaries are ambiguous (ผู้คน is pʰûː-kʰon, not pʰûːk-na), so
//! this is a DP over the string: each candidate reading is an edge with a
//! cost, and the cheapest path wins. Explicit-vowel syllables cost less than
//! inherent-vowel ones, which is what stops a following onset from being
//! stolen as a coda.

use once_cell::sync::Lazy;
use std::collections::HashMap;

const MID: &str = "˧";
const LOW: &str = "˨˩";
const FALL: &str = "˥˩";
const HIGH: &str = "˦˥";
const RISE: &str = "˩˩˦";

static INIT: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('ก', "k"), ('ข', "kʰ"), ('ฃ', "kʰ"), ('ค', "kʰ"), ('ฅ', "kʰ"), ('ฆ', "kʰ"),
        ('ง', "ŋ"), ('จ', "tɕ"), ('ฉ', "tɕʰ"), ('ช', "tɕʰ"), ('ซ', "s"), ('ฌ', "tɕʰ"),
        ('ญ', "j"), ('ฎ', "d"), ('ฏ', "t"), ('ฐ', "tʰ"), ('ฑ', "tʰ"), ('ฒ', "tʰ"),
        ('ณ', "n"), ('ด', "d"), ('ต', "t"), ('ถ', "tʰ"), ('ท', "tʰ"), ('ธ', "tʰ"),
        ('น', "n"), ('บ', "b"), ('ป', "p"), ('ผ', "pʰ"), ('ฝ', "f"), ('พ', "pʰ"),
        ('ฟ', "f"), ('ภ', "pʰ"), ('ม', "m"), ('ย', "j"), ('ร', "r"), ('ล', "l"),
        ('ว', "w"), ('ศ', "s"), ('ษ', "s"), ('ส', "s"), ('ห', "h"), ('ฬ', "l"),
        ('อ', "ʔ"), ('ฮ', "h"),
    ].into_iter().collect()
});

static FINAL: Lazy<HashMap<char, &'static str>> = Lazy::new(|| {
    [
        ('ก', "k̚"), ('ข', "k̚"), ('ค', "k̚"), ('ฆ', "k̚"), ('ง', "ŋ"),
        ('จ', "t̚"), ('ช', "t̚"), ('ซ', "t̚"), ('ฎ', "t̚"), ('ฏ', "t̚"), ('ฐ', "t̚"),
        ('ฑ', "t̚"), ('ฒ', "t̚"), ('ด', "t̚"), ('ต', "t̚"), ('ถ', "t̚"), ('ท', "t̚"),
        ('ธ', "t̚"), ('ศ', "t̚"), ('ษ', "t̚"), ('ส', "t̚"), ('ญ', "n"), ('ณ', "n"),
        ('น', "n"), ('ร', "n"), ('ล', "n"), ('ฬ', "n"), ('บ', "p̚"), ('ป', "p̚"),
        ('พ', "p̚"), ('ฟ', "p̚"), ('ภ', "p̚"), ('ม', "m"), ('ย', "j"), ('ว', "w"),
    ].into_iter().collect()
});

const CLASS_MID: &str = "กจฎฏดตบปอ";
const CLASS_HIGH: &str = "ขฃฉฐถผฝศษสห";
const SONORANT: &str = "งญนมยรลวฬณ";
const PREPOSED: &str = "เแโใไ";

static TRUE_CLUSTERS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec!["กร", "กล", "กว", "ขร", "ขล", "ขว", "คร", "คล", "คว", "ตร",
         "ปร", "ปล", "ผล", "พร", "พล", "บร", "บล", "ดร", "ฟร", "ฟล"]
});
static PSEUDO: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    [("ทร", "s"), ("จร", "tɕ"), ("ซร", "s"), ("สร", "s"), ("ศร", "s")]
        .into_iter().collect()
});

fn is_long(v: &str) -> bool {
    matches!(v, "aː" | "iː" | "ɯː" | "uː" | "eː" | "ɛː" | "oː" | "ɔː" | "ɤː" | "ia" | "ɯa" | "ua")
}

fn is_dependent(c: char) -> bool {
    matches!(c,
        'ะ' | 'ั' | 'า' | 'ำ' | 'ิ' | 'ี' | 'ึ' | 'ื' | 'ุ' | 'ู' | '็'
        | '่' | '้' | '๊' | '๋' | '์' | 'ํ' | '๎' | 'ฺ')
}

#[derive(Clone, Copy, PartialEq)]
enum Class { Mid, High, Low }

fn class_of(c: char) -> Class {
    if CLASS_MID.contains(c) { Class::Mid }
    else if CLASS_HIGH.contains(c) { Class::High }
    else { Class::Low }
}

fn tone(cls: Class, vowel: &str, coda: &str, mark: Option<char>) -> &'static str {
    let live = matches!(coda, "m" | "n" | "ŋ" | "j" | "w") || (coda.is_empty() && is_long(vowel));
    match mark {
        Some('่') => if cls == Class::Low { FALL } else { LOW },
        Some('้') => if cls == Class::Low { HIGH } else { FALL },
        Some('๊') => HIGH,
        Some('๋') => RISE,
        _ => match cls {
            Class::Mid => if live { MID } else { LOW },
            Class::High => if live { RISE } else { LOW },
            Class::Low => {
                if live { MID } else if !is_long(vowel) { HIGH } else { FALL }
            }
        },
    }
}

/// One vowel frame: what is written before the consonant, above/below it,
/// and after it, plus the vowel it spells and whether a final consonant is
/// required. Order matters — earlier frames win ties in the DP.
struct Frame {
    pre: &'static str,
    ab: &'static str,
    post: &'static str,
    vowel: &'static str,
    needs_final: bool,
}

macro_rules! frames {
    ($(($p:expr, $a:expr, $o:expr, $v:expr, $f:expr)),* $(,)?) => {
        vec![$(Frame { pre: $p, ab: $a, post: $o, vowel: $v, needs_final: $f }),*]
    };
}

static FRAMES: Lazy<Vec<Frame>> = Lazy::new(|| frames![
    // ◌ัว first: the ว here belongs to the vowel, never to the coda slot
    ("", "ั", "ว", "ua", false),
    ("", "ั", "วะ", "ua", false),
    // closed frames
    ("เ", "ี", "ย", "ia", true),
    ("เ", "ื", "อ", "ɯa", true),
    ("", "ั", "ว", "ua", true),
    ("เ", "็", "", "e", true),
    ("แ", "็", "", "ɛ", true),
    ("เ", "ิ", "", "ɤː", true),
    ("เ", "", "", "eː", true),
    ("แ", "", "", "ɛː", true),
    ("โ", "", "", "oː", true),
    ("", "", "อ", "ɔː", true),
    ("", "ั", "", "a", true),
    ("", "", "า", "aː", true),
    // /ua/ loses its ◌ั before a final (สวย, ด้วย, ห่วย)
    ("", "", "ว", "ua", true),
    ("", "ิ", "", "i", true),
    ("", "ี", "", "iː", true),
    ("", "ึ", "", "ɯ", true),
    ("", "ื", "", "ɯː", true),
    ("", "ุ", "", "u", true),
    ("", "ู", "", "uː", true),
    // open frames
    ("เ", "ี", "ยะ", "ia", false),
    ("เ", "ี", "ย", "ia", false),
    ("เ", "ื", "อะ", "ɯa", false),
    ("เ", "ื", "อ", "ɯa", false),
    ("เ", "", "อะ", "ɤ", false),
    ("เ", "ิ", "", "ɤː", false),
    ("เ", "", "อ", "ɤː", false),
    ("เ", "", "าะ", "ɔ", false),
    ("เ", "", "า", "aw", false),
    ("เ", "", "ะ", "e", false),
    ("แ", "", "ะ", "ɛ", false),
    ("โ", "", "ะ", "o", false),
    ("เ", "", "", "eː", false),
    ("แ", "", "", "ɛː", false),
    ("โ", "", "", "oː", false),
    ("ใ", "", "", "aj", false),
    ("ไ", "", "", "aj", false),
    ("", "", "อ", "ɔː", false),
    ("", "", "ำ", "am", false),
    ("", "ั", "", "a", false),
    ("", "", "า", "aː", false),
    ("", "ิ", "", "i", false),
    ("", "ี", "", "iː", false),
    ("", "ึ", "", "ɯ", false),
    // ือ without a leading เ is plain /ɯː/ (มือ, คือ); the อ only carries it
    ("", "ื", "อ", "ɯː", false),
    ("", "ื", "", "ɯː", false),
    ("", "ุ", "", "u", false),
    ("", "ู", "", "uː", false),
    ("", "", "ะ", "a", false),
]);

fn starts_with_at(chars: &[char], i: usize, s: &str) -> Option<usize> {
    let mut k = i;
    for c in s.chars() {
        if k >= chars.len() || chars[k] != c {
            return None;
        }
        k += 1;
    }
    Some(k)
}

struct Reading {
    syls: Vec<String>,
    next: usize,
    cost: u32,
}

/// Readings of one syllable starting at `chars[i]`, using the frame table.
fn frame_readings(chars: &[char], i: usize, forced: Option<Class>) -> Vec<Reading> {
    let mut out = Vec::new();
    for f in FRAMES.iter() {
        let mut k = match starts_with_at(chars, i, f.pre) {
            Some(k) => k,
            None => continue,
        };
        if k >= chars.len() { continue; }
        let c1 = chars[k];
        let onset_base = match INIT.get(&c1) { Some(o) => *o, None => continue };
        k += 1;
        // optional cluster consonant
        let mut onset = onset_base.to_string();
        if k < chars.len() && matches!(chars[k], 'ร' | 'ล' | 'ว') {
            let pair: String = [c1, chars[k]].iter().collect();
            if let Some(p) = PSEUDO.get(pair.as_str()) {
                onset = (*p).to_string();
                k += 1;
            } else if TRUE_CLUSTERS.contains(&pair.as_str()) {
                onset.push_str(INIT[&chars[k]]);
                k += 1;
            }
        }
        k = match starts_with_at(chars, k, f.ab) { Some(k) => k, None => continue };
        let mark = if k < chars.len() && matches!(chars[k], '่' | '้' | '๊' | '๋') {
            let m = chars[k];
            k += 1;
            Some(m)
        } else { None };
        k = match starts_with_at(chars, k, f.post) { Some(k) => k, None => continue };

        let mut vowel = f.vowel.to_string();
        let mut coda = String::new();
        if f.needs_final {
            if k >= chars.len() { continue; }
            match FINAL.get(&chars[k]) {
                Some(c) => { coda = (*c).to_string(); k += 1; }
                None => continue,
            }
        } else {
            match f.vowel {
                "am" => { vowel = "a".into(); coda = "m".into(); }
                "aw" => { vowel = "a".into(); coda = "w".into(); }
                "aj" => {
                    vowel = "a".into();
                    coda = "j".into();
                    // ไ-ย spells a silent ย (ไทย, ไชย)
                    if k < chars.len() && chars[k] == 'ย' { k += 1; }
                }
                _ => {
                    // a following ย/ว is a glide coda unless a vowel follows it
                    if k < chars.len() && matches!(chars[k], 'ย' | 'ว') {
                        let nxt = chars.get(k + 1).copied();
                        let taken = !matches!(nxt, Some(n) if is_dependent(n) || PREPOSED.contains(n));
                        if taken {
                            coda = if chars[k] == 'ย' { "j".into() } else { "w".into() };
                            k += 1;
                        }
                    }
                }
            }
        }
        let cls = forced.unwrap_or_else(|| class_of(c1));
        let t = tone(cls, &vowel, &coda, mark);
        let explicit = !(f.pre.is_empty() && f.ab.is_empty() && f.post.is_empty());
        out.push(Reading {
            syls: vec![format!("{onset}{vowel}{coda}{t}")],
            next: k,
            cost: if explicit { 1 } else { 2 },
        });
    }
    out
}

/// Vowel-less readings: ◌รร (ro han), a lone ร final taking /ɔː/, the
/// inherent /o/ of a closed syllable, and the inherent /a/ of an open one.
fn inherent_readings(chars: &[char], i: usize, forced: Option<Class>) -> Vec<Reading> {
    let mut out = Vec::new();
    let c = chars[i];
    let onset = match INIT.get(&c) { Some(o) => *o, None => return out };
    let cls = forced.unwrap_or_else(|| class_of(c));

    if chars.get(i + 1) == Some(&'ร') && chars.get(i + 2) == Some(&'ร') {
        match chars.get(i + 3) {
            Some(f) if FINAL.contains_key(f) && !is_dependent(*f) => {
                let coda = FINAL[f];
                out.push(Reading {
                    syls: vec![format!("{onset}a{coda}{}", tone(cls, "a", coda, None))],
                    next: i + 4, cost: 1,
                });
            }
            _ => out.push(Reading {
                syls: vec![format!("{onset}an{}", tone(cls, "a", "n", None))],
                next: i + 3, cost: 1,
            }),
        }
        return out;
    }
    let nxt = chars.get(i + 1).copied();
    let nxt2 = chars.get(i + 2).copied();
    let next_is_vowel = matches!(nxt2, Some(n) if is_dependent(n));
    if nxt == Some('ร') && !next_is_vowel && !matches!(nxt2, Some(n) if FINAL.contains_key(&n)) {
        out.push(Reading {
            syls: vec![format!("{onset}ɔːn{}", tone(cls, "ɔː", "n", None))],
            next: i + 2, cost: 2,
        });
    }
    if let Some(n) = nxt {
        if let Some(coda) = FINAL.get(&n) {
            if !next_is_vowel {
                out.push(Reading {
                    syls: vec![format!("{onset}o{coda}{}", tone(cls, "o", coda, None))],
                    next: i + 2, cost: 2,
                });
            }
        }
    }
    out.push(Reading {
        syls: vec![format!("{onset}a{}", tone(cls, "a", "", None))],
        next: i + 1, cost: 3,
    });
    out
}

/// Silent leading ห (and อ before ย): the letter is not pronounced at all,
/// it only lends its class to the syllable — หลอก is /lɔ̀ːk/, not /hà-lɔ̀ːk/,
/// and อยาก is /jàːk/. Distinct from อักษรนำ below, where the leading
/// consonant IS pronounced with an inherent /a/.
///
/// A preposed vowel is written before the silent letter (เหลือ, ไหน), so the
/// vowel character has to be carried across the deletion.
fn silent_lead_readings(chars: &[char], i: usize) -> Vec<Reading> {
    let mut out = Vec::new();
    let pre_len = if i < chars.len() && PREPOSED.contains(chars[i]) { 1 } else { 0 };
    let li = i + pre_len;
    let (lead, next) = match (chars.get(li), chars.get(li + 1)) {
        (Some(a), Some(b)) => (*a, *b),
        _ => return out,
    };
    let cls = if lead == 'ห' && SONORANT.contains(next) {
        Class::High
    } else if lead == 'อ' && next == 'ย' {
        Class::Mid
    } else {
        return out;
    };
    // rebuild the string without the silent letter, keeping any preposed vowel
    let mut reduced: Vec<char> = Vec::with_capacity(chars.len() - 1);
    reduced.extend_from_slice(&chars[i..i + pre_len]);
    reduced.extend_from_slice(&chars[li + 1..]);
    for r in frame_readings(&reduced, 0, Some(cls)) {
        out.push(Reading { syls: r.syls, next: i + 1 + r.next, cost: r.cost });
    }
    for r in inherent_readings(&reduced, pre_len, Some(cls)) {
        out.push(Reading { syls: r.syls, next: i + 1 + r.next, cost: r.cost });
    }
    out
}

/// อักษรนำ: a high/mid consonant with no vowel, directly before a low-class
/// sonorant. Emits both syllables at once, the second re-toned with the
/// first consonant's class.
fn leading_readings(chars: &[char], i: usize) -> Vec<Reading> {
    let mut out = Vec::new();
    if i + 2 > chars.len() { return out; }
    let (c1, c2) = (chars[i], match chars.get(i + 1) { Some(c) => *c, None => return out });
    if !INIT.contains_key(&c1) || class_of(c1) == Class::Low { return out; }
    // ห and อ before a sonorant are silent, not pronounced with /a/
    if c1 == 'ห' || (c1 == 'อ' && c2 == 'ย') { return out; }
    if !SONORANT.contains(c2) { return out; }
    let pair: String = [c1, c2].iter().collect();
    if TRUE_CLUSTERS.contains(&pair.as_str()) || PSEUDO.contains_key(pair.as_str()) { return out; }
    if i + 2 >= chars.len() { return out; }

    let cls1 = class_of(c1);
    let first = format!("{}a{}", INIT[&c1], tone(cls1, "a", "", None));
    let mut inner = frame_readings(chars, i + 1, Some(cls1));
    inner.extend(inherent_readings(chars, i + 1, Some(cls1)));
    for r in inner {
        let mut syls = vec![first.clone()];
        syls.extend(r.syls);
        out.push(Reading { syls, next: r.next, cost: r.cost + 1 });
    }
    out
}

/// Read any Thai string into canonical syllables. Never fails: unreadable
/// characters are skipped at a cost so the DP still finds a path.
pub fn g2p_word(word: &str) -> Vec<String> {
    let cleaned = strip_silent(word);
    let chars: Vec<char> = cleaned.chars().collect();
    let n = chars.len();
    if n == 0 { return Vec::new(); }

    const INF: u32 = u32::MAX;
    let mut best = vec![INF; n + 1];
    let mut back: Vec<Option<(usize, Vec<String>)>> = vec![None; n + 1];
    best[0] = 0;

    for i in 0..n {
        if best[i] == INF { continue; }
        let mut edges = silent_lead_readings(&chars, i);
        edges.extend(leading_readings(&chars, i));
        edges.extend(frame_readings(&chars, i, None));
        edges.extend(inherent_readings(&chars, i, None));
        if edges.is_empty() {
            edges.push(Reading { syls: Vec::new(), next: i + 1, cost: 10 });
        }
        for e in edges {
            if e.next <= i || e.next > n { continue; }
            let cand = best[i].saturating_add(e.cost);
            if cand < best[e.next] {
                best[e.next] = cand;
                back[e.next] = Some((i, e.syls));
            }
        }
    }
    if best[n] == INF { return Vec::new(); }
    let mut out: Vec<String> = Vec::new();
    let mut k = n;
    while k > 0 {
        let (prev, syls) = back[k].take().expect("reachable by construction");
        let mut head = syls;
        head.extend(out);
        out = head;
        k = prev;
    }
    out
}

/// Drop letters silenced by thanthakhat ( ์) along with the mark itself.
fn strip_silent(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut keep = vec![true; chars.len()];
    for (i, c) in chars.iter().enumerate() {
        if *c == '์' {
            keep[i] = false;
            let mut j = i;
            // the silent letter, plus any vowel sign sitting on it
            while j > 0 {
                j -= 1;
                keep[j] = false;
                if INIT.contains_key(&chars[j]) { break; }
            }
        }
    }
    chars.iter().zip(keep).filter(|(_, k)| *k).map(|(c, _)| *c).collect()
}
