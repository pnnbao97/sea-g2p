import re
from .num2vi import n2w, n2w_single
from .vi_resources import (
    _measurement_key_vi, _currency_key, _CURRENCY_SYMBOL_MAP, _MAGNITUDE_P, _NUMERIC_P
)

# Re-exports or internal helpers used by multiple functions
def _expand_scientific(num_str):
    num_lower = num_str.lower()
    e_idx = num_lower.find('e')
    base, exp = num_str[:e_idx], num_str[e_idx+1:]

    # Base normalization
    if base.count('.') == 1:
        parts = base.split('.')
        dec_part = parts[1].rstrip('0')
        base_norm = f"{n2w(parts[0])} chấm {n2w_single(dec_part)}" if dec_part else n2w(parts[0])
    elif base.count(',') == 1:
        parts = base.split(',')
        dec_part = parts[1].rstrip('0')
        base_norm = f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}" if dec_part else n2w(parts[0])
    else:
        base_norm = n2w(base.replace(',', '').replace('.', ''))

    # Exponent normalization
    exp_val = exp.lstrip('+')
    exp_norm = f"trừ {n2w(exp_val[1:])}" if exp_val.startswith('-') else n2w(exp_val)
    return f"{base_norm} nhân mười mũ {exp_norm}"

def _expand_mixed_sep(num_str):
    if num_str.rfind('.') > num_str.rfind(','): # English style (1,299.5)
        parts = num_str.replace(',', '').split('.')
    else: # Vietnamese style (1.299,5)
        parts = num_str.replace('.', '').split(',')

    dec_part = parts[1].rstrip('0')
    if not dec_part:
        return n2w(parts[0])
    return f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}"

def _expand_single_sep(num_str):
    if ',' in num_str:
        parts = num_str.split(',')
        if len(parts) > 2 or (len(parts) == 2 and len(parts[1]) == 3):
            return n2w(num_str.replace(',', ''))

        # Strip trailing zeros for decimals
        dec_part = parts[1].rstrip('0')
        if not dec_part:
            return n2w(parts[0])
        return f"{n2w(parts[0])} phẩy {n2w_single(dec_part)}"

    parts = num_str.split('.')
    if len(parts) > 2 or (len(parts) == 2 and len(parts[1]) == 3):
        return n2w(num_str.replace('.', ''))

    # Strip trailing zeros for decimals
    dec_part = parts[1].rstrip('0')
    if not dec_part:
        return n2w(parts[0])
    return f"{n2w(parts[0])} chấm {n2w_single(dec_part)}"

def _expand_number_with_sep(num_str):
    if not num_str: return ""
    if 'e' in num_str.lower(): return _expand_scientific(num_str)
    if ',' in num_str and '.' in num_str: return _expand_mixed_sep(num_str)
    if ',' in num_str or '.' in num_str: return _expand_single_sep(num_str)
    return n2w(num_str)

# Pre-compiled regex for compound units
RE_COMPOUND_UNIT = re.compile(rf"\b{_NUMERIC_P}?\s*([a-zμµ²³°]+)/([a-zμµ²³°0-9]+)\b", re.IGNORECASE)

# Currency and Measurement consolidation
_ALL_UNITS_MAP = {k.lower(): v for k, v in _measurement_key_vi.items()}
_ALL_UNITS_MAP.update({k.lower(): v for k, v in _currency_key.items() if k != '%'})
# Ensure 'm' maps to 'mét' as it's more common for compound units and standalone lowercase
_ALL_UNITS_MAP['m'] = "mét"

_SAFE_STANDALONE = ["km", "cm", "mm", "kg", "mg", "usd", "vnd", "ph"]
_SORTED_UNITS_KEYS = sorted(list(_measurement_key_vi.keys()) + [k for k in _currency_key.keys() if k != '%'], key=len, reverse=True)
_UNITS_RE_PATTERN = "|".join(re.escape(u) for u in _SORTED_UNITS_KEYS)

RE_UNITS_WITH_NUM = re.compile(rf"(?<![\d.,]){_NUMERIC_P}{_MAGNITUDE_P}\s*({_UNITS_RE_PATTERN})\b", re.IGNORECASE)
RE_STANDALONE_UNIT = re.compile(rf"(?<![\d.,])\b({'|'.join(re.escape(u) for u in _SAFE_STANDALONE)})\b", re.IGNORECASE)

