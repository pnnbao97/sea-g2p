import re
from .num2vi import n2w, n2w_single

_vi_letter_names = {
    "a": "a", "b": "bê", "c": "xê", "d": "đê", "đ": "đê", "e": "e", "ê": "ê",
    "f": "ép", "g": "gờ", "h": "hát", "i": "i", "j": "giây", "k": "ca", "l": "lờ",
    "m": "mờ", "n": "nờ", "o": "o", "ô": "ô", "ơ": "ơ", "p": "pê", "q": "qui",
    "r": "rờ", "s": "ét", "t": "tê", "u": "u", "ư": "ư", "v": "vê", "w": "đắp liu",
    "x": "ích", "y": "y", "z": "dét"
}

_common_email_domains = {
    "gmail.com": "__start_en__gmail__end_en__ chấm com",
    "yahoo.com": "__start_en__yahoo__end_en__ chấm com",
    "yahoo.com.vn": "__start_en__yahoo__end_en__ chấm com chấm __start_en__v n__end_en__",
    "outlook.com": "__start_en__outlook__end_en__ chấm com",
    "hotmail.com": "__start_en__hotmail__end_en__ chấm com",
    "icloud.com": "__start_en__icloud__end_en__ chấm com",
    "fpt.vn": "__start_en__f p t__end_en__ chấm __start_en__v n__end_en__",
    "fpt.com.vn": "__start_en__f p t__end_en__ chấm com chấm __start_en__v n__end_en__",
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
    "gallon": "__start_en__gallon__end_en__", "mol": "mol", "ms": "mi li giây"
}

_currency_key = {
    "usd": "__start_en__u s d__end_en__",
    "vnd": "đồng", "đ": "đồng", "v n d": "đồng", "v n đ": "đồng", "€": "__start_en__euro__end_en__", "euro": "__start_en__euro__end_en__", "eur": "__start_en__euro__end_en__",
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
    "TPHCM": "thành phố hồ chí minh", "PGS": "phó giáo sư", "SP500": "ét pê năm trăm",
    "PGS.TS": "phó giáo sư tiến sĩ", "GS.TS": "giáo sư tiến sĩ", "ThS": "thạc sĩ", "BS": "bác sĩ",
    "UAE": "u a e", "CUDA": "cu đa"
}

_technical_terms = {
    "JSON": "__start_en__j son__end_en__",
    "VRAM": "__start_en__v ram__end_en__",
    "VN-Index": "__start_en__v n__end_en__ index",
    "MS DOS": "__start_en__m s dos__end_en__",
    "MS-DOS": "__start_en__m s dos__end_en__",
    "B2B": "__start_en__b two b__end_en__",
    "MI5": "__start_en__m i five__end_en__",
    "MI6": "__start_en__m i six__end_en__",
    "2FA": "__start_en__two f a__end_en__",
    "TX-0": "__start_en__t x zero__end_en__",
    "IPv6": "__start_en__i p v__end_en__ sáu",
    "IPv4": "__start_en__i p v__end_en__ bốn",
}

# Compiled Regular Expressions
RE_ROMAN_NUMBER = re.compile(r"\b(?=[IVXLCDM]{2,})M{0,4}(CM|CD|D?C{0,3})(XC|XL|L?X{0,3})(IX|IV|V?I{0,3})\b")
RE_LETTER = re.compile(r"(chữ|chữ cái|kí tự|ký tự)\s+(['\"]?)([a-z])(['\"]?)\b", re.IGNORECASE)
RE_STANDALONE_LETTER = re.compile(r'(?<![\'’])\b([a-zA-Z])\b(\.?)')
RE_TECHNICAL = re.compile(r'''
    \b(?:https?|ftp)://[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b(?:www\.)[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b[A-Za-z0-9.\-]+(?:\.com|\.vn|\.net|\.org|\.gov|\.io|\.biz|\.info)(?:/[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]*)?\b
    |
    (?<!\w)/[a-zA-Z0-9._\-/]{2,}\b
    |
    \b[a-zA-Z0-9._\-]+\.(?:txt|log|tar|gz|zip|sh|py|js|cpp|h|json|xml|yaml|yml|md|csv|pdf|docx|xlsx|exe|dll|so|config)\b
    |
    \b[a-zA-Z][a-zA-Z0-9]*(?:[._\-][a-zA-Z0-9]+){2,}\b
    |
    \b(?:[a-fA-F0-9]{1,4}:){3,7}[a-fA-F0-9]{1,4}\b
''', re.VERBOSE | re.IGNORECASE)
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
    "vn": "__start_en__v n__end_en__",
    "net": "nét",
    "org": "o rờ gờ",
    "edu": "__start_en__edu__end_en__",
    "gov": "gờ o vê",
    "io": "__start_en__i o__end_en__",
    "biz": "biz",
    "info": "info",
}

