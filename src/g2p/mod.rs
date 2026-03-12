use memmap2::Mmap;
use std::fs::File;
use std::io;
use regex::Regex;
use once_cell::sync::Lazy;

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

#[derive(Clone)]
pub struct Token {
    pub lang: String, 
    pub content: String,
    pub phone: Option<String>,
}

use std::collections::HashMap;
use std::sync::RwLock;

pub struct G2PEngine {
    pub dict: PhonemeDict,
    merged_cache: RwLock<HashMap<String, Option<String>>>,
    common_cache: RwLock<HashMap<String, Option<(String, String)>>>,
}

impl G2PEngine {
    pub fn new(dict_path: &str) -> io::Result<Self> {
        Ok(Self {
            dict: PhonemeDict::new(dict_path)?,
            merged_cache: RwLock::new(HashMap::with_capacity(2048)),
            common_cache: RwLock::new(HashMap::with_capacity(1024)),
        })
    }

    fn cached_lookup_merged(&self, word: &str) -> Option<String> {
        {
            let read_guard = self.merged_cache.read().unwrap();
            if let Some(cached) = read_guard.get(word) {
                return cached.clone();
            }
        }
        let res = self.dict.lookup_merged(word).map(|s| s.to_string());
        let mut write_guard = self.merged_cache.write().unwrap();
        // Limit cache size to 10k items
        if write_guard.len() < 10000 {
            write_guard.insert(word.to_string(), res.clone());
        }
        res
    }

    fn cached_lookup_common(&self, word: &str) -> Option<(String, String)> {
        {
            let read_guard = self.common_cache.read().unwrap();
            if let Some(cached) = read_guard.get(word) {
                return cached.clone();
            }
        }
        let res = self.dict.lookup_common(word).map(|(v, e)| (v.to_string(), e.to_string()));
        let mut write_guard = self.common_cache.write().unwrap();
        if write_guard.len() < 5000 {
            write_guard.insert(word.to_string(), res.clone());
        }
        res
    }

    pub fn phonemize(&self, text: &str) -> String {
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
                            phone_val = Some(p.replace("<en>", "").replace("</en>", "").trim().to_string());
                        } else if let Some((_, en)) = self.cached_lookup_common(&lw) {
                            if !en.is_empty() {
                                phone_val = Some(en.replace("<en>", "").replace("</en>", "").trim().to_string());
                            }
                        }

                        tokens.push(Token {
                            lang: "en".to_string(),
                            content: word,
                            phone: phone_val,
                        });
                    } else if let Some(sp) = scall.get(2) {
                        tokens.push(Token {
                            lang: "punct".to_string(),
                            content: sp.as_str().to_string(),
                            phone: Some(sp.as_str().to_string()),
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
                        phone: Some(p.replace("<en>", "").replace("</en>", "").trim().to_string()),
                    });
                } else if let Some((vi, en)) = self.cached_lookup_common(&lw) {
                    tokens.push(Token {
                        lang: "common".to_string(),
                        content: word.as_str().to_string(),
                        phone: Some(format!("|{}|{}|", vi, en)), 
                    });
                } else {
                    let has_vi_accent = lw.chars().any(|c| VI_ACCENTS.contains(c));
                    tokens.push(Token {
                        lang: if has_vi_accent { "vi".to_string() } else { "en".to_string() },
                        content: word.as_str().to_string(),
                        phone: None,
                    });
                }
            } else if let Some(punct) = cap.get(3) {
                tokens.push(Token {
                    lang: "punct".to_string(),
                    content: punct.as_str().to_string(),
                    phone: Some(punct.as_str().to_string()),
                });
            }
        }

        self.propagate_language(&mut tokens);
        
        let mut result = Vec::new();
        for t in tokens {
            if t.lang == "punct" {
                result.push(t.content);
            } else {
                let phone = if let Some(p) = t.phone {
                    if p.starts_with('|') && p.ends_with('|') {
                        let parts: Vec<&str> = p[1..p.len()-1].split('|').collect();
                        if t.lang == "en" { parts[1].to_string() } else { parts[0].to_string() }
                    } else {
                        p
                    }
                } else {
                    // Primitive char fallback
                    t.content.chars().map(|c| {
                        let cl = c.to_lowercase().to_string();
                        if let Some(cp) = self.cached_lookup_merged(&cl) {
                            cp.replace("<en>", "").replace("</en>", "").trim().to_string()
                        } else if let Some((v, e)) = self.cached_lookup_common(&cl) {
                            let p = if t.lang == "en" && !e.is_empty() { e } else { if !v.is_empty() { v } else { e } };
                            p.replace("<en>", "").replace("</en>", "").trim().to_string()
                        } else {
                            cl
                        }
                    }).collect::<Vec<String>>().join("")
                };
                result.push(phone.replace("<en>", "").replace("</en>", "").trim().to_string());
            }
        }

        let joined = result.join(" ");
        joined.replace(" .", ".").replace(" ,", ",").replace(" !", "!").replace(" ?", "?")
    }

    fn propagate_language(&self, tokens: &mut Vec<Token>) {
        let n = tokens.len();
        let mut i = 0;
        while i < n {
            if tokens[i].lang == "common" {
                let start = i;
                while i < n && tokens[i].lang == "common" { i += 1; }
                let end = i - 1;

                let mut left_anchor = None;
                let mut left_dist = 999;
                for l in (0..start).rev() {
                    if tokens[l].lang == "punct" && ".!?;:()[]{}".contains(&tokens[l].content) { break; }
                    if tokens[l].lang == "vi" || tokens[l].lang == "en" {
                        left_anchor = Some(tokens[l].lang.clone());
                        left_dist = start - l;
                        break;
                    }
                }

                let mut right_anchor = None;
                let mut right_dist = 999;
                for r in (end + 1)..n {
                    if tokens[r].lang == "punct" && ".!?;:()[]{}".contains(&tokens[r].content) { break; }
                    if tokens[r].lang == "vi" || tokens[r].lang == "en" {
                        right_anchor = Some(tokens[r].lang.clone());
                        right_dist = r - end;
                        break;
                    }
                }

                let final_lang = if let (Some(l), Some(r)) = (left_anchor.as_ref(), right_anchor.as_ref()) {
                    if right_dist <= left_dist { r.clone() } else { l.clone() }
                } else if let Some(l) = left_anchor {
                    l
                } else if let Some(r) = right_anchor {
                    r
                } else {
                    "vi".to_string()
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
