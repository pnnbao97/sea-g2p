import re
from .num2vi import n2w, n2w_single

# --- Compiled Regular Expressions ---

# Refined number patterns to handle various separators (dots, commas, spaces).
# For dots and spaces, we require exactly 3 digits to avoid matching versions/IPs.
# Patterns are structured to avoid ReDoS (no nested quantifiers with high repetition).
_number_pattern = (
    r"("
    r"\d+(?:\.\d{3})*(?:,\d+)?" # Vietnamese: 1.234.567 or 1.234,5
    r"|"
    r"\d+(?: \d{3})*(?:,\d+)?"   # With spaces: 1 234 567
    r"|"
    r"\d+(?:,\d+)*"              # Cardinal or single comma decimal: 1234 or 1,234
    r")"
)

RE_NUMBER = re.compile(r"(\D)(-{1})?" + _number_pattern)
RE_NUMBER_START = re.compile(r"^(-{1})?" + _number_pattern, re.MULTILINE)
RE_MULTIPLY = re.compile(r"(\d+)\s*[xX×]\s*(\d+)")
RE_ORDINAL = re.compile(r"(thứ|hạng)\s+(\d+)\b", re.IGNORECASE)
RE_PHONE = re.compile(r"((?:\+84|84|0|0084)[35789]\d{8})\b")
RE_DOT_SEP = re.compile(r"\d+(\.\d{3})+")

def _normalize_dot_sep(number: str) -> str:
    if RE_DOT_SEP.fullmatch(number):
        return number.replace(".", "")
    return number

def _num_to_words(number: str, negative: bool = False) -> str:
    number = number.replace(" ", "")
    number = _normalize_dot_sep(number)

    if "," in number:
        parts = number.split(",")
        if len(parts) == 2:
            return f"{n2w(parts[0])} phẩy {n2w_single(parts[1])}"
        return n2w(number.replace(",", ""))

    prefix = "âm " if negative else ""
    return prefix + n2w(number)

def _expand_number(match):
    prefix, negative_symbol, number = match.groups()
    negative = (negative_symbol == "-")
    word = _num_to_words(number, negative)
    prefix_str = prefix if prefix else ""
    return f"{prefix_str} {word} "

def _expand_number_start(match):
    negative_symbol, number = match.groups()
    negative = (negative_symbol == "-")
    return f"{_num_to_words(number, negative)} "

def _expand_phone(match):
    return n2w_single(match.group(0).strip())

def _expand_ordinal(match):
    prefix, number = match.groups()
    if number == "1": return f"{prefix} nhất"
    if number == "4": return f"{prefix} tư"
    return f"{prefix} {n2w(number)}"

def _expand_multiply_number(match):
    n1, n2 = match.groups()
    return f"{n2w(n1)} nhân {n2w(n2)}"

def normalize_number_vi(text):
    text = RE_ORDINAL.sub(_expand_ordinal, text)
    text = RE_MULTIPLY.sub(_expand_multiply_number, text)
    text = RE_PHONE.sub(_expand_phone, text)
    text = RE_NUMBER_START.sub(_expand_number_start, text)
    text = RE_NUMBER.sub(_expand_number, text)
    return text