# Reusable patterns for measurement/currency
_MAGNITUDE_P = r"(?:\s*(tỷ|triệu|nghìn|ngàn))?"
_NUMERIC_P = r"(\d+(?:[.,]\d+)*)"

# Pre-compiled regex for compound units
# Using non-overlapping patterns to satisfy SonarCloud
RE_COMPOUND_UNIT = re.compile(rf'''
    \b
    (?:
        (\d+(?:[.,]\d+)*)\s*    # Numeric part (group 1)
        ([a-zμµ²³°]+)           # Unit 1 (group 2)
        /
        ([a-zμµ²³°0-9]+)        # Unit 2 (group 3)
        |
        ([a-zμµ²³°]+)           # Unit 1 (group 4, no numeric part)
        /
        ([a-zμµ²³°0-9]+)        # Unit 2 (group 5)
    )\b
''', re.VERBOSE | re.IGNORECASE)

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
_ALL_UNITS_MAP = {k.lower(): v for k, v in {**_measurement_key_vi, **_currency_key}.items()}
if "%" in _ALL_UNITS_MAP:
    del _ALL_UNITS_MAP["%"]

_UNIT_NAMES_SORTED = sorted(_ALL_UNITS_MAP.keys(), key=len, reverse=True)
RE_UNITS_WITH_NUM = re.compile(rf"(?<![\d.,]){_NUMERIC_P}{_MAGNITUDE_P}\s*(" + "|".join(re.escape(u) for u in _UNIT_NAMES_SORTED) + r")\b", re.IGNORECASE)

_SAFE_STANDALONE = ["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph", "đ"]
RE_STANDALONE_UNIT = re.compile(rf"(?<![\d.,])\b(" + "|".join(re.escape(u) for u in _SAFE_STANDALONE) + r")\b", re.IGNORECASE)

# Pre-compile acronyms exceptions (sorted by length descending for longest-match-first)
_combined_exceptions = {**_acronyms_exceptions_vi, **_technical_terms}
_EXCEPTIONS_SORTED_KEYS = sorted(_combined_exceptions.keys(), key=len, reverse=True)
RE_ACRONYMS_EXCEPTIONS = re.compile(rf"\b(" + "|".join(re.escape(k) for k in _EXCEPTIONS_SORTED_KEYS) + r")\b")

_ROMAN_NUMERALS = {"I": 1, "V": 5, "X": 10, "L": 50, "C": 100, "D": 500, "M": 1000}

_ABBRS = {"v.v": " vân vân", "v/v": " về việc", "đ/c": "địa chỉ"}

_SYMBOLS_MAP = {
    '&': ' và ', '+': ' cộng ', '=': ' bằng ', '#': ' thăng ',
    '>': ' lớn hơn ', '<': ' nhỏ hơn ',
    '≥': ' lớn hơn hoặc bằng ', '≤': ' nhỏ hơn hoặc bằng ',
    '±': ' cộng trừ ', '≈': ' xấp xỉ ',
    '/': ' trên ', '→': ' đến ', '÷': ' chia ',
    '*': ' nhân ', '×': ' nhân ', '^': ' mũ ', '~': ' khoảng '
}

def _expand_scientific(num_str):
    num_lower = num_str.lower()
    e_idx = num_lower.find('e')
    base, exp = num_str[:e_idx], num_str[e_idx+1:]

    # Base normalization
    if base.count('.') == 1:
        parts = base.split('.')
        dec_part = parts[1].rstrip('0')
        base_norm = f"{n2w(parts[0])} chấm {n2w_single(dec_part)}" if dec_part else n2w(parts[0])
    elif base.count(',') == 1:
        parts = base.split(',')
        dec_part = parts[1].rstrip('0')
        base_norm = f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}" if dec_part else n2w(parts[0])
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
    
    dec_part = parts[1].rstrip('0')
    if not dec_part:
        return n2w(parts[0])
    return f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}"

