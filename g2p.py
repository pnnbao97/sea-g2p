import os
import re
import logging
import platform
import glob
import sqlite3
import threading
import functools
import base64
from phonemizer import phonemize
from phonemizer.backend.espeak.espeak import EspeakWrapper

# Configure logging
logger = logging.getLogger("sea_g2p.G2P")

@functools.lru_cache(maxsize=10000)
def deobfuscate(encoded_text: str) -> str:
    if not encoded_text: return ""
    try:
        decoded_bytes = base64.b64decode(encoded_text)
        decoded_str = decoded_bytes.decode('utf-8')
        return decoded_str[::-1]
    except (TypeError, ValueError, UnicodeDecodeError):
        return encoded_text

class PhonemeDB:
    """SQLite-based dictionary for fast lookup and low memory usage."""
    def __init__(self, db_path: str):
        self.db_path = db_path
        self._local = threading.local()
        # Cache for lookup results to minimize DB queries
        self._cache_merged = {}
        self._cache_common = {}
        self._cache_missing = set()

    def _get_conn(self):
        if not hasattr(self._local, "conn"):
            if not os.path.exists(self.db_path):
                # Try fallback to project root if not found in package (for dev)
                alt_path = os.path.join(os.path.dirname(os.path.dirname(__file__)), "phone_dict", "phone_dict.db")
                if os.path.exists(alt_path):
                    self.db_path = alt_path
                else:
                    logger.warning(f"Phoneme database not found at {self.db_path} or {alt_path}")
            
            if os.path.exists(self.db_path):
                self._local.conn = sqlite3.connect(self.db_path, check_same_thread=False)
            else:
                logger.warning("Phoneme database not found. G2P will rely entirely on eSpeak-NG fallback (this may be slower).")
                self._local.conn = None # Will rely on espeak fallback
        return self._local.conn

    def lookup_batch(self, words: list[str]) -> tuple[dict, dict]:
        """Fetch multiple words from DB in two logical groups: merged and common (with caching)."""
        if not words: return {}, {}
        
        merged_map = {}
        common_map = {}
        to_query = []
        
        for w in words:
            if w in self._cache_merged:
                merged_map[w] = self._cache_merged[w]
            elif w in self._cache_common:
                common_map[w] = self._cache_common[w]
            elif w in self._cache_missing:
                continue
            else:
                to_query.append(w)
        
        if not to_query:
            return merged_map, common_map

        conn = self._get_conn()
        if not conn: return merged_map, common_map
        
        cursor = conn.cursor()
        chunk_size = 950
        for i in range(0, len(to_query), chunk_size):
            chunk = to_query[i : i + chunk_size]
            placeholders = ','.join(['?'] * len(chunk))

            query_merged = "SELECT word, phone FROM merged WHERE word IN ({})".format(placeholders)
            cursor.execute(query_merged, chunk)
            for word, phone in cursor.fetchall():
                val = deobfuscate(phone)
                merged_map[word] = val
                self._cache_merged[word] = val

            query_common = "SELECT word, vi_phone, en_phone FROM common WHERE word IN ({})".format(placeholders)
            cursor.execute(query_common, chunk)
            for row in cursor.fetchall():
                val = {
                    "vi": deobfuscate(row[1]), 
                    "en": deobfuscate(row[2])
                }
                common_map[row[0]] = val
                self._cache_common[row[0]] = val
        
        # Track words that were not found in any table
        for w in to_query:
            if w not in merged_map and w not in common_map:
                self._cache_missing.add(w)
        
        return merged_map, common_map

