use fancy_regex::{Regex, Captures};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::sync::OnceLock;
use crate::g2p::PhonemeDict;
use crate::vi_normalizer::num2vi::{n2w, n2w_single};
use crate::vi_normalizer::resources::{VI_LETTER_NAMES, COMMON_EMAIL_DOMAINS, DOMAIN_SUFFIX_MAP};

// Dict phoneme dùng chung với G2P (mmap, rẻ) — để normalizer tra "từ này có
// trong dict không" khi quyết định giữ nguyên hay tách âm tiết trong path/email.
static NORM_DICT: OnceLock<PhonemeDict> = OnceLock::new();

pub fn init_norm_dict(path: &str) {
    if NORM_DICT.get().is_some() { return; }
    if let Ok(d) = PhonemeDict::new(path) {
        let _ = NORM_DICT.set(d);
    }
}

fn dict_has(word: &str) -> bool {
    NORM_DICT.get()
        .map(|d| d.lookup_merged(word).is_some() || d.lookup_common(word).is_some())
        .unwrap_or(false)
}

// ── Đọc path/URL/email kiểu Việt khi câu chứa từ tiếng Việt ──────────────────
// Âm đầu tiếng Việt dạng KHÔNG DẤU ("đ" gộp về "d"). Chuỗi dài xếp trước để
// thử khớp trước; "" cuối cùng cho âm tiết không có âm đầu ("an", "uong").
static VI_ONSETS: &[&str] = &[
    "ngh", "ch", "gh", "gi", "kh", "ng", "nh", "ph", "qu", "th", "tr",
    "b", "c", "d", "g", "h", "k", "l", "m", "n", "p", "r", "s", "t", "v", "x", "",
];

// Vần tiếng Việt dạng không dấu (gộp ă/â->a, ê->e, ô/ơ->o, ư->u, các biến thể
// có dấu quy về cùng skeleton). Chỉ cần đúng ở mức "trông như âm tiết Việt".
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
        "uy", "uya", "uych", "uyen", "uyet", "uynh", "uyt", "uyu",
        "y", "yem", "yen", "yet", "yeu",
    ].into_iter().collect()
});

// Từ tiếng Anh hay gặp trong path/domain/email — đọc như từ Anh, KHÔNG tách âm
// tiết Việt (nhiều từ trong số này vô tình tách được: "home"->"ho me",
// "compose"->"com po se", "data"->"da ta"...).
static PATH_EN_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "home", "user", "users", "admin", "administrator", "root", "desktop",
        "documents", "document", "download", "downloads", "upload", "uploads",
        "pictures", "picture", "videos", "video", "music", "public", "appdata",
        "roaming", "local", "temp", "tmp", "windows", "system", "program",
        "programs", "files", "file", "data", "database", "backup", "backups",
        "config", "configs", "settings", "cache", "log", "logs", "mail",
        "archive", "archives", "script", "scripts", "docker", "compose", "api",
        "app", "apps", "demo", "patch", "update", "updates", "setup", "install",
        "src", "lib", "bin", "dist", "build", "test", "tests", "debug",
        "release", "releases", "deploy", "assets", "images", "image", "img",
        "docs", "doc", "media", "static", "template", "templates", "project",
        "projects", "report", "reports", "export", "import", "output", "input",
        "readme", "index", "main", "server", "client", "share", "shared",
        "drive", "google", "domain", "example", "gmail", "hotmail", "yahoo",
        "outlook", "proton", "contact", "support", "info", "office", "sale",
        "sales", "base", "game", "games", "name", "page", "pages", "site",
        "mode", "note", "notes", "core", "save", "phone", "camera", "code",
        "node", "web", "etc", "var", "opt", "usr", "openai", "python",
        "github", "gitlab", "youtube", "facebook", "localhost",
    ].into_iter().collect()
});

// Phần mở rộng file: đứng sau "chấm" thì giữ cách đọc kiểu Anh hiện tại
// (kể cả trong câu tiếng Việt) — "chấm p y", "chấm jpg"...
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