def _expand_single_sep(num_str):
    if ',' in num_str:
        parts = num_str.split(',')
        if len(parts) > 2 or (len(parts) == 2 and len(parts[1]) == 3):
            return n2w(num_str.replace(',', ''))
        
        # Strip trailing zeros for decimals
        dec_part = parts[1].rstrip('0')
        if not dec_part:
            return n2w(parts[0])
        return f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}"

    parts = num_str.split('.')
    if len(parts) > 2 or (len(parts) == 2 and len(parts[1]) == 3):
        return n2w(num_str.replace('.', ''))
    
    # Strip trailing zeros for decimals
    dec_part = parts[1].rstrip('0')
    if not dec_part:
        return n2w(parts[0])
    return f"{n2w(parts[0])} chấm {n2w_single(dec_part)}"

def _expand_number_with_sep(num_str):
    if not num_str: return ""
    if 'e' in num_str.lower(): return _expand_scientific(num_str)
    if ',' in num_str and '.' in num_str: return _expand_mixed_sep(num_str)
    if ',' in num_str or '.' in num_str: return _expand_single_sep(num_str)
    return n2w(num_str)


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

def expand_measurement_currency(text):
    """Combined expansion for measurement and currency units."""
    def _repl_symbol(m, is_prefix=True):
        symbol = m.group(1 if is_prefix else 3)
        num = m.group(2 if is_prefix else 1)
        mag = m.group(3 if is_prefix else 2)
        mag = mag if mag else ""
        full = _CURRENCY_SYMBOL_MAP.get(symbol, "")
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()

    def _repl_unit(m):
        num = m.group(1)
        mag = m.group(2) if m.group(2) else ""
        unit = m.group(3).lower()
        full = _ALL_UNITS_MAP.get(unit, unit)
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()

    # 1. Symbols
    text = RE_CURRENCY_PREFIX_SYMBOL.sub(lambda m: _repl_symbol(m, True), text)
    text = RE_CURRENCY_SUFFIX_SYMBOL.sub(lambda m: _repl_symbol(m, False), text)
    text = RE_PERCENTAGE.sub(lambda m: f"{_expand_number_with_sep(m.group(1))} phần trăm", text)
    
    # 2. Units with numbers
    text = RE_UNITS_WITH_NUM.sub(_repl_unit, text)

    # 3. Standalone units
    text = RE_STANDALONE_UNIT.sub(lambda m: f" {_ALL_UNITS_MAP.get(m.group(1).lower(), m.group(1))} ", text)

    return text

def expand_compound_units(text):
    def _repl_compound(m):
        g1, g2, g3, g4, g5 = m.groups()
        if g1 is not None: # Case with number: num unit1/unit2
            num = _expand_number_with_sep(g1)
            u1, u2 = g2.lower(), g3.lower()
        else: # Case without number: unit1/unit2
            num = ""
            u1, u2 = g4.lower(), g5.lower()

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

def expand_unit_powers(text):
    def _repl(m):
        base = m.group(1)
        power = m.group(2)
        power_norm = ""
        if power.startswith('-'):
            power_norm = "trừ " + n2w(power[1:])
        elif power.startswith('+'):
            power_norm = n2w(power[1:])
        else:
            power_norm = n2w(power)
        
        # If the base matches a known unit, use its expansion
        base_lower = base.lower()
        full_base = _measurement_key_vi.get(base_lower, _currency_key.get(base_lower, base))
        # If it's a known unit, it might have __start_en__ tags, let's keep them if present
        return f" {full_base} mũ {power_norm} "

    # Match word^[-]number
    return re.sub(r'\b([a-zA-Z]+)\^([-+]?\d+)\b', _repl, text)

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

