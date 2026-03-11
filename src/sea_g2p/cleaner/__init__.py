import re
import string
from .num2vi import n2w, n2w_single

from .numerical import normalize_number_vi
from .datestime import normalize_date, normalize_time
from .text_norm import (
    normalize_others, expand_measurement, expand_currency,
    expand_compound_units, expand_abbreviations, expand_standalone_letters,
    expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten
)

def _expand_float(m):
    int_part = n2w(m.group(1).replace('.', ''))
    dec_part = n2w_single(m.group(2))
    res = f"{int_part} phẩy {dec_part}"
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
    
    text = re.sub(r'(\d+(?:,\d+)?)\s*[–\-—~]\s*(\d+(?:,\d+)?)', r'\1 đến \2', text)
    text = re.sub(r'(?<=\s)[–\-—](?=\s)', ',', text)
    text = re.sub(r'\s*(?:->|=>)\s*', ' sang ', text)
    return text

def _normalize_units_currency(text):
    text = expand_compound_units(text)
    text = expand_measurement(text)
    text = expand_scientific_notation(text)
    text = expand_currency(text)

    text = re.sub(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b', fix_english_style_numbers, text)
    text = re.sub(r'(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?', _expand_float, text)
    text = re.sub(r'\b\d+(?:\.\d{3})+\b', _strip_dot_sep, text)
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
    text = _normalize_pre_number(text)
    text = _normalize_units_currency(text)
    text = _normalize_post_number(text)

    # Protect internally generated <en> tags before standalone letter expansion
    text = re.sub(r'<en>.*?</en>', protect, text, flags=re.IGNORECASE)
    text = expand_standalone_letters(text)
    text = _cleanup_whitespace(text)

    for mask, original in mask_map.items():
        text = text.replace(mask, original)
        text = text.replace(mask.lower(), original)
        
    return text
