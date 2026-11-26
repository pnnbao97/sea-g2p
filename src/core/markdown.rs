//! Markdown formatting stripped before normalization.
//!
//! Text pasted from a chat model or a README arrives with Markdown in it, and
//! every marker is also a symbol the normalizers read aloud: "**Bảo**" read
//! "sao sao bảo sao sao", "~~x~~" read "khoảng khoảng x", "# Tiêu đề" read
//! "thăng tiêu đề". The markers carry formatting, not words, so they are
//! removed before any language pass sees them.
//!
//! Every rule is anchored to Markdown's own shape so that the same characters
//! used as symbols survive:
//!  - emphasis needs a marker that hugs its text on both sides, preceded and
//!    followed by a non-word character: "5 * 3", "2*3*4" and "a_b_c" stay;
//!  - list bullets, headings and quotes only count at the start of a line, and
//!    a quote must be followed by a letter, so a line opening "> 5 triệu"
//!    keeps its comparison.
//!
//! The patterns use the plain `regex` crate, with no look-around or
//! back-references; the boundary checks those would need are done in code.

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

// ![alt](url) and [text](url): keep the visible text, drop the target.
static RE_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"!?\[([^\]\n]+)\]\([^)\s]+(?:\s+"[^"]*")?\)"#).unwrap()
});

// A horizontal rule: a line of three or more of the same -, * or _.
static RE_RULE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*(?:-(?:[ \t]*-){2,}|\*(?:[ \t]*\*){2,}|_(?:[ \t]*_){2,})[ \t]*$").unwrap()
});

// "# Heading" through "###### Heading".
static RE_HEADING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]{0,3}#{1,6}[ \t]+").unwrap()
});

// "> quoted text", only when a letter follows (kept by the replacement).
static RE_QUOTE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*>[ \t]*(\p{L})").unwrap()
});

// "- item", "* item", "+ item".
static RE_BULLET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*[-*+][ \t]+").unwrap()
});

/// Emphasis markers, longest first so "***x***" is not read as "*" + "**x**" + "*".
/// Underscore emphasis may not contain an underscore, so an identifier wrapped
/// in underscores ("___PROTECTED_EN_TAG_0___") is left for the later passes.
///
/// Each entry carries whether the closing marker may touch a following letter:
/// CommonMark allows it for "*" ("**Bảo**đi") but not for "_" ("snake_case_").
static RE_EMPHASIS: Lazy<Vec<(Regex, bool)>> = Lazy::new(|| {
    ["***", "**", "*", "~~", "___", "__", "_"]
        .iter()
        .map(|m: &&str| {
            let intraword = m.starts_with('*');
            let body = if m.starts_with('_') { r"[^\s_](?:[^_\n]*?[^\s_])?" } else { r"\S(?:[^\n]*?\S)?" };
            let m = regex::escape(m);
            // Group 1 is the character before the opening marker (or nothing at
            // the start of the text); group 2 is the emphasised text.
            (Regex::new(&format!(r"(^|[^\p{{L}}\p{{N}}*_~]){m}({body}){m}")).unwrap(), intraword)
        })
        .collect()
});

fn is_marker(c: char) -> bool {
    matches!(c, '*' | '_' | '~')
}

/// Remove Markdown formatting, keeping the text it formats.
pub fn strip_markdown(text: &str) -> String {
    // Fast path: none of the marker characters present.
    if !text.contains(['*', '_', '~', '#', '[', '>', '-', '+']) {
        return text.to_string();
    }
    let mut out = RE_LINK.replace_all(text, "$1").into_owned();
    out = RE_RULE.replace_all(&out, "").into_owned();
    out = RE_HEADING.replace_all(&out, "").into_owned();
    out = RE_QUOTE.replace_all(&out, "$1").into_owned();
    out = RE_BULLET.replace_all(&out, "").into_owned();
    // Nested emphasis ("**_x_**") unwraps one layer per pass.
    for _ in 0..3 {
        let before = out.clone();
        for (re, intraword) in RE_EMPHASIS.iter() {
            let src = out.clone();
            out = re.replace_all(&src, |caps: &Captures| {
                let whole = caps.get(0).unwrap();
                // The closing marker must not run into another marker, nor
                // into a word unless this marker allows it.
                let next = src[whole.end()..].chars().next();
                if next.is_some_and(|c: char| is_marker(c) || (!intraword && c.is_alphanumeric())) {
                    return whole.as_str().to_string();
                }
                // "**Bảo**đi" was a missing space, not one word: keep them apart.
                let gap = if next.is_some_and(char::is_alphanumeric) { " " } else { "" };
                format!("{}{}{}", &caps[1], &caps[2], gap)
            }).into_owned();
        }
        if out == before { break; }
    }
    out
}