def normalize_technical(text):
    import re as std_re
    def _repl_tech(m):
        orig = m.group(0)

        # Protocol handling
        rest = orig
        res = []
        if '://' in orig.lower():
            p_idx = orig.lower().find('://')
            protocol = orig[:p_idx]
            # Space out protocol if it looks like an acronym (short & uppercase)
            if protocol.isupper() and len(protocol) <= 4:
                p_norm = " ".join(protocol.lower())
            elif len(protocol) <= 3:
                p_norm = " ".join(protocol.lower())
            else:
                p_norm = protocol.lower()
            res.append(f"__start_en__{p_norm}__end_en__")
            rest = orig[p_idx+3:]
        elif orig.startswith('/'):
            res.append('gạch')
            rest = orig[1:]

        # Simple segments based on delimiters
        segments = std_re.split(r'([./:?&=/_ \-])', rest)
        idx = 0
        while idx < len(segments):
            s = segments[idx]
            if not s: # Empty string from split, e.g., between delimiter and next char
                idx += 1
                continue

            if s == '.':
                # Peek next segment for suffix map
                next_seg = ""
                for j in range(idx + 1, len(segments)):
                    if segments[j] and segments[j] not in './:?&=/_ -':
                        next_seg = segments[j]
                        break
                if next_seg.lower() in _DOMAIN_SUFFIX_MAP:
                    res.append('chấm')
                    res.append(_DOMAIN_SUFFIX_MAP[next_seg.lower()])
                    # Move idx forward to consume the segment we just peeked
                    idx += 1
                    while idx < len(segments) and (not segments[idx] or segments[idx].lower() != next_seg.lower()):
                        idx += 1
                    idx += 1 # move past the next_seg
                    continue
                res.append('chấm')
            elif s == '/':
                res.append('gạch')
            elif s == '-':
                res.append('gạch ngang')
            elif s == '_':
                res.append('gạch dưới')
            elif s == ':':
                res.append('hai chấm')
            elif s == '?':
                res.append('hỏi')
            elif s == '&':
                res.append('và')
            elif s == '=':
                res.append('bằng')
            elif s.lower() in _DOMAIN_SUFFIX_MAP:
                res.append(_DOMAIN_SUFFIX_MAP[s.lower()])
            elif s.isalnum() and s.isascii():
                if s.isdigit():
                    # In technical contexts (filenames, IDs), read digits individually
                    res.append(" ".join(n2w_single(c) for c in s))
                else:
                    # Split into letters and digits
                    sub_tokens = std_re.findall(r'[a-zA-Z]+|\d+', s)
                    if len(sub_tokens) > 1:
                        for t in sub_tokens:
                            if t.isdigit():
                                res.append(" ".join(n2w_single(c) for c in t))
                            else:
                                val = t.lower()
                                if t.isupper() and len(t) <= 4:
                                    val = " ".join(val)
                                elif len(val) <= 2 and len(val) > 0:
                                    val = " ".join(val)
                                res.append(f"__start_en__{val}__end_en__")
                    else:
                        val = s.lower()
                        if s.isupper() and len(s) <= 4:
                            val = " ".join(val)
                        elif len(val) <= 2 and len(val) > 0: # e.g. 'io' -> 'i o'
                            val = " ".join(val)
                        res.append(f"__start_en__{val}__end_en__")
            else:
                for char in s.lower():
                    if char.isalnum():
                        if char.isdigit(): res.append(n2w_single(char))
                        else: res.append(_vi_letter_names.get(char, char))
                    else: res.append(char)
            idx += 1

        return " ".join(res).replace("  ", " ").strip()
    return RE_TECHNICAL.sub(_repl_tech, text)

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

        def _norm_segment(s):
            if not s: return ""
            if s.isdigit(): return n2w(s)
            if s.isalnum() and s.isascii():
                sub_tokens = re.findall(r'[a-zA-Z]+|\d+', s)
                if len(sub_tokens) > 1:
                    res_parts = []
                    for t in sub_tokens:
                        if t.isdigit():
                            res_parts.append(n2w(t))
                        else:
                            res_parts.append(f"__start_en__{t.lower()}__end_en__")
                    return " ".join(res_parts)
                val = s.lower()
                # Use English tags for segments to avoid character-by-character spelling
                return f"__start_en__{val}__end_en__"
            
            # Character-by-character fallback for mixed/unrecognized segments
            res = []
            for char in s.lower():
                if char.isalnum():
                    if char.isdigit(): res.append(n2w_single(char))
                    else: res.append(_vi_letter_names.get(char, char))
                else: res.append(char)
            return " ".join(res)

        def _process_part(p, is_domain=False):
            # Split by delimiters but keep them: . _ - +
            segments = re.split(r'([._\-+])', p)
            res = []
            idx = 0
            while idx < len(segments):
                s = segments[idx]
                if not s:
                    idx += 1
                    continue
                if s == '.':
                    # Special check for domain suffixes
                    if is_domain:
                        next_seg = None
                        peek_idx = -1
                        for j in range(idx + 1, len(segments)):
                            if segments[j] and segments[j] not in '._-+':
                                next_seg = segments[j]
                                peek_idx = j
                                break
                        
                        if next_seg and next_seg.lower() in _DOMAIN_SUFFIX_MAP:
                            res.append('chấm')
                            res.append(_DOMAIN_SUFFIX_MAP[next_seg.lower()])
                            idx = peek_idx + 1
                            continue
                    res.append('chấm')
                elif s == '_': res.append('gạch dưới')
                elif s == '-': res.append('gạch ngang')
                elif s == '+': res.append('cộng')
                else:
                    res.append(_norm_segment(s))
                idx += 1
            return " ".join(res)

        user_norm = _process_part(user_part)
        
        domain_part_lower = domain_part.lower()
        if domain_part_lower in _common_email_domains:
            domain_norm = _common_email_domains[domain_part_lower]
        else:
            domain_norm = _process_part(domain_part, is_domain=True)

        return f"{user_norm} a còng {domain_norm}".replace("  ", " ").strip()

    return RE_EMAIL.sub(_repl_email, text)