/// Tách chuỗi ASCII thường thành dãy âm tiết Việt không dấu ("thongbao" ->
/// ["thong","bao"]). Backtracking từ trái, ưu tiên âm tiết DÀI trước
/// ("nauan" -> "nau an" chứ không phải "na uan"); trả None nếu không phủ hết.
fn split_vi_syllables(s: &str) -> Option<Vec<String>> {
    fn go(s: &str, i: usize, out: &mut Vec<String>, dead: &mut Vec<bool>) -> bool {
        if i == s.len() { return true; }
        if dead[i] { return false; }
        for j in ((i + 1)..=s.len().min(i + 7)).rev() {
            let seg = &s[i..j];
            if is_vi_syllable(seg) {
                out.push(seg.to_string());
                if go(s, j, out, dead) { return true; }
                out.pop();
            }
        }
        dead[i] = true;
        false
    }
    if s.is_empty() || !s.is_ascii() { return None; }
    let mut out = Vec::new();
    let mut dead = vec![false; s.len()];
    if go(s, 0, &mut out, &mut dead) { Some(out) } else { None }
}

fn vi_letter_names(s: &str) -> String {
    s.chars().map(|c: char| {
        let cl = c.to_lowercase().to_string();
        VI_LETTER_NAMES.get(cl.as_str()).map(|v| v.to_string()).unwrap_or(cl)
    }).collect::<Vec<String>>().join(" ")
}

/// Cách đọc kiểu Anh hiện hành cho một cụm chữ cái (giữ nguyên hành vi cũ):
/// ALL-CAPS ngắn hoặc <=2 ký tự -> đánh vần chữ cái Anh, còn lại đọc như từ.
fn en_chunk(t: &str) -> String {
    let mut val = t.to_lowercase();
    if (t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4) || t.len() <= 2 {
        val = val.chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ");
    }
    format!("__start_en__{}__end_en__", val)
}

/// Bản cho email: hành vi cũ là luôn đọc như TỪ tiếng Anh (không đánh vần
/// token ngắn), nên chỉ thêm nhánh tiếng Việt khi vi_ctx.
fn norm_letter_chunk_email(t: &str, vi_ctx: bool, _en_ctx: bool) -> String {
    let lw = t.to_lowercase();
    if !vi_ctx { return format!("__start_en__{}__end_en__", lw); }
    // Chữ cái đơn / toàn phụ âm -> tên chữ Việt (trước dict, như path).
    if lw.chars().count() == 1 || !lw.chars().any(|c: char| "aeiouy".contains(c)) {
        return vi_letter_names(&lw);
    }
    if dict_has(&lw) { return lw; }
    if PATH_EN_WORDS.contains(lw.as_str()) { return lw; }
    if let Some(sylls) = split_vi_syllables(&lw) {
        return sylls.join(" ");
    }
    // Từ lạ: để trần cho G2P tra dict / đọc OOV kiểu Anh.
    lw
}

/// Đọc một cụm chữ cái trong path/URL/email.
/// `vi_ctx`: câu chứa từ tiếng Việt -> ưu tiên đọc kiểu Việt: tách âm tiết
/// không dấu ("thongbao" -> "thong bao"); toàn phụ âm -> tên chữ Việt
/// ("mn" -> "mờ nờ"); từ Anh quen thuộc vẫn đọc kiểu Anh.
fn norm_letter_chunk(t: &str, vi_ctx: bool, after_dot: bool) -> String {
    if !vi_ctx { return en_chunk(t); }
    let lw = t.to_lowercase();
    // Đuôi file quen thuộc: có nguyên âm thật -> để trần đọc như từ ("zip",
    // "yaml"); toàn phụ âm (y không tính) -> tên chữ Việt ("py" -> "phê y",
    // "jpg" -> "giây phê gờ").
    if after_dot && FILE_EXTS.contains(lw.as_str()) {
        if lw.chars().any(|c: char| "aeiou".contains(c)) { return lw; }
        return vi_letter_names(&lw);
    }
    // ALL-CAPS ngắn coi là acronym (TTS, GPU) -> tên chữ Việt "tê tê ét".
    if t.chars().all(|c: char| c.is_uppercase()) && t.len() <= 4 && t.len() >= 2 {
        return vi_letter_names(&lw);
    }
    // Chữ cái đơn ("v" trong v2, "c" trong C:) và cụm toàn phụ âm ("www",
    // "mn", "db") -> tên chữ Việt, TRƯỚC khi tra dict để "www"/"v" không bị
    // dict nuốt mất ("vê kép vê kép vê kép", "vê", "xê", "mờ nờ").
    if lw.chars().count() == 1 || !lw.chars().any(|c: char| "aeiouy".contains(c)) {
        return vi_letter_names(&lw);
    }
    // Có trong dict sea-g2p -> để TRẦN (không tag): G2P tự đọc theo dict
    // (merged EN đọc kiểu Anh, từ common ưu tiên ngữ cảnh Việt xung quanh).
    if dict_has(&lw) { return lw; }
    // Fallback khi dict chưa nạp: whitelist từ Anh hay gặp trong path.
    if PATH_EN_WORDS.contains(lw.as_str()) { return lw; }
    // camelCase mang sẵn ranh giới âm tiết ("CanHoMau" -> Can|Ho|Mau): nếu mọi
    // mảnh đều là âm tiết Việt thì dùng luôn, khỏi đoán bằng backtracking.
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
    if let Some(sylls) = split_vi_syllables(&lw) {
        return sylls.join(" ");
    }
    // Từ lạ ("pnnbao"): để trần, G2P tự tra dict / đọc OOV kiểu Anh.
    lw
}

