import os
import re
import logging
import platform
import glob
import sqlite3
import threading
import functools
from phonemizer import phonemize
from phonemizer.backend.espeak.espeak import EspeakWrapper

import base64

# Configure logging
logger = logging.getLogger("sea_g2p.G2P")

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
        """Fetch multiple words from DB in two logical groups: merged and common."""
        if not words: return {}, {}
        conn = self._get_conn()
        if not conn: return {}, {}
        
        cursor = conn.cursor()
        merged_map = {}
        common_map = {}
        
        chunk_size = 950
        for i in range(0, len(words), chunk_size):
            chunk = words[i : i + chunk_size]
            placeholders = ','.join(['?'] * len(chunk))

            query_merged = "SELECT word, phone FROM merged WHERE word IN ({})".format(placeholders)
            cursor.execute(query_merged, chunk)
            for word, phone in cursor.fetchall():
                merged_map[word] = deobfuscate(phone)

            query_common = "SELECT word, vi_phone, en_phone FROM common WHERE word IN ({})".format(placeholders)
            cursor.execute(query_common, chunk)
            for row in cursor.fetchall():
                common_map[row[0]] = {
                    "vi": deobfuscate(row[1]), 
                    "en": deobfuscate(row[2])
                }
        
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
        
        if not found:
            logger.warning("\033[93m⚠️ eSpeak-NG not found. The system will rely on the built-in dictionary (covers ~99.9% words). Install eSpeak-NG for full fallback support.\033[0m")

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
            return [p.strip() for p in ph]
        except Exception as e:
            logger.warning(f"eSpeak fallback ({language}) failed: {e}")
            return texts

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
        force_espeak_words = set()

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
                            sent_tokens.append({'lang': 'en', 'content': sw, 'phone': None, 'force_espeak': True})
                            force_espeak_words.add(sw)
                elif punct:
                    sent_tokens.append({'lang': 'punct', 'content': punct, 'phone': punct})
                elif word:
                    sent_tokens.append({'lang': 'unknown', 'content': word, 'phone': None})
                    all_words.add(word.lower())
            batch_token_lists.append(sent_tokens)

        # Include force_espeak_words so we can resolve them from dict before using eSpeak
        all_lookup_words = all_words | {w.lower() for w in force_espeak_words}
        db_merged, db_common = self.db.lookup_batch(list(all_lookup_words))

        for sent in batch_token_lists:
            for t in sent:
                if t['lang'] == 'punct' or t.get('force_espeak'): continue
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
        if force_espeak_words:
            fe_words = sorted(list(force_espeak_words))
            still_need_espeak = []

            for w in fe_words:
                lw = w.lower()
                if lw in custom:
                    lut[w] = f"<en>{custom[lw]}"
                elif lw in db_merged:
                    val = db_merged[lw]
                    # Strip any existing <en> wrapper and re-apply cleanly
                    clean_val = val.replace('<en>', '').replace('</en>', '').strip()
                    lut[w] = f"<en>{clean_val}"
                elif lw in db_common:
                    common_val = db_common[lw]
                    if isinstance(common_val, dict):
                        # Prefer English phoneme for words explicitly tagged <en>
                        en_phone = common_val.get('en') or common_val.get('vi') or w
                        lut[w] = f"<en>{en_phone}"
                    else:
                        lut[w] = f"<en>{common_val}"
                else:
                    still_need_espeak.append(w)

            # eSpeak chỉ được gọi cho các từ thực sự không có trong từ điển
            if still_need_espeak:
                fe_phones = self._espeak_fallback_batch(still_need_espeak, 'en-us')
                if fe_phones and fe_phones != still_need_espeak:
                    lut.update({w: f"<en>{p}" for w, p in zip(still_need_espeak, fe_phones)})
                else:
                    lut.update({w: p for w, p in zip(still_need_espeak, fe_phones)})

        if global_unknown:
            u_words = sorted(list(global_unknown))
            def has_accent(w): return any(c in self._VI_ACCENTS for c in w.lower())
            vi_words = [w for w in u_words if has_accent(w)]
            en_words = [w for w in u_words if not has_accent(w)]

            if vi_words:
                res_vi = self._espeak_fallback_batch(vi_words, 'vi')
                lut.update(dict(zip(vi_words, res_vi)))
            if en_words:
                res_en = self._espeak_fallback_batch(en_words, 'en-us')
                lut.update({w: f"<en>{p}" for w, p in zip(en_words, res_en)})

        results = []
        for sent in batch_token_lists:
            for t in sent:
                if t['phone'] is None and t['content'] in lut:
                    t['phone'] = lut[t['content']]
                    if not t.get('force_espeak') and any(c in self._VI_ACCENTS for c in t['content'].lower()):
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
