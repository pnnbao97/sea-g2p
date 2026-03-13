import re
import string
from .num2vi import n2w, n2w_single

from .numerical import normalize_number_vi
from .datestime import normalize_date, normalize_time
from .text_norm import (
    normalize_others, expand_measurement, expand_currency,
    expand_compound_units, expand_abbreviations, expand_standalone_letters,
    expand_scientific_notation, fix_english_style_numbers, expand_power_of_ten,
    normalize_technical, normalize_emails, RE_TECHNICAL, RE_EMAIL,
    _RE_ACRONYMS_EXCEPTIONS
)

RE_POWER_OF_TEN_EXPLICIT = re.compile(r'\b(\d+(?:[.,]\d+)?)\s*[x*×]\s*10\^([-+]?\d+)\b', re.IGNORECASE)
RE_POWER_OF_TEN_IMPLICIT = re.compile(r'\b10\^([-+]?\d+)\b')
RE_RANGE = re.compile(r'(\d+(?:[,.]\d+)?)\s*[–\-—]\s*(\d+(?:[,.]\d+)?)')
RE_DASH_TO_COMMA = re.compile(r'(?<=\s)[–\-—](?=\s)')
RE_TO_SANG = re.compile(r'\s*(?:->|=>)\s*')
RE_ENGLISH_STYLE_NUMBERS = re.compile(r'\b\d{1,3}(?:,\d{3})+(?:\.\d+)?\b')
RE_MULTI_COMMA = re.compile(r'\b(\d+(?:,\d+){2,})\b')
RE_FLOAT_WITH_COMMA = re.compile(r'(?<![\d.])(\d+(?:\.\d{3})*),(\d+)(%)?')
RE_STRIP_DOT_SEP_RE = re.compile(r'(?<![\d.])\d+(?:\.\d{3})+(?![\d.])')

RE_EXTRA_SPACES = re.compile(r'[ \t\xA0]+')
RE_EXTRA_COMMAS = re.compile(r',\s*,')
RE_COMMA_BEFORE_PUNCT = re.compile(r',\s*([.!?;])')
RE_SPACE_BEFORE_PUNCT = re.compile(r'\s+([,.!?;:])')
RE_MISSING_SPACE_AFTER_PUNCT = re.compile(r'([.,!?;:])(?=[^\s\d<])')

RE_ENTOKEN = re.compile(r'ENTOKEN\d+', flags=re.IGNORECASE)
RE_INTERNAL_EN_TAG = re.compile(r'(__start_en__.*?__end_en__|<en>.*?</en>)', flags=re.IGNORECASE)
RE_DOT_BETWEEN_DIGITS = re.compile(r'(\d+)\.(\d+)')


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
    text = RE_POWER_OF_TEN_EXPLICIT.sub(expand_power_of_ten, text)
    text = RE_POWER_OF_TEN_IMPLICIT.sub(lambda m: f"mười mũ {('trừ ' + n2w(m.group(1)[1:])) if m.group(1).startswith('-') else n2w(m.group(1).replace('+', ''))}", text)
    
    text = expand_abbreviations(text)
    text = normalize_date(text)
    text = normalize_time(text)
    
    def _range_sub(m):
        n1 = m.group(1).replace(',', '').replace('.', '')
        n2 = m.group(2).replace(',', '').replace('.', '')
        # Only treat as a numeric range if digit counts are similar (within 1)
        if abs(len(n1) - len(n2)) <= 1:
            return f'{m.group(1)} đến {m.group(2)}'
        return f'{m.group(1)} {m.group(2)}'
    text = RE_RANGE.sub(_range_sub, text)
    text = RE_DASH_TO_COMMA.sub(',', text)
    text = RE_TO_SANG.sub(' sang ', text)
    return text

def _normalize_units_currency(text):
    text = expand_scientific_notation(text)
    text = expand_compound_units(text)
    text = expand_measurement(text)
    text = expand_currency(text)

    text = RE_ENGLISH_STYLE_NUMBERS.sub(fix_english_style_numbers, text)

    def _expand_multi_comma(m):
        res = []
        for s in m.group(1).split(','):
            res.append(' '.join(n2w_single(c) for c in s))
        return ' phẩy '.join(res)
    text = RE_MULTI_COMMA.sub(_expand_multi_comma, text)

    text = RE_FLOAT_WITH_COMMA.sub(_expand_float, text)
    text = RE_STRIP_DOT_SEP_RE.sub(_strip_dot_sep, text)
    return text

def _normalize_post_number(text):
    text = normalize_others(text)
    text = normalize_number_vi(text)
    return text

def _cleanup_whitespace(text):
    text = RE_EXTRA_SPACES.sub(' ', text)
    text = RE_EXTRA_COMMAS.sub(',', text)
    text = RE_COMMA_BEFORE_PUNCT.sub(r'\1', text)
    text = RE_SPACE_BEFORE_PUNCT.sub(r'\1', text)
    
    # Add space after punctuation if missing (e.g. "USD.Tiếng Việt" -> "USD. Tiếng Việt")
    # But carefully avoiding tags like <en> or numbers (already handled if they meant to be decimals)
    text = RE_MISSING_SPACE_AFTER_PUNCT.sub(r'\1 ', text)
    
    return text.strip().strip(',')

def clean_vietnamese_text(text):
    mask_map = {}

    def protect(match):
        idx = len(mask_map)
        mask = f"mask{str(idx).zfill(4)}mask".translate(str.maketrans('0123456789', string.ascii_lowercase[:10]))
        mask_map[mask] = match.group(0)
        return mask

    # Simple regex to protect existing tags, avoiding potential ReDoS in nested patterns
    text = RE_ENTOKEN.sub(protect, text)

    # Normalize URLs and Emails early and protect them
    def protect_url_email(match):
        orig = match.group(0)
        
        # Priority 1: Email has @ (most specific pattern)
        if '@' in orig:
            return protect(re.Match if False else type('Match', (), {'group': lambda self, n: normalize_emails(orig)})())

        # Priority 2: Check if it's explicitly in our specialized technical exceptions
        # Move this after email to ensure email patterns aren't partially matched by exceptions
        if _RE_ACRONYMS_EXCEPTIONS.fullmatch(orig):
            from .vi_resources import _combined_exceptions
            return protect(re.Match if False else type('Match', (), {'group': lambda self, n: _combined_exceptions[orig]})())

        # Priority 3: Standard technical normalization (URLs, Paths, Versions, etc.)
        normed = normalize_technical(orig)
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
    text = RE_INTERNAL_EN_TAG.sub(protect, text)
    text = expand_standalone_letters(text)

    # Convert remaining dots between digits to ' chấm ' (for IPs, versions that survived)
    text = RE_DOT_BETWEEN_DIGITS.sub(r'\1 chấm \2', text)
    # Recursively handle multiple dots like 10.10.10.10
    while RE_DOT_BETWEEN_DIGITS.search(text):
        text = RE_DOT_BETWEEN_DIGITS.sub(r'\1 chấm \2', text)

    for mask, original in mask_map.items():
        text = text.replace(mask, original)
        text = text.replace(mask.lower(), original)

    # Final conversion of any remaining __start_en__ tags
    text = text.replace('__start_en__', '<en>').replace('__end_en__', '</en>')
    
    # Finally, remove any other underscores after internal tag replacement
    text = text.replace('_', ' ')

    text = _cleanup_whitespace(text)
    return text.lower()
