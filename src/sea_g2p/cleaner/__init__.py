import re
import string
from .num2vi import n2w, n2w_single

from .numerical import normalize_number_vi
from .datestime import normalize_date, normalize_time
from .text_norm import (
    normalize_others, expand_measurement_currency,
    expand_compound_units, expand_abbreviations, expand_standalone_letters,
    expand_scientific_notation, expand_power_of_ten,
    normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL
)


def _normalize_pre_number(text):
    # 1. Powers of ten
    text = re.sub(r'\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b', expand_power_of_ten, text, flags=re.IGNORECASE)
    text = re.sub(r'\b10\^([-+]?\d+)\b', lambda m: f"mười mũ {('trừ ' + n2w(m.group(1)[1:])) if m.group(1).startswith('-') else n2w(m.group(1).replace('+', ''))}", text)
    
    # 2. Abbreviations and Date/Time
    text = expand_abbreviations(text)
    text = normalize_date(text)
    text = normalize_time(text)
    
    # 3. Ranges and Arrows
    def _range_or_dash(m):
        g1, g2, g3, g4 = m.groups()
        if g1 is not None and g2 is not None: # Range
            n1 = re.sub(r'[,.]', '', g1)
            n2 = re.sub(r'[,.]', '', g2)
            if abs(len(n1) - len(n2)) <= 1:
                return f'{g1} đến {g2}'
            return f'{g1} {g2}'
        elif g3 is not None: # Arrow
            return g3.replace('->', ' sang ').replace('=>', ' sang ')
        return ',' # Standalone dash

    text = re.sub(r'''
        (\d+(?:[.,]\d+)?)\s*[–\-—]\s*(\d+(?:[.,]\d+)?)  # Range (groups 1, 2)
        |
        (\s*(?:->|=>)\s*)                                # Arrow (group 3)
        |
        ((?<=\s)[–\-—](?=\s))                            # Standalone dash (group 4)
    ''', _range_or_dash, text, flags=re.VERBOSE)

    return text

def _normalize_units_currency(text):
    # 1. Scientific and compound units
    text = expand_scientific_notation(text)
    text = expand_compound_units(text)
    text = expand_measurement_currency(text)

    # 2. Number style fixes and multi-comma expansion
    def _fix_and_expand_numbers(m):
        g1, g2 = m.groups()

        # fix_english_style_numbers logic for g1
        if g1 is not None:
            val = g1
            has_comma = ',' in val
            has_dot = '.' in val
            if val.count(',') > 1 or (has_comma and has_dot and val.find(',') < val.find('.')):
                return val.replace(',', '').replace('.', ',') if has_dot else val.replace(',', '')
            if has_comma and has_dot:
                return val.replace(',', '').replace('.', ',')
            return val

        # _expand_multi_comma logic for g2
        if g2 is not None:
            res = []
            for s in g2.split(','):
                res.append(' '.join(n2w_single(c) for c in s))
            return ' phẩy '.join(res)
        return m.group(0)

    # Patterns are mutually exclusive to avoid overlapping
    text = re.sub(r'''
        \b(\d{1,3}(?:,\d{3})+(?:\.\d+)?)\b  # English style numbers (g1)
        |
        \b(\d+(?:,\d+){2,})\b               # Multi-comma numbers (g2)
    ''', _fix_and_expand_numbers, text, flags=re.VERBOSE)

    # 3. Float and dot separator normalization
    def _float_or_dot_sep(m):
        g1, g2, g3, g4 = m.groups()
        if g1 is not None: # Float with comma decimal
            int_part = n2w(g1.replace('.', ''))
            dec_part = g2.rstrip('0')
            res = f"{int_part} phẩy {n2w_single(dec_part)}" if dec_part else int_part
            if g3: res += " phần trăm"
            return f" {res} "
        elif g4 is not None: # Dot separated thousands
            return g4.replace('.', '')
        return m.group(0)

    # Patterns are mutually exclusive to avoid overlapping
    text = re.sub(r'''
        (?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?  # Float with comma decimal (g1, g2, g3)
        |
        (?<![\d.])(\d+(?:\.\d{3})+)(?![\d.])    # Dot separated thousands (g4)
    ''', _float_or_dot_sep, text, flags=re.VERBOSE)

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
