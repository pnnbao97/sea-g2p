//! Memory-mapped phoneme dictionary ("SEAP" format, versions 1 and 2).
//!
//! Common layout: header, NUL-terminated string pool, then sorted index
//! tables binary-searched at lookup time:
//!
//!   - **merged** — word -> one phoneme string (the word exists in a single
//!     language, or its reading was merged during dictionary cleaning);
//!   - **common** — word -> (Vietnamese, English) phoneme pair, for words
//!     both languages claim; sentence context picks a side at G2P time.
//!
//! Version 2 extends the header from 32 to 48 bytes (the string pool then
//! starts at 48) and adds **language sections**: a `(kind, count, pos)`
//! table of extra per-language word -> phoneme indexes, so scripts that
//! cannot collide with the Latin keyspace (Thai today, more later) get
//! their own namespace instead of being forced into `merged`.
//!
//! The mmap is never modified; `scripts/seap.py` is the writer.

use memmap2::Mmap;
use std::fs::File;
use std::io;

/// Section kinds for version-2 language tables.
pub const SECTION_TH: u32 = 3;
/// Thai word frequencies (value is the count as a decimal string). Feeds the
/// segmenter's unigram cost model; stored beside the pronunciations so the
/// two are always built from the same wordlist.
pub const SECTION_TH_FREQ: u32 = 4;
/// Indonesian word -> phonemes. Latin script like English, but looked up by
/// language rather than by keyspace: "air", "dia" and "media" are words in
/// both languages with different readings, so the two cannot share a table.
pub const SECTION_ID: u32 = 5;

pub struct PhonemeDict {
    mmap: Mmap,
    string_count: u32,
    merged_count: u32,
    common_count: u32,
    string_offsets_pos: usize,
    merged_pos: usize,
    common_pos: usize,
    /// Byte offset the string-pool offsets are relative to: 32 (v1) / 48 (v2).
    string_base: usize,
    /// v2 language sections as (kind, entry_count, table_pos).
    sections: Vec<(u32, u32, usize)>,
}

impl PhonemeDict {
    pub fn new(path: &str) -> io::Result<Self> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        if mmap.len() < 32 || &mmap[0..4] != b"SEAP" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid dictionary format"));
        }
        let version = u32::from_le_bytes(mmap[4..8].try_into().unwrap());

        let string_count = u32::from_le_bytes(mmap[8..12].try_into().unwrap());
        let merged_count = u32::from_le_bytes(mmap[12..16].try_into().unwrap());
        let common_count = u32::from_le_bytes(mmap[16..20].try_into().unwrap());

        let string_offsets_pos = u32::from_le_bytes(mmap[20..24].try_into().unwrap()) as usize;
        let merged_pos = u32::from_le_bytes(mmap[24..28].try_into().unwrap()) as usize;
        let common_pos = u32::from_le_bytes(mmap[28..32].try_into().unwrap()) as usize;

        let (string_base, sections) = if version >= 2 {
            if mmap.len() < 48 {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated v2 header"));
            }
            let section_count = u32::from_le_bytes(mmap[32..36].try_into().unwrap()) as usize;
            let sections_pos = u32::from_le_bytes(mmap[36..40].try_into().unwrap()) as usize;
            let mut sections = Vec::with_capacity(section_count);
            for i in 0..section_count {
                let p = sections_pos + i * 12;
                if p + 12 > mmap.len() {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Truncated section table"));
                }
                let kind = u32::from_le_bytes(mmap[p..p + 4].try_into().unwrap());
                let count = u32::from_le_bytes(mmap[p + 4..p + 8].try_into().unwrap());
                let pos = u32::from_le_bytes(mmap[p + 8..p + 12].try_into().unwrap()) as usize;
                sections.push((kind, count, pos));
            }
            (48usize, sections)
        } else {
            (32usize, Vec::new())
        };

        Ok(Self {
            mmap,
            string_count,
            merged_count,
            common_count,
            string_offsets_pos,
            merged_pos,
            common_pos,
            string_base,
            sections,
        })
    }

    fn get_string(&self, id: u32) -> &str {
        if id >= self.string_count { return ""; }
        let off_ptr = self.string_offsets_pos + (id as usize * 4);
        let offset = u32::from_le_bytes(self.mmap[off_ptr..off_ptr + 4].try_into().unwrap()) as usize;

        let start = self.string_base + offset;
        let mut end = start;
        while end < self.mmap.len() && self.mmap[end] != 0 {
            end += 1;
        }
        std::str::from_utf8(&self.mmap[start..end]).unwrap_or("")
    }

    /// Every key of a language section, in stored (sorted) order. Used to
    /// build the Thai segmenter's word trie from the dictionary itself.
    pub fn section_keys(&self, kind: u32) -> Vec<&str> {
        match self.sections.iter().find(|s| s.0 == kind) {
            None => Vec::new(),
            Some(&(_, count, pos)) => (0..count as usize)
                .map(|i| {
                    let ptr = pos + i * 8;
                    let w_id = u32::from_le_bytes(self.mmap[ptr..ptr + 4].try_into().unwrap());
                    self.get_string(w_id)
                })
                .collect(),
        }
    }

    /// Every `(key, value)` pair of a language section, in stored order.
    pub fn section_entries(&self, kind: u32) -> Vec<(&str, &str)> {
        match self.sections.iter().find(|s| s.0 == kind) {
            None => Vec::new(),
            Some(&(_, count, pos)) => (0..count as usize)
                .map(|i| {
                    let ptr = pos + i * 8;
                    let w = u32::from_le_bytes(self.mmap[ptr..ptr + 4].try_into().unwrap());
                    let v = u32::from_le_bytes(self.mmap[ptr + 4..ptr + 8].try_into().unwrap());
                    (self.get_string(w), self.get_string(v))
                })
                .collect(),
        }
    }

    /// Look a word up in a v2 language section (8-byte rows, same shape as
    /// `merged`). Returns `None` on a v1 file or an absent section.
    pub fn lookup_section(&self, kind: u32, word: &str) -> Option<&str> {
        let &(_, count, pos) = self.sections.iter().find(|s| s.0 == kind)?;
        let mut low = 0;
        let mut high = count as i32 - 1;
        while low <= high {
            let mid = (low + high) / 2;
            let ptr = pos + (mid as usize * 8);
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

    /// Does the English side of the dictionary actually contain `word`?
    ///
    /// This is a membership test, not a lookup through the G2P engine: the
    /// engine segments any unknown string into pieces and always returns
    /// something, so asking it "is this English?" answers yes for every
    /// input. A non-English pipeline routing on that reads the Indonesian
    /// name Gadjah as the English "gad jah".
    pub fn has_english(&self, word: &str) -> bool {
        self.lookup_merged(word).is_some_and(|p| p.starts_with("<en>"))
            || self.lookup_common(word).is_some()
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