RE_CURRENCY_PREFIX_SYMBOL = re.compile(rf"([$€¥£₩])\s*{_NUMERIC_P}{_MAGNITUDE_P}", re.IGNORECASE)
RE_CURRENCY_SUFFIX_SYMBOL = re.compile(rf"{_NUMERIC_P}{_MAGNITUDE_P}([$€¥£₩])", re.IGNORECASE)
RE_PERCENTAGE = re.compile(rf"{_NUMERIC_P}\s*%", re.IGNORECASE)

def expand_measurement_currency(text):
    """Optimized single-pass measurement and currency expansion."""
    def _repl_units(m):
        num = m.group(1)
        mag = m.group(2) or ""
        unit = m.group(3)

        # Special case for M/m to distinguish million/meter
        if unit == 'M': full = "triệu"
        elif unit == 'm': full = "mét"
        else: full = _ALL_UNITS_MAP.get(unit.lower(), unit)

        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()

    def _repl_standalone(m):
        unit = m.group(1).lower()
        return f" {_ALL_UNITS_MAP.get(unit, unit)} "

    text = RE_UNITS_WITH_NUM.sub(_repl_units, text)
    text = RE_STANDALONE_UNIT.sub(_repl_standalone, text)
    return text

def expand_currency(text):
    """Maintains backward compatibility by calling the optimized unified expander and symbol expander."""
    text = expand_currency_symbols(text)
    # We still need to call the general units expander because it handles named units like 'usd', 'vnd'
    # but to match old behavior exactly, we might want to be careful.
    # Actually, the optimized expander handles all named units.
    return expand_measurement_currency(text)

def expand_measurement(text):
    """Maintains backward compatibility."""
    return expand_measurement_currency(text)

def expand_currency_symbols(text):
    def _repl_symbol(m, is_prefix=True):
        symbol = m.group(1 if is_prefix else 3)
        num = m.group(2 if is_prefix else 1)
        mag = m.group(3 if is_prefix else 2) or ""
        full = _CURRENCY_SYMBOL_MAP.get(symbol, "")
        expanded_num = _expand_number_with_sep(num)
        return f"{expanded_num} {mag} {full}".replace("  ", " ").strip()

    text = RE_CURRENCY_PREFIX_SYMBOL.sub(lambda m: _repl_symbol(m, True), text)
    text = RE_CURRENCY_SUFFIX_SYMBOL.sub(lambda m: _repl_symbol(m, False), text)
    text = RE_PERCENTAGE.sub(lambda m: f"{_expand_number_with_sep(m.group(1))} phần trăm", text)
    return text

def expand_compound_units(text):
    def _repl_compound(m):
        num_str = m.group(1) or ""
        if not num_str:
            return m.group(0)

        num = _expand_number_with_sep(num_str)
        u1_raw = m.group(2)
        u2_raw = m.group(3)

        # Follow original logic: use raw case for M/m if possible, otherwise lower
        def _get_unit(u_raw):
            if u_raw == 'M': return "triệu"
            if u_raw == 'm': return "mét"
            return _ALL_UNITS_MAP.get(u_raw.lower(), u_raw)

        full1 = _get_unit(u1_raw)
        full2 = _get_unit(u2_raw)
        res = f" {full1} trên {full2} "
        if num:
            res = f"{num} {res}"
        return res

    return RE_COMPOUND_UNIT.sub(_repl_compound, text)

def fix_english_style_numbers(m):
    val = m.group(0)
    has_comma = ',' in val
    has_dot = '.' in val
    if val.count(',') > 1 or (has_comma and has_dot and val.find(',') < val.find('.')):
        return val.replace(',', '').replace('.', ',') if has_dot else val.replace(',', '')
    if has_comma and has_dot:
        return val.replace(',', '').replace('.', ',')
    return val

def expand_power_of_ten(m):
    from .others import normalize_others
    base = m.group(1)
    exp = m.group(2)
    base_norm = normalize_others(base).strip()
    exp_val = exp.replace('+', '')
    exp_norm = ("trừ " + n2w(exp_val[1:])) if exp_val.startswith('-') else n2w(exp_val)
    return f" {base_norm} nhân mười mũ {exp_norm} "

def expand_scientific_notation(text):
    pattern = re.compile(r'\b(\d+(?:[.,]\d+)?e[+-]?\d+)\b', re.IGNORECASE)
    return pattern.sub(lambda m: _expand_number_with_sep(m.group(1)), text)
