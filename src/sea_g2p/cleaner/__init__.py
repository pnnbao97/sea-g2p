import re
import string
from .num2vi import n2w, n2w_single

from .numerical import normalize_number_vi
from .datestime import normalize_date, normalize_time
from .text_norm import (
    normalize_others, expand_measurement, expand_currency,
    expand_compound_units, expand_abbreviations, expand_standalone_letters,
    expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten,
    normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL
)

def _expand_float(m):
    int_part = n2w(m.group(1).replace('.', ''))
    dec_part = m.group(2).rstrip('0')
    res = int_part if not dec_part else f"{int_part} phẩy {n2w_single(dec_part)}"
    if m.group(3): res += " phần trăm"
    return f" {res} "

def _strip_dot_sep(m):
    return m.group(0).replace('.', '')

def _normalize_pre_number(text):
    def _ten_power_repl(m):
        if m.group(1):
            return expand_power_of_ten(m)
        exp = m.group(2)
        exp_norm = ("trừ " + n2w(exp[1:])) if exp.startswith('-') else n2w(exp.replace('+', ''))
        return f"mười mũ {exp_norm}"

    text = re.sub(r'\b(?:(\d+(?:[.,]\d+)?)\s*[x*×]\s*)?10\^([-+]?\d+)\b', _ten_power_repl, text, flags=re.IGNORECASE)
    text = expand_abbreviations(text)
    text = normalize_date(text)
    text = normalize_time(text)
    
    def _misc_pre_repl(m):
        if m.group(1):
            n1, n2 = re.sub(r'[,.]', '', m.group(1)), re.sub(r'[,.]', '', m.group(2))
            return f'{m.group(1)} đến {m.group(2)}' if abs(len(n1) - len(n2)) <= 1 else m.group(0)
        return ' sang ' if ('->' in m.group(0) or '=>' in m.group(0)) else ','

    text = re.sub(r'(\d+(?:[.,]\d+)?)\s*[–\-—]\s*(\d+(?:[.,]\d+)?)|(?<=\s)[–\-—](?=\s)|\s*(?:->|=>)\s*', _misc_pre_repl, text)
    return text

def _normalize_units_currency(text):
    text = expand_scientific_notation(text)
    text = expand_compound_units(text)
    text = expand_measurement(text)
    text = expand_currency(text)

    # Use more precise non-backtracking structures for thousands separators
    text = re.sub(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b', fix_english_style_numbers, text)

    def _expand_multi_comma(m):
        return ' phẩy '.join(n2w_single(s) for s in m.group(1).split(','))
    text = re.sub(r'\b(\d+(?:,\d+){2,})\b', _expand_multi_comma, text)

    def _float_dot_repl(m):
        if m.group(2):
            return _expand_float(m)
        return _strip_dot_sep(m)

    # Refactored for SonarCloud: avoid nested quantifiers like (\d+(?:\.\d{3})*)*
    text = re.sub(r'(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?|(?<![\d.])\d+(?:\.\d{3})+(?![\d.])', _float_dot_repl, text)
    return text

def _normalize_post_number(text):
    text = normalize_others(text)
    text = normalize_number_vi(text)
    return text

def _cleanup_whitespace(text):
    text = re.sub(r'[ \t\xA0]+', ' ', text)
    text = re.sub(r',\s*,', ',', text)
    text = re.sub(r',\s*([.!?;])', r'\1', text)
    text = re.sub(r'\s+([,.!?;:])', r'\1', text)
    return text.strip().strip(',')

def clean_vietnamese_text(text):
    mask_map = {}
    def protect(match):
        idx = len(mask_map)
        mask = f"mask{str(idx).zfill(4)}mask".translate(str.maketrans('0123456789', string.ascii_lowercase[:10]))
        mask_map[mask] = match.group(0)
        return mask

    text = re.sub(r'___PROTECTED_EN_TAG_\d+___', protect, text)

    def protect_url_email(match):
        orig = match.group(0)
        normed = normalize_emails(orig) if '@' in orig else normalize_technical(orig)
        return protect(type('Match', (), {'group': lambda self, n: normed})())

    text = RE_EMAIL.sub(protect_url_email, text)
    text = RE_TECHNICAL.sub(protect_url_email, text)

    text = _normalize_pre_number(text)
    text = _normalize_units_currency(text)
    text = _normalize_post_number(text)

    text = re.sub(r'(__start_en__.*?__end_en__|<en>.*?</en>)', protect, text, flags=re.IGNORECASE)
    text = expand_standalone_letters(text)

    for mask, original in mask_map.items():
        text = text.replace(mask, original)
        text = text.replace(mask.lower(), original)

    text = text.replace('__start_en__', '<en>').replace('__end_en__', '</en>')
    return _cleanup_whitespace(text).lower()
