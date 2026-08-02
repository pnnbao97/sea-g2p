use memmap2::Mmap;
use std::fs::File;
use std::io;
use regex::Regex;
use once_cell::sync::Lazy;

pub mod en_top_words;
use en_top_words::EN_TOP_WORDS;

pub struct PhonemeDict {
    mmap: Mmap,
    string_count: u32,
    merged_count: u32,
    common_count: u32,
    string_offsets_pos: usize,
    merged_pos: usize,
    common_pos: usize,
}

impl PhonemeDict {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 32 || &mmap[0..4] != b"SEAP" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid dictionary format"));
        }

        let string_count = u32::from_le_bytes(mmap[8..12].try_into().unwrap());
        let merged_count = u32::from_le_bytes(mmap[12..16].try_into().unwrap());
        let common_count = u32::from_le_bytes(mmap[16..20].try_into().unwrap());

        let string_offsets_pos = u32::from_le_bytes(mmap[20..24].try_into().unwrap()) as usize;
        let merged_pos = u32::from_le_bytes(mmap[24..28].try_into().unwrap()) as usize;
        let common_pos = u32::from_le_bytes(mmap[28..32].try_into().unwrap()) as usize;

        Ok(Self {
            mmap,
            string_count,
            merged_count,
            common_count,
            string_offsets_pos,
            merged_pos,
            common_pos,
        })
    }

    fn get_string(&self, id: u32) -> &str {
        if id >= self.string_count { return ""; }
        let off_ptr = self.string_offsets_pos + (id as usize * 4);
        let offset = u32::from_le_bytes(self.mmap[off_ptr..off_ptr + 4].try_into().unwrap()) as usize;

        let start = 32 + offset;
        let mut end = start;
        while end < self.mmap.len() && self.mmap[end] != 0 {
            end += 1;
        }
        std::str::from_utf8(&self.mmap[start..end]).unwrap_or("")
    }

    pub fn lookup_merged(&self, word: &str) -> Option<&str> {
        let mut low = 0;
        let mut high = self.merged_count as i32 - 1;

        while low <= high {
            let mid = (low + high) / 2;
            let ptr = self.merged_pos + (mid as usize * 8);
            let w_id = u32::from_le_bytes(self.mmap[ptr..ptr + 4].try_into().unwrap());
            let current_word = self.get_string(w_id);

            if current_word == word {
                let p_id = u32::from_le_bytes(self.mmap[ptr + 4..ptr + 8].try_into().unwrap());
                return Some(self.get_string(p_id));
            } else if current_word < word {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        None
    }

    pub fn lookup_common(&self, word: &str) -> Option<(&str, &str)> {
        let mut low = 0;
        let mut high = self.common_count as i32 - 1;

        while low <= high {
            let mid = (low + high) / 2;
            let ptr = self.common_pos + (mid as usize * 12);
            let w_id = u32::from_le_bytes(self.mmap[ptr..ptr + 4].try_into().unwrap());
            let current_word = self.get_string(w_id);

            if current_word == word {
                let vi_id = u32::from_le_bytes(self.mmap[ptr + 4..ptr + 8].try_into().unwrap());
                let en_id = u32::from_le_bytes(self.mmap[ptr + 8..ptr + 12].try_into().unwrap());
                return Some((self.get_string(vi_id), self.get_string(en_id)));
            } else if current_word < word {
                low = mid + 1;
            } else {
                high = mid - 1;
            }
        }
        None
    }
}

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

// Nguyên âm tiếng Anh + tiếng Việt (lowercase, đã include dấu)
static VOWELS: &str = "aeiouyàáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵ";

/// Kiểm tra segment có cả nguyên âm lẫn phụ âm không.
/// Loại "n", "st" (chỉ phụ âm) và "e", "a" (chỉ nguyên âm).
/// Với tiếng Việt, các từ đơn âm thuần nguyên âm như "ơi", "ừ"
/// thường đã có trong dict nên không đi qua segment_oov.
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

/// Ánh xạ một token dấu câu về dạng GIỮ trong chuỗi phoneme, ĐỒNG BỘ với quy
/// tắc của `Normalizer`. Trả `None` nghĩa là bỏ hẳn ký hiệu đó.
///
/// Cần thiết vì nội dung trong tag `<en>` KHÔNG đi qua `Normalizer` (normalizer
/// giữ nguyên nội dung <en>), nên các dấu như `"` `(` `-` sẽ lọt vào phoneme nếu
/// không xử lý ở đây. Quy tắc khớp `Normalizer`:
///   - `, . ! ?`            -> giữ nguyên
///   - `; :`                -> `,`
///   - `… ‥ ․` (ellipsis)   -> `.`
///   - còn lại (nháy `"` `'` `«` `»`, ngoặc `(` `)` `{` `}` `[` `]`,
///     gạch nối rời `-` `–` `—`, ...) -> bỏ
///
/// Token punct luôn là MỘT ký tự (regex `[^\w\s]`).
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
    /// Cache kết quả segment_oov. Key = "{word}_{lang}", value = None nếu không segment được.
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

    /// Resolve phoneme cho một segment đơn từ dict, theo ngữ cảnh lang.
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

    /// DP segmentation cho OOV word, tối ưu theo CHI PHÍ:
    ///   - Segment là TỪ THẬT trong dict (có nguyên âm + phụ âm, phoneme không
    ///     phải kiểu đánh vần) -> giá 1. DP vì thế ưu tiên cách cắt ít mảnh,
    ///     mảnh dài ("vietinbank" -> "viet in bank" thay vì "vi eti en bank").
    ///   - Segment ngắn (<=4 ký tự) mà phoneme có >=2 trọng âm là entry kiểu
    ///     ĐÁNH VẦN acronym trong dict ("mbo" -> em bi ô) -> giá đắt, chỉ dùng
    ///     khi không còn đường nào khác.
    ///   - Đoạn <=3 ký tự KHÔNG có trong dict -> cho phép đánh vần từng chữ
    ///     với giá đắt ("vpbank" -> "vp" đánh vần + "bank"; "chunkr" ->
    ///     "chunk" + "r") thay vì bỏ cả từ sang char_fallback.
    ///   - Hòa giá -> ưu tiên đoạn ĐẦU dài hơn (leftmost-longest).
    fn segment_oov(&self, word: &str, lang: &str) -> Option<String> {
        // Check cache trước
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
        // true nếu a tốt hơn b: giá thấp hơn; hòa -> NHIỀU từ tiếng Anh phổ
        // biến hơn ("fine|tune" thắng "fin|etune", "family|app" thắng
        // "famil|yapp" — entry rác trong dict không nằm trong top wordlist);
        // hòa tiếp -> đoạn CUỐI dài hơn ("vin|homes" thắng "vinho|mes");
        // vẫn hòa -> ít đoạn hơn. (Đã thử tiêu chí "cắt cân đối" nhưng
        // morpheme tiếng Anh không cân đối: nó phá "vin|homes" -> "vinh|omes".)
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
                        // Entry "rác" trong dict: đoạn ngắn mà phoneme nhiều trọng
                        // âm (kiểu đánh vần "mbo" -> em bi ô), hoặc >=2 trọng âm
                        // CHÍNH (entry ghép "enbank" -> en-bank). Từ thật dài có
                        // trọng âm phụ (ˈ + ˌ) không bị tính.
                        if (seg_len <= 4 && total >= 2) || primary >= 2 {
                            cost = JUNK_COST + seg_len as u32;
                        }
                        phone = Some(p);
                    }
                }
                if phone.is_none() && seg_len <= 3 {
                    // Đánh vần từng chữ, giá tăng theo độ dài: 1 chữ cuối rẻ
                    // ("chunk r"), cụm 3 phụ âm giữa từ đắt.
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

        // Cache lại — kể cả None để tránh tính lại
        {
            let mut w = self.segmentation_cache.write().unwrap();
            if w.len() >= 5_000 { w.clear(); }
            w.insert(cache_key, result.clone());
        }

        result
    }

    /// Char-by-char fallback — last resort khi segment_oov cũng thất bại.
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
        // Nháy cong -> nháy thẳng để "i’m" tra được dict ("i'm") khi caller
        // gọi G2P trực tiếp không qua Normalizer.
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
                // Map dấu câu theo quy tắc Normalizer; bỏ nháy/ngoặc/gạch nối rời...
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
                    // 1. DP segmentation với vowel filter
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
        // Gộp dấu câu lặp liên tiếp về một (đồng bộ Normalizer: "..."/"…" -> ".",
        // ",," -> ","). An toàn vì chuỗi phoneme không chứa '.'/','.
        while joined.contains("..") { joined = joined.replace("..", "."); }
        while joined.contains(",,") { joined = joined.replace(",,", ","); }
        joined
    }

    fn propagate_language(&self, tokens: &mut Vec<Token>) {
        let n = tokens.len();
        // Câu không có token tiếng Việt nào -> mặc định cho từ common là EN
        // ("I can do it" toàn từ common không được rơi về đọc kiểu Việt).
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

                // Khoảng cách tới neo đếm theo TOKEN TỪ, bỏ qua dấu câu không chặn
                // (phẩy, nháy...): "OK, go thôi" -> "go" cách "ok" 1 từ, hòa với
                // "thôi" -> đi theo neo EN thay vì bị dấu phẩy đẩy xa neo trái.
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
                        // Hòa khoảng cách: từ common là TỪ thật đứng sát từ tiếng Anh
                        // thường thuộc cụm tiếng Anh đó ("let's go ăn" -> "go" là EN,
                        // "muốn go to market" -> "go to" là EN). Riêng chữ cái đơn lẻ
                        // ("a" trong "a còng", "i" trong "core i chín") giữ ưu tiên
                        // neo phải như cũ để không bị kéo sang EN.
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
