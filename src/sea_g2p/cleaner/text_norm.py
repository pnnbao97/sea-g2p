import re
from .num2vi import n2w, n2w_single
from .symbols import vietnamese_re, vietnamese_without_num_re

_vi_letter_names = {
    "a": "a", "b": "bê", "c": "xê", "d": "đê", "đ": "đê", "e": "e", "ê": "ê",
    "f": "ép", "g": "gờ", "h": "hát", "i": "i", "j": "giây", "k": "ca", "l": "lờ",
    "m": "mờ", "n": "nờ", "o": "o", "ô": "ô", "ơ": "ơ", "p": "pê", "q": "qui",
    "r": "rờ", "s": "ét", "t": "tê", "u": "u", "ư": "ư", "v": "vê", "w": "đắp liu",
    "x": "ích", "y": "y", "z": "dét"
}

_common_email_domains = {
    "gmail.com": "__START_EN__gmail__END_EN__ chấm com",
    "yahoo.com": "__START_EN__yahoo__END_EN__ chấm com",
    "yahoo.com.vn": "__START_EN__yahoo__END_EN__ chấm com chấm __START_EN__v n__END_EN__",
    "outlook.com": "__START_EN__outlook__END_EN__ chấm com",
    "hotmail.com": "__START_EN__hotmail__END_EN__ chấm com",
    "icloud.com": "__START_EN__icloud__END_EN__ chấm com",
    "fpt.vn": "__START_EN__f p t__END_EN__ chấm __START_EN__v n__END_EN__",
    "fpt.com.vn": "__START_EN__f p t__END_EN__ chấm com chấm __START_EN__v n__END_EN__",
}

_measurement_key_vi = {
    "km": "ki lô mét", "dm": "đê xi mét", "cm": "xen ti mét", "mm": "mi li mét",
    "nm": "na nô mét", "µm": "mic rô mét", "μm": "mic rô mét", "m": "mét",
    "kg": "ki lô gam", "g": "gam", "mg": "mi li gam",
    "km2": "ki lô mét vuông", "m2": "mét vuông", "cm2": "xen ti mét vuông", "mm2": "mi li mét vuông",
    "ha": "héc ta",
    "km3": "ki lô mét khối", "m3": "mét khối", "cm3": "xen ti mét khối", "mm3": "mi li mét khối",
    "l": "lít", "dl": "đê xi lít", "ml": "mi li lít", "hl": "héc tô lít",
    "kw": "ki lô oát", "mw": "mê ga oát", "gw": "gi ga oát",
    "kwh": "ki lô oát giờ", "mwh": "mê ga oát giờ", "wh": "oát giờ",
    "hz": "héc", "khz": "ki lô héc", "mhz": "mê ga héc", "ghz": "gi ga héc",
    "pa": "__start_en__pascal__end_en__", "kpa": "__start_en__kilopascal__end_en__", "mpa": "__start_en__megapascal__end_en__",
    "bar": "__start_en__bar__end_en__", "mbar": "__start_en__millibar__end_en__", "atm": "__start_en__atmosphere__end_en__", "psi": "__start_en__p s i__end_en__",
    "j": "__start_en__joule__end_en__", "kj": "__start_en__kilojoule__end_en__",
    "cal": "__start_en__calorie__end_en__", "kcal": "__start_en__kilocalorie__end_en__",
    "h": "giờ", "p": "phút", "s": "giây",
    "sqm": "mét vuông", "cum": "mét khối",
    "gb": "__start_en__gigabyte__end_en__", "mb": "__start_en__megabyte__end_en__", "kb": "__start_en__kilobyte__end_en__", "tb": "__start_en__terabyte__end_en__",
    "db": "__start_en__decibel__end_en__", "oz": "__start_en__ounce__end_en__", "lb": "__start_en__pound__end_en__", "lbs": "__start_en__pounds__end_en__",
    "ft": "__start_en__feet__end_en__", "in": "__start_en__inch__end_en__", "dpi": "__start_en__d p i__end_en__", "pH": "pê hát",
    "gbps": "__start_en__gigabits per second__end_en__", "mbps": "__start_en__megabits per second__end_en__", "kbps": "__start_en__kilobits per second__end_en__",
    "gallon": "__start_en__gallon__end_en__"
}

