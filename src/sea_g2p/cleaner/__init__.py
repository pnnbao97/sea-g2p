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
    if not dec_part:
        res = int_part
    else:
        res = f"{int_part} phẩy {n2w_single(dec_part)}"
    
    if m.group(3):
        res += " phần trăm"
    return f" {res} "

def _strip_dot_sep(m):
    return m.group(0).replace('.', '')

def _normalize_pre_number(text):
    # Handle explicit powers of ten: 1.5×10^-3 or 1.5x10^3 or 1.5*10^3
    # Anchored regex to reduce search space and avoid ReDoS
    text = re.sub(r'\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b', expand_power_of_ten, text, flags=re.IGNORECASE)
    text = re.sub(r'\b10\^([-+]?\d+)\b', lambda m: f"mười mũ {('trừ ' + n2w(m.group(1)[1:])) if m.group(1).startswith('-') else n2w(m.group(1).replace('+', ''))}", text)
    
    text = expand_abbreviations(text)
    text = normalize_date(text)
    text = normalize_time(text)
    
    def _range_sub(m):
        n1 = re.sub(r'[,.]', '', m.group(1))
        n2 = re.sub(r'[,.]', '', m.group(2))
        # Only treat as a numeric range if digit counts are similar (within 1)
        if abs(len(n1) - len(n2)) <= 1:
            return f'{m.group(1)} đến {m.group(2)}'
        return m.group(0)
    text = re.sub(r'(\d+(?:[,.]\d+)?)\s*[–\-—]\s*(\d+(?:[,.]\d+)?)', _range_sub, text)
    text = re.sub(r'(?<=\s)[–\-—](?=\s)', ',', text)
    text = re.sub(r'\s*(?:->|=>)\s*', ' sang ', text)
    return text

def _normalize_units_currency(text):
    text = expand_scientific_notation(text)
    text = expand_compound_units(text)
    text = expand_measurement(text)
    text = expand_currency(text)

    text = re.sub(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b', fix_english_style_numbers, text)

    def _expand_multi_comma(m):
        res = []
        for s in m.group(1).split(','):
            res.append(' '.join(n2w_single(c) for c in s))
        return ' phẩy '.join(res)
    text = re.sub(r'\b(\d+(?:,\d+){2,})\b', _expand_multi_comma, text)

    text = re.sub(r'(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?', _expand_float, text)
    text = re.sub(r'(?<![\d.])\d+(?:\.\d{3})+(?![\d.])', _strip_dot_sep, text)
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

    # Simple regex to protect existing tags, avoiding potential ReDoS in nested patterns
    text = re.sub(r'___PROTECTED_EN_TAG_\d+___', protect, text)

    # Normalize URLs and Emails early and protect them
    def protect_url_email(match):
        orig = match.group(0)
        # First expand it
        if '@' in orig:
            normed = normalize_emails(orig)
        else:
            normed = normalize_technical(orig)
        # Then mask the result
        return protect(re.Match if False else type('Match', (), {'group': lambda self, n: normed})())

    # Order matters: Emails first as they are more specific than generic URLs
    text = RE_EMAIL.sub(protect_url_email, text)
    text = RE_TECHNICAL.sub(protect_url_email, text)

    # Some tokens like VND might be misinterpreted as acronyms or currency
    # Currency expansion usually happens in _normalize_units_currency

    text = _normalize_pre_number(text)
    text = _normalize_units_currency(text)
    text = _normalize_post_number(text)

    # Protect internally generated tags before standalone letter expansion
    text = re.sub(r'(__start_en__.*?__end_en__|<en>.*?</en>)', protect, text, flags=re.IGNORECASE)
    text = expand_standalone_letters(text)

    for mask, original in mask_map.items():
        text = text.replace(mask, original)
        text = text.replace(mask.lower(), original)

    # Final conversion of any remaining __start_en__ tags
    text = text.replace('__start_en__', '<en>').replace('__end_en__', '</en>')

    text = _cleanup_whitespace(text)
    return text.lower()