WORD_LIKE_ACRONYMS = {"UNESCO", "NASA", "NATO", "ASEAN", "OPEC", "SARS", "FIFA", "UNIC", "RAM", "VRAM", "COVID", "IELTS", "STEM", "SWAT", "SEAL", "WASP", "COBOL", "BASIC", "OLED", "COVAX", "BRICS", "APEC", "VUCA", "PERMA", "DINK", "MENA", "EPIC", "OASIS", "BASE", "DART", "IDEA", "CHAOS", "SMART", "FANG", "BLEU", "REST"}
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
                    return f"__start_en__{word.lower()}__end_en__"
                
                if any(c.isdigit() for c in word):
                    res = []
                    for c in word.lower():
                        if c.isdigit():
                            res.append(n2w_single(c))
                        else:
                            res.append(_vi_letter_names.get(c, c))
                    return " ".join(res)

                spaced_word = " ".join(c.lower() for c in word if c.isalnum())
                if spaced_word:
                    return f"__start_en__{spaced_word}__end_en__"
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
    text = RE_ACRONYMS_EXCEPTIONS.sub(lambda m: _combined_exceptions[m.group(1)], text)
    
    text = normalize_slashes(text)
    
    # Handle domain suffixes like .com, .vn (especially after acronyms)
    text = _DOMAIN_SUFFIXES_RE.sub(lambda m: " chấm " + _DOMAIN_SUFFIX_MAP.get(m.group(1).lower(), m.group(1).lower()), text)

    # 2. Expand Roman numerals and special letter patterns
    text = RE_ROMAN_NUMBER.sub(expand_roman, text)
    text = RE_LETTER.sub(expand_letter, text)
    text = expand_alphanumeric(text)
    
    # 3. Clean quotes and expand general symbols
    text = expand_prime(text) # Handle A' or 1' before cleaning general quotes
    text = expand_unit_powers(text)
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

    # 6. Expand version numbers and IPs (e.g., 1.2.3 -> một chấm hai chấm ba)
    def _expand_version(m):
        res = []
        for s in m.group(1).split('.'):
            res.append(" ".join(n2w_single(c) for c in s))
        return ' chấm '.join(res)
    text = RE_VERSION.sub(_expand_version, text)

    # 7. Final punctuation normalization: convert : and ; to commas for better prosody
    text = re.sub(r'[:;]', ',', text)

    # 8. Final cleanup of any remaining unsupported characters
    text = RE_CLEAN_OTHERS.sub(' ', text)
    
    return text
