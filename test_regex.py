import re
from sea_g2p.vi_cleaner.num2vi import n2w, n2w_single

_normal_number_re        = r"[\d]+"
_float_number_re         = r"[\d]+[,]{1}[\d]+"
_decimal_number_re       = r"[\d]+[.]{1}[\d]+"
_number_with_one_dot     = r"[\d]+[.]{1}[\d]{3}"
_number_with_two_dot     = r"[\d]+[.]{1}[\d]{3}[.]{1}[\d]{3}"
_number_with_three_dot   = r"[\d]+[.]{1}[\d]{3}[.]{1}[\d]{3}[.]{1}[\d]{3}"
_number_with_one_space   = r"[\d]+[\s]{1}[\d]{3}"
_number_with_two_space   = r"[\d]+[\s]{1}[\d]{3}[\s]{1}[\d]{3}"
_number_with_three_space = r"[\d]+[\s]{1}[\d]{3}[\s]{1}[\d]{3}[\s]{1}[\d]{3}"

_number_combined = (
    r"("
    + _float_number_re + "|"
    + _decimal_number_re + "|"
    + _number_with_three_dot + "|"
    + _number_with_two_dot + "|"
    + _number_with_one_dot + "|"
    + _number_with_three_space + "|"
    + _number_with_two_space + "|"
    + _number_with_one_space + "|"
    + _normal_number_re
    + r")"
)

RE_NUMBER = re.compile(r"(\D)(-{1})?" + _number_combined + r"(?!\d)")

def _num_to_words(number: str, negative: bool = False) -> str:
    # number = _normalize_dot_sep(number).replace(" ", "")
    if "," in number:
        parts = number.split(",")
        return (("âm " if negative else "") + n2w(parts[0]) + " phẩy " + n2w(parts[1])).strip()
    elif "." in number:
        parts = number.split(".")
        return (("âm " if negative else "") + n2w(parts[0]) + " chấm " + n2w_single(parts[1])).strip()
    elif negative:
        return "âm " + n2w(number)
    return n2w(number)

def _expand_number(match):
    prefix, negative_symbol, number = match.groups(0)
    print(f"Match: groups={match.groups(0)}")
    negative = (negative_symbol == "-")
    word = _num_to_words(number, negative)
    prefix_str = "" if prefix in (0, None) else prefix
    return prefix_str + " " + word + " "

text = ", -2.5"
print(f"Input: '{text}'")
out = RE_NUMBER.sub(_expand_number, text)
print(f"Output: '{out}'")