_currency_key = {
    "usd": "__start_en__u s d__end_en__",
    "vnd": "đồng", "đ": "đồng", "€": "__start_en__euro__end_en__", "euro": "__start_en__euro__end_en__", "eur": "__start_en__euro__end_en__",
    "¥": "yên", "yên": "yên", "jpy": "yên", "%": "phần trăm"
}

_letter_key_vi = _vi_letter_names

_acronyms_exceptions_vi = {
    "CĐV": "cổ động viên", "HĐND": "hội đồng nhân dân", "HĐQT": "hội đồng quản trị", "TAND": "toàn án nhân dân",
    "BHXH": "bảo hiểm xã hội", "BHTN": "bảo hiểm thất nghiệp", "TP.HCM": "thành phố hồ chí minh",
    "VN": "việt nam", "UBND": "uỷ ban nhân dân", "TP": "thành phố", "HCM": "hồ chí minh",
    "HN": "hà nội", "BTC": "ban tổ chức", "CLB": "câu lạc bộ", "HTX": "hợp tác xã",
    "NXB": "nhà xuất bản", "TW": "trung ương", "CSGT": "cảnh sát giao thông", "LHQ": "liên hợp quốc",
    "THCS": "trung học cơ sở", "THPT": "trung học phổ thông", "ĐH": "đại học", "HLV": "huấn luyện viên",
    "GS": "giáo sư", "TS": "tiến sĩ", "TNHH": "trách nhiệm hữu hạn", "VĐV": "vận động viên",
    "TPHCM": "thành phố hồ chí minh", "PGS": "phó giáo sư", "SP500": "ét pê năm trăm"
}

