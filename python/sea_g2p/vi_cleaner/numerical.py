import re
from .num2vi import n2w, n2w_single, n2w_decimal
from .symbols import vietnamese_set

# Compiled Regular Expressions
# Mitigation of ReDoS by using lookbehind and ordered non-overlapping patterns.
RE_NUMBER = re.compile(
    r"(?<!\d)(?P<neg>[-–—])?"
    r"(\d+(?:,\d+|(?:\.\d{3})+(?!\d)|\.\d+|(?:\s\d{3})+(?!\d))?)"
    r"(?!\d)"
)
RE_MULTIPLY = re.compile(r"(\d+)(x|\sx\s)(\d+)")
RE_ORDINAL = re.compile(r"(thứ|hạng)(\s+)(\d+)\b", re.IGNORECASE)
RE_PHONE = re.compile(r"((\+84|84|0|0084)(3|5|7|8|9)[0-9]{8})")
RE_DOT_SEP = re.compile(r"\d+(\.\d{3})+")

def _normalize_dot_sep(number: str) -> str:
    if RE_DOT_SEP.fullmatch(number):
        return number.replace(".", "")
    return number

def _num_to_words(number: str, negative: bool = False) -> str:
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
    number_str = match.group(2)

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
    prefix, space, number = match.groups()
    if number == "1": return prefix + space + "nhất"
    if number == "4": return prefix + space + "tư"
    return prefix + space + n2w(number)

def _expand_multiply_number(match):
    n1, _, n2 = match.groups()
    return n2w(n1) + " nhân " + n2w(n2)

def normalize_number_vi(text):
    text = RE_ORDINAL.sub(_expand_ordinal, text)
    text = RE_MULTIPLY.sub(_expand_multiply_number, text)
    text = RE_PHONE.sub(_expand_phone, text)
    # Process numbers with a single pass handling negative signs via context
    text = RE_NUMBER.sub(_expand_number, text)
    return text
