import re
from .num2vi import n2w, n2w_single
from .vi_resources import (
    _vi_letter_names, _letter_key_vi, _acronyms_exceptions_vi, _technical_terms,
    _DOMAIN_SUFFIX_MAP, _ROMAN_NUMERALS, _ABBRS, _SYMBOLS_MAP, WORD_LIKE_ACRONYMS
)
from .technical import normalize_slashes, _DOMAIN_SUFFIXES_RE, RE_SLASH_NUMBER

RE_ROMAN_NUMBER = re.compile(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b")
RE_LETTER = re.compile(r"(chữ|chữ cái|kí tự|ký tự)\s+(['\"]?)([a-z])(['\"]?)\b", re.IGNORECASE)
RE_STANDALONE_LETTER = re.compile(r'(?<![\'’])\b([a-zA-Z])\b(\.?)')
RE_SENTENCE_SPLIT = re.compile(r'([.!?]+(?:\s+|$))')
RE_ACRONYM = re.compile(r'\b(?=[A-Z0-9]*[A-Z])[A-Z0-9]{2,}\b')
RE_ALPHANUMERIC = re.compile(r'\b(\d+)([a-zA-Z])\b')
RE_BRACKETS = re.compile(r'[\(\[\{]\s*(.*?)\s*[\)\]\}]')
RE_STRIP_BRACKETS = re.compile(r'[\[\]\(\)\{\}]')
RE_TEMP_C_NEG = re.compile(r'-(\d+(?:[.,]\d+)?)\s*°\s*c\b', re.IGNORECASE)
RE_TEMP_F_NEG = re.compile(r'-(\d+(?:[.,]\d+)?)\s*°\s*f\b', re.IGNORECASE)
RE_TEMP_C = re.compile(r'(\d+(?:[.,]\d+)?)\s*°\s*c\b', re.IGNORECASE)
RE_TEMP_F = re.compile(r'(\d+(?:[.,]\d+)?)\s*°\s*f\b', re.IGNORECASE)
RE_DEGREE = re.compile(r'°')
RE_VERSION = re.compile(r'\b(\d+(?:\.\d+)+)\b')
RE_CLEAN_OTHERS = re.compile(r'[^a-zA-Z0-9\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữựỳýỷỹỵđÀÁẢÃẠĂẮẰẲẴẶÂẤẦẨẪẬÈÉẺẼẸÊẾỀỂỄỆÌÍỈĨỊÒÓỎÕỌÔỐỒỔỖỘƠỚỜỞỠỢÙÚỦŨỤƯỨỪỬỮỰỲÝỶỸỴĐ.,!?_\'’]')
RE_CLEAN_QUOTES = re.compile(r'["“”"]')
RE_CLEAN_QUOTES_EDGES = re.compile(r"(^|\s)['’]+|['’]+($|\s)")
RE_PRIME = re.compile(r"(\b[a-zA-Z0-9])['’](?!\w)")
RE_COLON_SEMICOLON = re.compile(r'[:;]')
RE_UNIT_POWERS = re.compile(r'\b([a-zA-Z]+)\^([-+]?\d+)\b')

from .vi_resources import _combined_exceptions

# Optimized single-pass acronym/exception regex
RE_ACRONYMS_EXCEPTIONS = re.compile(
    "|".join(rf"\b{re.escape(k)}\b" for k in sorted(_combined_exceptions.keys(), key=len, reverse=True))
)

def expand_roman(match):
    num = match.group(0).upper()
    if not num: return ""
    result = 0
    for i, c in enumerate(num):
        if (i + 1) == len(num) or _ROMAN_NUMERALS[c] >= _ROMAN_NUMERALS[num[i + 1]]:
            result += _ROMAN_NUMERALS[c]
        else:
            result -= _ROMAN_NUMERALS[c]
    return f" {n2w(str(result))} "

def expand_unit_powers(text):
    from .units import _measurement_key_vi, _currency_key
    def _repl(m):
        base = m.group(1)
        power = m.group(2)
        power_norm = ("trừ " + n2w(power[1:])) if power.startswith('-') else n2w(power.replace('+', ''))
        base_lower = base.lower()
        full_base = _measurement_key_vi.get(base_lower, _currency_key.get(base_lower, base))
        return f" {full_base} mũ {power_norm} "
    return RE_UNIT_POWERS.sub(_repl, text)

def expand_letter(match):
    prefix, q1, char, q2 = match.groups()
    if char.lower() in _letter_key_vi:
        return f"{prefix} {_letter_key_vi[char.lower()]} "
    return match.group(0)

def expand_abbreviations(text):
    for k, v in _ABBRS.items():
        text = text.replace(k, v)
    return text

def expand_standalone_letters(text):
    def _repl_letter(m):
        char_raw = m.group(1)
        char = char_raw.lower()
        dot = m.group(2) or ""
        if char in _letter_key_vi:
            if char_raw.isupper() and dot == '.':
                return f" {_letter_key_vi[char]} "
            return f" {_letter_key_vi[char]}{dot} "
        return m.group(0)
    return RE_STANDALONE_LETTER.sub(_repl_letter, text)

def normalize_acronyms(text):
    sentences = RE_SENTENCE_SPLIT.split(text)
    processed = []
    for i in range(0, len(sentences), 2):
        s = sentences[i]
        sep = sentences[i+1] if i+1 < len(sentences) else ""
        if not s:
            processed.append(sep)
            continue
        words = s.split()
        alpha_words = [w for w in words if any(c.isalpha() for c in w)]
        is_all_caps = len(alpha_words) > 0 and all(w.isupper() for w in alpha_words)
        if not is_all_caps:
            def _repl_acronym(m):
                word = m.group(0)
                if word.isdigit(): return word
                if word in WORD_LIKE_ACRONYMS:
                    return f"__start_en__{word.lower()}__end_en__"
                if any(c.isdigit() for c in word):
                    res = [n2w_single(c) if c.isdigit() else _vi_letter_names.get(c, c) for c in word.lower()]
                    return " ".join(res)
                spaced_word = " ".join(c.lower() for c in word if c.isalnum())
                return f"__start_en__{spaced_word}__end_en__" if spaced_word else word
            s = RE_ACRONYM.sub(_repl_acronym, s)
        processed.append(s + sep)
    return "".join(processed)

def expand_alphanumeric(text):
    def _repl(m):
        num = m.group(1)
        char = m.group(2).lower()
        if char in _letter_key_vi:
            pronunciation = _letter_key_vi[char]
            if char == 'd' and ('quốc lộ' in text.lower() or 'ql' in text.lower()):
                pronunciation = 'đê'
            return f"{num} {pronunciation}"
        return m.group(0)
    return RE_ALPHANUMERIC.sub(_repl, text)

def expand_symbols(text):
    for s, v in _SYMBOLS_MAP.items():
        text = text.replace(s, v)
    return text

def expand_prime(text):
    def _repl(m):
        val = m.group(1).lower()
        return f"{n2w_single(val) if val.isdigit() else _letter_key_vi.get(val, val)} phẩy"
    return RE_PRIME.sub(_repl, text)

def expand_temperatures(text):
    text = RE_TEMP_C_NEG.sub(r'âm \1 độ xê', text)
    text = RE_TEMP_F_NEG.sub(r'âm \1 độ ép', text)
    text = RE_TEMP_C.sub(r'\1 độ xê', text)
    text = RE_TEMP_F.sub(r'\1 độ ép', text)
    return RE_DEGREE.sub(' độ ', text)

def normalize_others(text):
    # 1. Expand acronym exceptions in a single pass
    text = RE_ACRONYMS_EXCEPTIONS.sub(lambda m: _combined_exceptions[m.group(0)], text)
    text = normalize_slashes(text)
    text = _DOMAIN_SUFFIXES_RE.sub(lambda m: " chấm " + _DOMAIN_SUFFIX_MAP.get(m.group(1).lower(), m.group(1).lower()), text)

    # 2. Basic patterns
    text = RE_ROMAN_NUMBER.sub(expand_roman, text)
    text = RE_LETTER.sub(expand_letter, text)
    text = expand_alphanumeric(text)

    # 3. Symbols and quotes
    text = expand_prime(text)
    text = expand_unit_powers(text)
    text = RE_CLEAN_QUOTES.sub('', text)
    text = RE_CLEAN_QUOTES_EDGES.sub(r"\1 \2", text)
    text = expand_symbols(text)

    # 4. Misc
    text = RE_BRACKETS.sub(r', \1, ', text)
    text = RE_STRIP_BRACKETS.sub(' ', text)
    text = expand_temperatures(text)
    text = normalize_acronyms(text)

    def _expand_version(m):
        res = [" ".join(n2w_single(c) for c in s) for s in m.group(1).split('.')]
        return ' chấm '.join(res)
    text = RE_VERSION.sub(_expand_version, text)
    text = RE_COLON_SEMICOLON.sub(',', text)
    return RE_CLEAN_OTHERS.sub(' ', text)
