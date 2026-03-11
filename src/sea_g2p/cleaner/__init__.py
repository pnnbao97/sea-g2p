import re
import string
from .num2vi import n2w, n2w_single

from .numerical import normalize_number_vi
from .datestime import normalize_date, normalize_time
from .text_norm import (
    normalize_others, expand_measurement, expand_currency,
    expand_compound_units, expand_abbreviations, expand_standalone_letters,
    expand_scientific_notation
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
    def _expand_power_of_ten(m):
        base = m.group(1)
        exp = m.group(2)
        base_norm = normalize_others(base).strip()
        if exp.startswith('-'):
            exp_norm = "trừ " + n2w(exp[1:])
        elif exp.startswith('+'):
            exp_norm = n2w(exp[1:])
        else:
            exp_norm = n2w(exp)
        return f" {base_norm} nhân mười mũ {exp_norm} "

    text = re.sub(r'(\d+(?:[.,]\d+)?)\s*(?:x|\*|×)\s*10\^([-+]?\d+)', _expand_power_of_ten, text, flags=re.IGNORECASE)
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
    def _fix_english_style(m):
        val = m.group(0)
        # If it has more than 1 comma, or it has a dot after a comma, it's definitely English thousands
        if val.count(',') > 1 or (',' in val and '.' in val and val.find(',') < val.find('.')):
            if '.' in val:
                parts = val.split('.')
                return parts[0].replace(',', '') + ',' + parts[1]
            else:
                return val.replace(',', '')

        # If it's something like 1,299 (single comma, 3 digits)
        if ',' in val and val.count(',') == 1 and '.' not in val:
            parts = val.split(',')
            if len(parts[1]) == 3:
                # User says 1,299 should be "một phẩy hai chín chín"
                # So we keep it as 1,299 and let _expand_float handle it?
                # Actually _expand_float expects (\d+(?:\.\d{3})*),(\d+)
                # 1,299 does match it if we consider it as 1 , 299
                return val

        # 1,299.5
        if ',' in val and '.' in val:
             parts = val.split('.')
             # if it's 1,299.5 -> we want 1299,5
             return parts[0].replace(',', '') + ',' + parts[1]

        return val

    text = re.sub(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b', _fix_english_style, text)

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