class G2P:
    # Compiled Regular Expressions
    RE_PHONEMIZE_MATCH = re.compile(r'(<en>.*?</en>)|(\w+(?:[\'’]\w+)*)|([^\w\s])', re.I | re.U)
    RE_PHONEMIZE_TAG_CONTENT = re.compile(r'(\w+(?:[\'’]\w+)*)|([^\w\s])', re.U)
    RE_PHONEMIZE_TAG_STRIP = re.compile(r'</?en>', flags=re.I)
    RE_PHONEMIZE_PUNCT_CLEANUP = re.compile(r'\s+([.,!?;:])')
    _VI_ACCENTS = "àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ"
    _STOP_PUNCT = {'.', '!', '?', ';', ':', '(', ')', '[', ']', '{', '}'}

    def __init__(self, lang: str = "vi", db_path: str = None):
        self.lang = lang
        if db_path is None:
            db_path = os.path.join(os.path.dirname(__file__), "phone_dict", "phone_dict.db")
        
        self.db = PhonemeDB(db_path)
        self._setup_espeak()

    def _setup_espeak(self):
        self.espeak_available = False
        system = platform.system()
        found = False
        if system == "Windows":
            default_path = r"C:\Program Files\eSpeak NG\libespeak-ng.dll"
            if os.path.exists(default_path):
                EspeakWrapper.set_library(default_path)
                found = True
        elif system == "Linux":
            search_patterns = [
                "/usr/lib/x86_64-linux-gnu/libespeak-ng.so*",
                "/usr/lib/x86_64-linux-gnu/libespeak.so*",
                "/usr/lib/libespeak-ng.so*",
                "/usr/lib64/libespeak-ng.so*",
                "/usr/local/lib/libespeak-ng.so*",
            ]
            for pattern in search_patterns:
                matches = glob.glob(pattern)
                if matches:
                    EspeakWrapper.set_library(sorted(matches, key=len)[0])
                    found = True
                    break
        elif system == "Darwin":
            espeak_lib = os.environ.get('PHONEMIZER_ESPEAK_LIBRARY')
            paths_to_check = [
                espeak_lib,
                "/opt/homebrew/lib/libespeak-ng.dylib",
                "/usr/local/lib/libespeak-ng.dylib",
            ]
            for path in paths_to_check:
                if path and os.path.exists(path):
                    EspeakWrapper.set_library(path)
                    found = True
                    break
        
        if found:
            try:
                # Test call to ensure espeak is actually functional
                phonemize(["check"], language='en-us', backend='espeak')
                self.espeak_available = True
            except Exception:
                self.espeak_available = False
        
        if not self.espeak_available:
            logger.warning("\033[93m⚠️ eSpeak-NG not found. The system will rely on the built-in dictionary (covers ~99.9% words). Performance may be affected for out-of-vocabulary words.\033[0m")

    def _espeak_fallback_batch(self, texts: list[str], language: str = 'en-us') -> list[str]:
        if not texts: return []
        try:
            ph = phonemize(
                texts,
                language=language,
                backend='espeak',
                preserve_punctuation=True,
                with_stress=True,
                language_switch="remove-flags"
            )
            if isinstance(ph, str): ph = [ph]
            res = [p.strip() for p in ph]
            if res == texts: return None
            return res
        except Exception as e:
            logger.warning(f"eSpeak runtime error ({language}): {e}")
            return None

    def _propagate_language(self, tokens):
        if not tokens: return
        n = len(tokens)
        i = 0
        while i < n:
            if tokens[i]['lang'] == 'common':
                start = i
                while i < n and tokens[i]['lang'] == 'common':
                    i += 1
                end = i - 1

                left_anchor, left_dist = None, 999
                right_anchor, right_dist = None, 999

                for l in range(start - 1, -1, -1):
                    if tokens[l]['content'] in self._STOP_PUNCT: break
                    if tokens[l]['lang'] in ('vi', 'en'):
                        left_anchor = tokens[l]['lang']
                        left_dist = start - l
                        break
                
                for r in range(end + 1, n):
                    if tokens[r]['content'] in self._STOP_PUNCT: break
                    if tokens[r]['lang'] in ('vi', 'en'):
                        right_anchor = tokens[r]['lang']
                        right_dist = r - end
                        break

                final_lang = 'vi'
                if left_anchor and right_anchor:
                    final_lang = right_anchor if right_dist <= left_dist else left_anchor
                elif left_anchor:
                    final_lang = left_anchor
                elif right_anchor:
                    final_lang = right_anchor

                for idx in range(start, end + 1):
                    tokens[idx]['lang'] = final_lang
            else:
                i += 1

    def convert(self, text: str, **kwargs) -> str:
        """Single text conversion."""
        return self.phonemize_batch([text], **kwargs)[0]

    def phonemize_batch(self, texts: list[str], phoneme_dict: dict = None, **kwargs) -> list[str]:
        """Phonemize multiple texts with bilingual support."""
        if not texts: return []
        
        custom = phoneme_dict or {}
        batch_token_lists = []
        all_words = set()
        global_unknown = set()
        en_tagged_words = set()

        for text in texts:
            sent_tokens = []
            for m in self.RE_PHONEMIZE_MATCH.finditer(text):
                en_tag, word, punct = m.groups()
                if en_tag:
                    content = self.RE_PHONEMIZE_TAG_STRIP.sub('', en_tag).strip()
                    for st in self.RE_PHONEMIZE_TAG_CONTENT.finditer(content):
                        sw, sp = st.groups()
                        if sp:
                            sent_tokens.append({'lang': 'punct', 'content': sp, 'phone': sp})
                        else:
                            sent_tokens.append({'lang': 'en', 'content': sw, 'phone': None, 'is_en_tag': True})
                            en_tagged_words.add(sw)
                elif punct:
                    sent_tokens.append({'lang': 'punct', 'content': punct, 'phone': punct})
                elif word:
                    sent_tokens.append({'lang': 'unknown', 'content': word, 'phone': None})
                    all_words.add(word.lower())
            batch_token_lists.append(sent_tokens)

        # Include en_tagged_words so we can resolve them from dict before using eSpeak
        all_lookup_words = all_words | {w.lower() for w in en_tagged_words}
        
        # If eSpeak is missing, we will need to split unknown words into characters.
        # Pre-lookup all possible characters to optimize.
        if not self.espeak_available:
            all_chars = set()
            for w in all_lookup_words:
                all_chars.update(list(w))
            all_lookup_words.update(all_chars)

        db_merged, db_common = self.db.lookup_batch(list(all_lookup_words))

        def get_char_phone(char: str, target_lang: str) -> str:
            """Helper to resolve phoneme for a single character."""
            lw = char.lower()
            if lw in custom: return custom[lw]
            if lw in db_merged: 
                return db_merged[lw].replace('<en>', '').replace('</en>', '').strip()
            if lw in db_common:
                res = db_common[lw]
                p = res.get(target_lang) or res.get('vi') or res.get('en') or char
                return p.replace('<en>', '').replace('</en>', '').strip()
            return char

        for sent in batch_token_lists:
            for t in sent:
                if t['lang'] == 'punct' or t.get('is_en_tag'): continue
                lw = t['content'].lower()

                if lw in custom:
                    t['phone'], t['lang'] = custom[lw], 'en'
                elif lw in db_merged:
                    val = db_merged[lw]
                    t['phone'] = val
                    t['lang'] = 'en' if val.startswith('<en>') else 'vi'
                elif lw in db_common:
                    t['phone'], t['lang'] = db_common[lw], 'common'
                else:
                    global_unknown.add(t['content'])
                    t['lang'] = 'en'

        lut = {}
        if en_tagged_words:
            et_words = sorted(list(en_tagged_words))
            still_need_fallback = []

            for w in et_words:
                lw = w.lower()
                if lw in custom:
                    lut[w] = f"<en>{custom[lw]}"
                elif lw in db_merged:
                    val = db_merged[lw]
                    # Strip any existing <en> wrapper and re-apply cleanly
                    clean_val = val.replace('<en>', '').strip()
                    lut[w] = f"<en>{clean_val}"
                elif lw in db_common:
                    common_val = db_common[lw]
                    # Only take English phoneme for words explicitly tagged <en>
                    en_phone = common_val.get('en') if isinstance(common_val, dict) else common_val
                    if en_phone:
                        # Re-apply cleanly
                        clean_en = en_phone.replace('<en>', '').strip()
                        lut[w] = f"<en>{clean_en}"
                    else:
                        still_need_fallback.append(w)
                else:
                    still_need_fallback.append(w)

            # eSpeak/Char fallback chỉ được gọi cho các từ thực sự không có trong từ điển
            if still_need_fallback:
                et_phones = None
                if self.espeak_available:
                    et_phones = self._espeak_fallback_batch(still_need_fallback, 'en-us')
                
                if et_phones:
                    lut.update({w: f"<en>{p}" for w, p in zip(still_need_fallback, et_phones)})
                else:
                    # Fallback for <en> tags: split into characters and lookup 'en' phonemes
                    # Case: eSpeak not available OR eSpeak failed at runtime
                    for w in still_need_fallback:
                        p = "".join(get_char_phone(c, 'en') for c in w)
                        lut[w] = f"<en>{p}"

        if global_unknown:
            u_words = sorted(list(global_unknown))
            def has_accent(w): return any(c in self._VI_ACCENTS for c in w.lower())
            vi_words = [w for w in u_words if has_accent(w)]
            en_words = [w for w in u_words if not has_accent(w)]

            vi_res, en_res = None, None
            if self.espeak_available:
                if vi_words: vi_res = self._espeak_fallback_batch(vi_words, 'vi')
                if en_words: en_res = self._espeak_fallback_batch(en_words, 'en-us')

            if vi_res:
                lut.update(dict(zip(vi_words, vi_res)))
            elif vi_words: # Fallback vi
                for w in vi_words:
                    lut[w] = "".join(get_char_phone(c, 'vi') for c in w)

            if en_res:
                lut.update({w: f"<en>{p}" for w, p in zip(en_words, en_res)})
            elif en_words: # Fallback en
                for w in en_words:
                    lut[w] = f"<en>{"".join(get_char_phone(c, 'en') for c in w)}"

        results = []
        for sent in batch_token_lists:
            for t in sent:
                if t['phone'] is None and t['content'] in lut:
                    t['phone'] = lut[t['content']]
                    if not t.get('is_en_tag') and any(c in self._VI_ACCENTS for c in t['content'].lower()):
                        t['lang'] = 'vi'

            self._propagate_language(sent)

            sent_phones = []
            for t in sent:
                if t['lang'] == 'punct':
                    sent_phones.append(t['phone'])
                else:
                    p = t['phone']
                    if isinstance(p, dict):
                        p = p['en'] if t['lang'] == 'en' else p['vi']
                    if p is None:
                        p = t['content']
                    sent_phones.append(p.replace('<en>', ''))

            txt = self.RE_PHONEMIZE_PUNCT_CLEANUP.sub(r'\1', " ".join(sent_phones)).strip()
            results.append(txt)
        return results
