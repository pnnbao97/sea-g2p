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

def clean_vietnamese_text(text):
    mask_map = {}

    # Handle explicit powers of ten: 1.5×10^-3 or 1.5x10^3 or 1.5*10^3
    # Mitigate ReDoS by avoiding nested quantifiers and ensuring efficient matching
    text = re.sub(r'(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)', expand_power_of_ten, text, flags=re.IGNORECASE)
    text = re.sub(r'10\^([-+]?\d+)', lambda m: f"mười mũ {('trừ ' + n2w(m.group(1)[1:])) if m.group(1).startswith('-') else n2w(m.group(1).replace('+', ''))}", text)
    
    def protect(match):
        idx = len(mask_map)
        mask = "mask" + "".join([string.ascii_lowercase[int(d)] for d in str(idx).zfill(4)]) + "mask"
        mask_map[mask] = match.group(0)
        return mask
    
    text = re.sub(r'___PROTECTED_EN_TAG_\d+___', protect, text)
    
    # Handle common abbreviations early to avoid unit conflicts
    text = expand_abbreviations(text)
    
    text = normalize_date(text)
    text = normalize_time(text)

    text = re.sub(r'(\d+(?:,\d+)?)\s*[–\-—~]\s*(\d+(?:,\d+)?)', r'\1 đến \2', text)
    
    # 3. Replace standalone hyphens with commas (for better TTS prosody/pausing)
    text = re.sub(r'(?<=\s)[–\-—](?=\s)', ',', text)
    
    text = re.sub(r'\s*(?:->|=>)\s*', ' sang ', text)

    # Expand measurements and currencies BEFORE general floats
    text = expand_compound_units(text)
    text = expand_measurement(text)
    text = expand_scientific_notation(text)
    text = expand_currency(text)

    # Convert English thousand separators to Vietnamese style or remove them for general numbers
    # only if they look like English thousands (1,299,495.5 or 1,299,495)
    text = re.sub(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b', fix_english_style_numbers, text)

    text = re.sub(r'(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?', _expand_float, text)
    text = re.sub(r'\b\d+(?:\.\d{3})+\b', _strip_dot_sep, text)
    
    text = normalize_others(text)
    text = normalize_number_vi(text)
    
    # Protect internally generated <en> tags from expand_standalone_letters
    text = re.sub(r'<en>.*?</en>', protect, text, flags=re.IGNORECASE)
    
    # Finally expand standalone letters to catch initials like "M."
    text = expand_standalone_letters(text)

    # Collapse redundant punctuation and whitespace
    # 1. Collapse multiple spaces BUT preserve newlines
    text = re.sub(r'[ \t\xA0]+', ' ', text)
    # 2. Collapse consecutive commas and handle comma-punctuation pairs
    text = re.sub(r',\s*,', ',', text)
    text = re.sub(r',\s*([.!?;])', r'\1', text)
    # 3. Handle redundant spaces before punctuation
    text = re.sub(r'\s+([,.!?;:])', r'\1', text)
    # 4. Remove leading/trailing commas if they end up at the start/end of sentence parts
    text = text.strip().strip(',')

    for mask, original in mask_map.items():
        text = text.replace(mask, original)
        text = text.replace(mask.lower(), original)
        
    return text