static RE_TECH_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([./:?&=/_ \-\\#])").unwrap());
static RE_EMAIL_SPLIT: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"([._\-+])").unwrap());
static RE_SUB_TOKENS: Lazy<regex::Regex> = Lazy::new(|| regex::Regex::new(r"[a-zA-Z]+|\d+").unwrap());

pub static RE_TECHNICAL: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?ix)
    \b(?:https?|ftp)://[\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b(?:www\.)[\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b[A-Za-z0-9.\-]+(?:\.com|\.vn|\.net|\.org|\.gov|\.io|\.biz|\.info)(?:/[\p{L}0-9.\-_~:/?#\[\]@!$&\'()*+,;=]*)?\b
    |
    (?<![\w\\])\\\\[a-zA-Z0-9._\-]+(?:\\[\p{L}0-9._\-]+)*\\?
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
                // "https" -> "hát tê tê phê ét", "ftp" -> "ép tê phê"
                res.push(vi_letter_names(&protocol.to_lowercase()));
            } else {
                let p_norm = if (protocol.chars().all(|c: char| c.is_uppercase()) && protocol.len() <= 4) || protocol.len() <= 3 {
                    protocol.to_lowercase().chars().map(|c: char| c.to_string()).collect::<Vec<String>>().join(" ")
                } else {
                    protocol.to_lowercase()
                };
                res.push(format!("__start_en__{}__end_en__", p_norm));
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
                    // Suffix map ("com", "o rờ gờ"...) chỉ dùng ngoài câu thuần Anh.
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
                "?" => res.push(if en_ctx { "question mark" } else { "hỏi" }.to_string()),
                "&" => res.push(if en_ctx { "and" } else { "và" }.to_string()),
                "=" => res.push(if en_ctx { "equals" } else { "bằng" }.to_string()),
                "#" => res.push(if en_ctx { "hash" } else { "thăng" }.to_string()),
                _ => {
                    // Đoạn path chứa chữ tiếng Việt (có dấu) -> đọc như TỪ tiếng Việt,
                    // không spell từng ký tự (vd ".../báo-cáo" -> "báo" "cáo").
                    if s.chars().any(|c: char| c.is_alphabetic() && !c.is_ascii()) {
                        res.push(s.to_lowercase());
                    } else if let Some(suffix) = DOMAIN_SUFFIX_MAP.get(s.to_lowercase().as_str()) {
                        res.push(suffix.to_string());
                    } else if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                        // Câu thuần Anh: chữ số đọc từng số kiểu Anh ("127" -> "one two seven").
                        let digits = |d: &str| -> String {
                            if en_ctx {
                                crate::vi_normalizer::num2en::n2w_en_digits(d)
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
                return if en_ctx { crate::vi_normalizer::num2en::n2w_en(s) } else { n2w(s) };
            }
            if s.chars().all(|c: char| c.is_alphanumeric() && c.is_ascii()) {
                let re_sub = &*RE_SUB_TOKENS;
                let sub_tokens: Vec<&str> = re_sub.find_iter(s).map(|m: regex::Match| m.as_str()).collect();
                if sub_tokens.len() > 1 {
                    let mut res_parts = Vec::new();
                    for t in sub_tokens {
                        if t.chars().all(|c: char| c.is_ascii_digit()) {
                            res_parts.push(if en_ctx { crate::vi_normalizer::num2en::n2w_en(t) } else { n2w(t) });
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
        // Map domain quen thuộc chứa "chấm" tiếng Việt -> chỉ dùng ngoài câu thuần Anh.
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
                    crate::vi_normalizer::resources::VI_LETTER_NAMES
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
