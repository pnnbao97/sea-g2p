import re
from .num2vi import n2w, n2w_single, n2w_decimal

# Compiled Regular Expressions
# Mitigation of ReDoS by using lookbehind and simple patterns with repetition limits.
RE_NUMBER = re.compile(r"(?<!\d)(?P<neg>[-–—])?(?P<num>\d{1,20}(?:(?:[.,\s]\d{3}){1,15}|[.,]\d{1,10})?)(?!\d)")
RE_MULTIPLY = re.compile(r"(?P<n1>\d{1,20})\s*[xX×]\s*(?P<n2>\d{1,20})")
RE_ORDINAL = re.compile(r"(?P<prefix>thứ|hạng)\s+(?P<num>\d{1,20})\b", re.IGNORECASE)
RE_PHONE = re.compile(r"(?<!\d)(?:\+84|84|0|0084)[35789]\d{8}(?!\d)")
RE_DOT_SEP = re.compile(r"\d{1,3}(?:\.\d{3}){1,15}")

def _normalize_dot_sep(number: str) -> str:
    if RE_DOT_SEP.fullmatch(number):
        return number.replace(".", "")
    return number

def _num_to_words(number: str, negative: bool = False) -> str:
    # Handle space-separated clusters that aren't valid thousand-separated numbers
    if " " in number:
        parts = number.split()
        if not all(len(p) == 3 for p in parts[1:]):
            res = " ".join(n2w(p) for p in parts)
            return (("âm " if negative else "") + res).strip()

    # First check if it's a decimal with dot BEFORE stripping any dots
    if "." in number and not RE_DOT_SEP.fullmatch(number):
        parts = number.replace(" ", "").split(".")
        if len(parts) == 2:
            return (("âm " if negative else "") + n2w(parts[0]) + " chấm " + n2w_decimal(parts[1])).strip()

    number = _normalize_dot_sep(number).replace(" ", "")
    if "," in number:
        parts = number.split(",")
        return (("âm " if negative else "") + n2w(parts[0]) + " phẩy " + n2w_decimal(parts[1])).strip()
    elif negative:
        return ("âm " + n2w(number)).strip()
    return n2w(number)

def _expand_number(match):
    start = match.start()
    text = match.string
    prefix_char = text[start-1] if start > 0 else ""

    neg_symbol = match.group('neg')
    number_str = match.group('num')

    is_neg = False
    if neg_symbol:
        if not prefix_char or prefix_char.isspace() or prefix_char in "([;,.":
            is_neg = True

    word = _num_to_words(number_str, is_neg)
    if neg_symbol and not is_neg:
        word = neg_symbol + word

    return " " + word + " "

def _expand_phone(match):
    return n2w_single(match.group(0).strip())

def _expand_ordinal(match):
    prefix = match.group('prefix')
    number = match.group('num')
    if number == "1": return prefix + " nhất"
    if number == "4": return prefix + " tư"
    return prefix + " " + n2w(number)

def _expand_multiply_number(match):
    n1 = match.group('n1')
    n2 = match.group('n2')
    return n2w(n1) + " nhân " + n2w(n2)

def normalize_number_vi(text):
    text = RE_ORDINAL.sub(_expand_ordinal, text)
    text = RE_MULTIPLY.sub(_expand_multiply_number, text)
    text = RE_PHONE.sub(_expand_phone, text)
    # Process numbers with a single pass handling negative signs via context
    text = RE_NUMBER.sub(_expand_number, text)
    return text
