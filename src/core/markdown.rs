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

use fancy_regex::Regex;
use once_cell::sync::Lazy;

// ![alt](url) and [text](url): keep the visible text, drop the target.
static RE_LINK: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"!?\[([^\]\n]+)\]\([^)\s]+(?:\s+\x22[^\x22]*\x22)?\)").unwrap()
});

// A horizontal rule: a line of three or more -, * or _.
static RE_RULE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*([-*_])(?:[ \t]*\1){2,}[ \t]*$").unwrap()
});

// "# Heading" through "###### Heading".
static RE_HEADING: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]{0,3}#{1,6}[ \t]+").unwrap()
});

// "> quoted text", only when a letter follows.
static RE_QUOTE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*>[ \t]*(?=\p{L})").unwrap()
});

// "- item", "* item", "+ item".
static RE_BULLET: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^[ \t]*[-*+][ \t]+").unwrap()
});

// ***x***, **x**, *x*, ~~x~~ and __x__, _x_: the closing marker must equal the
// opening one, and neither may touch a letter or digit on its outer side.
// Underscore emphasis may not contain an underscore, so an identifier wrapped
// in underscores ("___PROTECTED_EN_TAG_0___") is left for the later passes.
static RE_EMPHASIS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?<![\p{L}\p{N}*_~])(?:(\*{1,3}|~~)(?=\S)([^\n]+?)(?<=\S)\1|(_{1,3})(?=[^\s_])([^_\n]+?)(?<=[^\s_])\3)(?![\p{L}\p{N}*_~])",
    ).unwrap()
});

/// Remove Markdown formatting, keeping the text it formats.
pub fn strip_markdown(text: &str) -> String {
    // Fast path: none of the marker characters present.
    if !text.contains(['*', '_', '~', '#', '[', '>', '-', '+']) {
        return text.to_string();
    }
    let mut out = RE_LINK.replace_all(text, "$1").into_owned();
    out = RE_RULE.replace_all(&out, "").into_owned();
    out = RE_HEADING.replace_all(&out, "").into_owned();
    out = RE_QUOTE.replace_all(&out, "").into_owned();
    out = RE_BULLET.replace_all(&out, "").into_owned();
    // Nested emphasis ("**_x_**") unwraps one layer per pass.
    for _ in 0..3 {
        let next = RE_EMPHASIS.replace_all(&out, "$2$4").into_owned();
        if next == out { break; }
        out = next;
    }
    out
}