# Compiled Regular Expressions
RE_ROMAN_NUMBER = re.compile(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b")
RE_LETTER = re.compile(r"(chữ|chữ cái|kí tự|ký tự)\s+(['\"]?)([a-z])(['\"]?)\b", re.IGNORECASE)
RE_STANDALONE_LETTER = re.compile(r'(?<![\'’])\b([a-zA-Z])\b(\.?)')
RE_URL = re.compile(r'\b(?:https?://|www\.)[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b')
RE_SLASH_NUMBER = re.compile(r'\b(\d+)/(\d+)\b')
RE_EMAIL = re.compile(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b')
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
RE_CLEAN_OTHERS = re.compile(r'[^\w\sàáảãạăắằẳẵặâấầẩẫậèéẻẽẹêếềểễệìíỉĩịòóỏõọôốồổỗộơớờởỡợùúủũụưứừửữỳýỷỹỵđ.,!?_\'’]')
RE_CLEAN_QUOTES = re.compile(r'["“”"]')
RE_PRIME = re.compile(r"(\b[a-zA-Z0-9])['’](?!\w)")

_DOMAIN_SUFFIXES_RE = re.compile(r'\.(com|vn|net|org|edu|gov|io|biz|info)\b', re.IGNORECASE)
_DOMAIN_SUFFIX_MAP = {
    "com": "com",
    "vn": "__START_EN__v n__END_EN__",
    "net": "nét",
    "org": "o rờ gờ",
    "edu": "__START_EN__edu__END_EN__",
    "gov": "gờ o vê",
    "io": "__START_EN__i o__END_EN__",
    "biz": "biz",
    "info": "info",
}

# Reusable patterns for measurement/currency
_MAGNITUDE_P = r"\s*(tỷ|triệu|nghìn|ngàn)?\s*"
_NUMERIC_P = r"(\d+(?:[.,]\d+)*)"

# Pre-compiled regex for compound units
RE_COMPOUND_UNIT = re.compile(rf"\b{_NUMERIC_P}?\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b", re.IGNORECASE)

# Pre-compiled currency patterns
_CURRENCY_SYMBOL_MAP = {
    "$": "__start_en__u s d__end_en__",
    "€": "__start_en__euro__end_en__",
    "¥": "yên",
    "£": "__start_en__pound__end_en__",
    "₩": "won",
}
_CURRENCY_SYMBOLS_RE = "[$€¥£₩]"
RE_CURRENCY_PREFIX_SYMBOL = re.compile(rf"({_CURRENCY_SYMBOLS_RE})\s*{_NUMERIC_P}{_MAGNITUDE_P}", re.IGNORECASE)
RE_CURRENCY_SUFFIX_SYMBOL = re.compile(rf"{_NUMERIC_P}{_MAGNITUDE_P}({_CURRENCY_SYMBOLS_RE})", re.IGNORECASE)
RE_PERCENTAGE = re.compile(rf"{_NUMERIC_P}\s*%", re.IGNORECASE)

# Pre-compile measurement and currency unit patterns
_MEASUREMENT_PATTERNS = []
for unit, full in sorted(_measurement_key_vi.items(), key=lambda x: len(x[0]), reverse=True):
    pattern = re.compile(rf"\b{_NUMERIC_P}{_MAGNITUDE_P}{unit}\b", re.IGNORECASE)

    standalone_pattern = None
    safe_standalone = [
        "km", "cm", "mm", "kg", "mg",
        "m2", "km2", "usd", "vnd",
        "mhz", "khz", "ghz", "hz", "ph"
    ]
    if unit.lower() in safe_standalone:
        standalone_pattern = re.compile(rf"(?<![\d.,])\b{unit}\b", re.IGNORECASE)

    _MEASUREMENT_PATTERNS.append((pattern, standalone_pattern, full))

_CURRENCY_PATTERNS = []
for unit, full in _currency_key.items():
    if unit == "%": continue
    pattern = re.compile(rf"\b{_NUMERIC_P}{_MAGNITUDE_P}{unit}\b", re.IGNORECASE)
    _CURRENCY_PATTERNS.append((pattern, full))

# Pre-compile acronyms exceptions
_ACRONYMS_EXCEPTIONS_RE = [(re.compile(rf"\b{re.escape(k)}\b"), v) for k, v in _acronyms_exceptions_vi.items()]

_ROMAN_NUMERALS = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000}

_ABBRS = {"v.v": " vân vân", "v/v": " về việc", "đ/c": "địa chỉ"}

_SYMBOLS_MAP = {
    '&': ' và ', '+': ' cộng ', '=': ' bằng ', '#': ' thăng ',
    '>': ' lớn hơn ', '<': ' nhỏ hơn ',
    '≥': ' lớn hơn hoặc bằng ', '≤': ' nhỏ hơn hoặc bằng ',
    '±': ' cộng trừ ', '≈': ' xấp xỉ '
}

def _expand_scientific(num_str):
    num_lower = num_str.lower()
    e_idx = num_lower.find('e')
    base, exp = num_str[:e_idx], num_str[e_idx+1:]

    # Base normalization
    if base.count('.') == 1:
        parts = base.split('.')
        base_norm = f"{n2w(parts[0])} chấm {n2w_single(parts[1])}"
    elif base.count(',') == 1:
        parts = base.split(',')
        base_norm = f"{n2w(parts[0])} phẩy {n2w_single(parts[1])}"
    else:
        base_norm = n2w(base.replace(',', '').replace('.', ''))

    # Exponent normalization
    exp_val = exp.lstrip('+')
    exp_norm = f"trừ {n2w(exp_val[1:])}" if exp_val.startswith('-') else n2w(exp_val)
    return f"{base_norm} nhân mười mũ {exp_norm}"

def _expand_mixed_sep(num_str):
    if num_str.rfind('.') > num_str.rfind(','): # English style (1,299.5)
        parts = num_str.replace(',', '').split('.')
    else: # Vietnamese style (1.299,5)
        parts = num_str.replace('.', '').split(',')
    return f"{n2w(parts[0])} phẩy {n2w_single(parts[1])}"

def _expand_single_sep(num_str):
    if ',' in num_str:
        parts = num_str.split(',')
        if len(parts) > 2: return n2w(num_str.replace(',', ''))
        return f"{n2w(parts[0])} phẩy {n2w_single(parts[1])}"

    parts = num_str.split('.')
    if len(parts) > 2 or (len(parts) == 2 and len(parts[1]) == 3):
        return n2w(num_str.replace('.', ''))
    return f"{n2w(parts[0])} chấm {n2w_single(parts[1])}"

def _expand_number_with_sep(num_str):
    if not num_str: return ""
    if 'e' in num_str.lower(): return _expand_scientific(num_str)
    if ',' in num_str and '.' in num_str: return _expand_mixed_sep(num_str)
    if ',' in num_str or '.' in num_str: return _expand_single_sep(num_str)
    return n2w(num_str)

def fix_english_style_numbers(m):
    val = m.group(0)
    has_comma = ',' in val
    has_dot = '.' in val

    # definitely English thousands (multiple commas or dot after comma)
    if val.count(',') > 1 or (has_comma and has_dot and val.find(',') < val.find('.')):
        return val.replace(',', '').replace('.', ',') if has_dot else val.replace(',', '')

    # single comma, likely English thousands or decimal (handled by single sep logic later)
    # but 1,299.5 (English style) needs to be 1299,5
    if has_comma and has_dot:
        return val.replace(',', '').replace('.', ',')

    return val

def expand_power_of_ten(m):
    base = m.group(1)
    exp = m.group(2)
    # Use global normalize_others instead of local import
    base_norm = normalize_others(base).strip()
    exp_val = exp.replace('+', '')
    if exp_val.startswith('-'):
        exp_norm = "trừ " + n2w(exp_val[1:])
    else:
        exp_norm = n2w(exp_val)
    return f" {base_norm} nhân mười mũ {exp_norm} "

def expand_scientific_notation(text):
    # Match something like 3.2e5 or 6.626e-34
    # But be careful not to match words containing 'e'
    # Use anchored regex to reduce search space
    pattern = re.compile(r'\b(\d+(?:[.,]\d+)?e[+-]?\d+)\b', re.IGNORECASE)
    return pattern.sub(lambda m: _expand_number_with_sep(m.group(1)), text)

def expand_measurement(text):
    def _repl(m, full):
        num = m.group(1)
        mag = m.group(2) if m.group(2) else ""
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()
    
    for pattern, standalone_pattern, full in _MEASUREMENT_PATTERNS:
        # Case with number
        text = pattern.sub(lambda m, f=full: _repl(m, f), text)
        
        # Standalone units
        if standalone_pattern:
            text = standalone_pattern.sub(f" {full} ", text)
    return text

def expand_currency(text):
    def _repl_symbol(m, is_prefix=True):
        symbol = m.group(1 if is_prefix else 3)
        num = m.group(2 if is_prefix else 1)
        mag = m.group(3 if is_prefix else 2)
        mag = mag if mag else ""
        full = _CURRENCY_SYMBOL_MAP.get(symbol, "")
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()

    def _repl(m, full):
        num = m.group(1)
        mag = m.group(2) if m.group(2) else ""
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()
        
    text = RE_CURRENCY_PREFIX_SYMBOL.sub(lambda m: _repl_symbol(m, True), text)
    text = RE_CURRENCY_SUFFIX_SYMBOL.sub(lambda m: _repl_symbol(m, False), text)
    text = RE_PERCENTAGE.sub(lambda m: f"{_expand_number_with_sep(m.group(1))} phần trăm", text)
    
    for pattern, full in _CURRENCY_PATTERNS:
        text = pattern.sub(lambda m, f=full: _repl(m, f), text)
    return text

def expand_compound_units(text):
    def _repl_compound(m):
        num_str = m.group(1) if m.group(1) else ""
        num = _expand_number_with_sep(num_str)
        u1 = m.group(2).lower()
        u2 = m.group(3).lower()
        full1 = _measurement_key_vi.get(u1, _currency_key.get(u1, u1))
        full2 = _measurement_key_vi.get(u2, _currency_key.get(u2, u2))
        res = f" {full1} trên {full2} "
        if num:
            res = f"{num} {res}"
        return res

    text = RE_COMPOUND_UNIT.sub(_repl_compound, text)
    return text

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
        dot = m.group(2) if m.group(2) else ""
        if char in _letter_key_vi:
            # Drop dot for uppercase initials (e.g., "M." -> "mờ")
            if char_raw.isupper() and dot == '.':
                return f" {_letter_key_vi[char]} "
            return f" {_letter_key_vi[char]}{dot} "
        return m.group(0)
    
    return RE_STANDALONE_LETTER.sub(_repl_letter, text)

def normalize_urls(text):
    def _repl_url(m):
        url = m.group(0)
        # Split URL into segments but keep delimiters
        parts = re.split(r'([./:?&=])', url)
        res = []
        for p in parts:
            if not p: continue
            if p == '.': res.append('chấm')
            elif p in './:?&=-_': continue # Skip technical delimiters like : // ? & =
            elif p.isalnum() and p.isascii():
                # Check suffixes first
                if p.lower() in _DOMAIN_SUFFIX_MAP:
                    res.append(_DOMAIN_SUFFIX_MAP[p.lower()])
                else:
                    res.append(f"__START_EN__{p.lower()}__END_EN__")
            else:
                # Fallback to character-by-character for non-ASCII or mixed
                for char in p.lower():
                    if char.isalnum():
                        if char.isdigit():
                            res.append(n2w_single(char))
                        else:
                            res.append(_vi_letter_names.get(char, char))
                    else:
                        res.append(char)
        return " ".join(res)

    return RE_URL.sub(_repl_url, text)

def normalize_slashes(text):
    def _repl(m):
        n1 = m.group(1)
        n2 = m.group(2)
        # If it's likely an address (first number is large)
        if len(n1) > 2 or int(n1) > 31:
            return f"{n2w(n1)} xẹt {n2w(n2)}"
        return f"{n2w(n1)} trên {n2w(n2)}"
    return RE_SLASH_NUMBER.sub(_repl, text)

def normalize_emails(text):
    def _repl_email(m):
        email = m.group(0)
        parts = email.split('@')
        if len(parts) != 2: return email

        user_part, domain_part = parts

        # User part: spell out or EN
        if user_part.isalnum() and user_part.isascii():
            user_norm = [f"__START_EN__{user_part.lower()}__END_EN__"]
        else:
            user_unorm = []
            for char in user_part.lower():
                if char.isalnum():
                    if char.isdigit():
                        user_unorm.append(n2w_single(char))
                    else:
                        user_unorm.append(_vi_letter_names.get(char, char))
                elif char == '.': user_unorm.append('chấm')
                elif char == '_': user_unorm.append('gạch dưới')
                elif char == '-': user_unorm.append('gạch ngang')
                else: user_unorm.append(char)
            user_norm = user_unorm

        # Domain part
        domain_part_lower = domain_part.lower()
        if domain_part_lower in _common_email_domains:
            domain_norm = _common_email_domains[domain_part_lower]
        else:
            domain_parts = domain_part.split('.')
            norm_domain_parts = []
            for i, dp in enumerate(domain_parts):
                # If it's the last part and in our suffix map, use the map
                if i == len(domain_parts) - 1 and dp.lower() in _DOMAIN_SUFFIX_MAP:
                    norm_domain_parts.append(_DOMAIN_SUFFIX_MAP[dp.lower()])
                    continue
                
                if dp.isalnum() and dp.isascii():
                    norm_domain_parts.append(f"__START_EN__{dp.lower()}__END_EN__")
                    continue

                dp_norm = []
                for char in dp.lower():
                    if char.isalnum():
                        if char.isdigit():
                            dp_norm.append(n2w_single(char))
                        else:
                            dp_norm.append(_vi_letter_names.get(char, char))
                    else: dp_norm.append(char)
                norm_domain_parts.append(" ".join(dp_norm))
            domain_norm = " chấm ".join(norm_domain_parts)

        return " ".join(user_norm) + " a còng " + domain_norm

        return " ".join(user_norm) + " a còng " + domain_norm

    return RE_EMAIL.sub(_repl_email, text)

WORD_LIKE_ACRONYMS = {"UNESCO", "NASA", "NATO", "ASEAN", "OPEC", "SARS", "FIFA", "UNIC", "RAM", "VRAM", "COVID", "IELTS", "STEM", "SWAT", "SEAL", "WASP", "COBOL", "BASIC", "OLED", "COVAX", "BRICS", "APEC", "VUCA", "PERMA", "DINK", "MENA", "EPIC", "OASIS", "BASE", "DART", "IDEA", "CHAOS", "SMART", "FANG"}
# AT&T
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
                    return f"__START_EN__{word.lower()}__END_EN__"
                if any(c.isdigit() for c in word):
                    if word.upper() == "B2B":
                        return "__START_EN__b two b__END_EN__"
                    
                    res = []
                    for c in word.lower():
                        if c.isdigit():
                            res.append(n2w_single(c))
                        else:
                            res.append(_vi_letter_names.get(c, c))
                    return " ".join(res)

                spaced_word = " ".join(c.lower() for c in word if c.isalnum())
                if spaced_word:
                    return f"__START_EN__{spaced_word}__END_EN__"
                return word

            s = RE_ACRONYM.sub(_repl_acronym, s)

        processed.append(s + sep)
    return "".join(processed)

def expand_alphanumeric(text):
    def _repl(m):
        num = m.group(1)
        char = m.group(2).lower()
        if char in _letter_key_vi:
            pronunciation = _letter_key_vi[char]
            # Special case for roads (Quốc lộ 1D -> Quốc lộ 1 đê)
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
        if val.isdigit():
            # word for digit + phẩy
            return f"{n2w_single(val)} phẩy"
        else:
            # letter name + phẩy
            return f"{_letter_key_vi.get(val, val)} phẩy"
    return RE_PRIME.sub(_repl, text)

def expand_temperatures(text):
    text = RE_TEMP_C_NEG.sub(r'âm \1 độ xê', text)
    text = RE_TEMP_F_NEG.sub(r'âm \1 độ ép', text)
    text = RE_TEMP_C.sub(r'\1 độ xê', text)
    text = RE_TEMP_F.sub(r'\1 độ ép', text)
    text = RE_DEGREE.sub(' độ ', text)
    return text

def normalize_others(text):
    """
    Apply various normalization rules that don't fit into specific categories.
    This function is called by clean_vietnamese_text.
    """
    # 1. Expand acronym exceptions and basic patterns
    for pattern, v in _ACRONYMS_EXCEPTIONS_RE:
        text = pattern.sub(v, text)
    
    text = normalize_urls(text)
    text = normalize_emails(text)
    text = normalize_slashes(text)
    
    # Handle domain suffixes like .com, .vn (especially after acronyms)
    text = _DOMAIN_SUFFIXES_RE.sub(lambda m: " chấm " + _DOMAIN_SUFFIX_MAP.get(m.group(1).lower(), m.group(1).lower()), text)

    # 2. Expand Roman numerals and special letter patterns
    text = RE_ROMAN_NUMBER.sub(expand_roman, text)
    text = RE_LETTER.sub(expand_letter, text)
    text = expand_alphanumeric(text)
    
    # 3. Clean quotes and expand general symbols
    text = expand_prime(text) # Handle A' or 1' before cleaning general quotes
    text = RE_CLEAN_QUOTES.sub('', text)
    
    # Remove single quotes only if they are not part of a word (start/end of word)
    text = re.sub(r"(^|\s)['’]+|['’]+($|\s)", r"\1 \2", text)
    
    text = expand_symbols(text)

    # 4. Handle brackets and temperatures
    # Note: Measurement/Currency expansion is handled by clean_vietnamese_text caller
    text = RE_BRACKETS.sub(r', \1, ', text)
    text = RE_STRIP_BRACKETS.sub(' ', text)
    text = expand_temperatures(text)

    # 5. Normalize acronyms (spell out or tag with <en>)
    text = normalize_acronyms(text)

    # 6. Expand version numbers (e.g., 1.2.3 -> 1 chấm 2 chấm 3)
    text = RE_VERSION.sub(lambda m: ' chấm '.join(m.group(1).split('.')), text)

    # 7. Final punctuation normalization: convert : and ; to commas for better prosody
    text = re.sub(r'[:;]', ',', text)

    # 8. Final cleanup of any remaining unsupported characters
    text = RE_CLEAN_OTHERS.sub(' ', text)
    
    # Restore internal <en> tags (handle both cases for backward compatibility)
    text = text.replace('__START_EN__', '<en>').replace('__END_EN__', '</en>')
    text = text.replace('__start_en__', '<en>').replace('__end_en__', '</en>')
    
    return text
