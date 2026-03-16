import re
from .num2vi import n2w
from .vi_resources import DATE_KEYWORDS, MATH_KEYWORDS

day_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
_date_seperator = r"(\/|-|\.)"
_short_date_seperator = r"(\/|-)"

# Compiled Regular Expressions
RE_FULL_DATE = re.compile(r"(?<![\d,.])(\d{1,2})" + _date_seperator + r"(\d{1,2})" + _date_seperator + r"(\d{4})(?![\d,.])", re.IGNORECASE)
RE_DAY_MONTH = re.compile(r"(?<![\d,.])(\d{1,2})" + _short_date_seperator + r"(\d{1,2})(?![\d,.])", re.IGNORECASE)
RE_MONTH_YEAR = re.compile(r"(?<![\d,.])(\d{1,2})" + _date_seperator + r"(\d{4})(?![\d,.])", re.IGNORECASE)
RE_FULL_TIME = re.compile(r"\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b", re.IGNORECASE)
RE_TIME = re.compile(r"\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b", re.IGNORECASE)
RE_HOUR_CONTEXT = re.compile(r'\b(\d+)g\s*(sáng|trưa|chiều|tối|khuya)\b', re.IGNORECASE)
RE_LUC_HOUR = re.compile(r'\blúc\s*(\d+)g\b', re.IGNORECASE)
RE_REDUNDANT_NGAY = re.compile(r'\bngày\s+ngày\b', re.IGNORECASE)
RE_REDUNDANT_THANG = re.compile(r'\btháng\s+tháng\b', re.IGNORECASE)
RE_REDUNDANT_NAM = re.compile(r'\bnăm\s+năm\b', re.IGNORECASE)

def _is_valid_date(day, month):
    try:
        day, month = int(day), int(month)
        return 1 <= month <= 12 and 1 <= day <= day_in_month[month - 1]
    except (ValueError, IndexError): return False

def _get_context_words(match, window_size=3):
    """Extract context words around the match within a window."""
    text = match.string
    start, end = match.span()
    
    # Simple tokenization by whitespace for context
    left_part = text[:start].split()
    right_part = text[end:].split()
    
    # We want to keep symbols like +, -, *, /, =, >, <, etc. for math context
    def clean(w):
        w = w.lower()
        # Only strip punctuation that isn't a math symbol
        return w.strip(",.!?;()[]{}")
    
    context = left_part[-window_size:] + right_part[:window_size]
    return [clean(w) for w in context if w]

def _expand_full_date(match):
    day, sep1, month, sep2, year = match.groups()
    if _is_valid_date(day, month):
        day = str(int(day))
        m_val = "tư" if int(month) == 4 else n2w(str(int(month)))
        return f"ngày {n2w(day)} tháng {m_val} năm {n2w(year)}"
    return match.group(0)

def _expand_month_year(match):
    month_str, sep, year_str = match.groups()
    m = int(month_str)
    y = int(year_str)
    
    # Month must be 1-12, Year should be reasonable (e.g. <= 2500 as per user request)
    if 1 <= m <= 12 and y <= 2500:
        m_val = "tư" if m == 4 else n2w(str(m))
        return f"tháng {m_val} năm {n2w(str(y))}"
    
    return match.group(0)

def _expand_day_month(match):
    day_str, sep, month_str = match.groups()
    a = int(day_str)
    b = int(month_str)
    
    context_words = _get_context_words(match, window_size=3)
    
    # Math symbols to check explicitly since they might not be in MATH_KEYWORDS
    math_symbols = {'+', '-', '*', 'x', '×', '/', '=', '>', '<', '≥', '≤', '≈', '±'}
    
    is_valid_date = _is_valid_date(day_str, month_str)

    # Heuristic:
    # 1. If math keywords/symbols are present -> fraction
    if any(w in context_words for w in MATH_KEYWORDS) or any(s in context_words for s in math_symbols):
        return f"{n2w(day_str)} trên {n2w(month_str)}"

    # 2. If month or day has leading zero (e.g. 21/02, 01/12) and is valid -> date
    month_has_leading_zero = month_str.startswith('0') and len(month_str) > 1
    day_has_leading_zero = day_str.startswith('0') and len(day_str) > 1
    if is_valid_date and (month_has_leading_zero or day_has_leading_zero):
        day = str(a)
        m_val = "tư" if b == 4 else n2w(str(b))
        return f"ngày {n2w(day)} tháng {m_val}"

    # 3. If date keywords are present and valid -> date
    if is_valid_date and any(w in context_words for w in DATE_KEYWORDS):
        day = str(a)
        m_val = "tư" if b == 4 else n2w(str(b))
        return f"ngày {n2w(day)} tháng {m_val}"

    # 4. If it's NOT a valid date, do NOT normalize it here (let other cleaners handle it)
    # UNLESS it has a leading zero, which strongly suggests it should be read by digits or as a fraction/date
    if not is_valid_date:
        if day_str.startswith('0') or month_str.startswith('0'):
            return f"{n2w(day_str)} trên {n2w(month_str)}"
        return match.group(0)

    # 5. Fallback for valid dates with no specific context -> fraction
    return f"{n2w(day_str)} trên {n2w(month_str)}"

def _norm_time_part(s):
    return '0' if s == '00' else s

def _expand_time(match):
    h, sep, m, suffix = match.groups()
    try:
        h_int = int(h)
        m_int = int(m)
    except ValueError:
        return match.group(0)

    if 0 <= m_int < 60:
        if sep == ':':
            if h_int < 24:
                return f"{n2w(_norm_time_part(h))} giờ {n2w(_norm_time_part(m))} phút"
            else:
                # Handle durations like 27:45 (MM:SS)
                return f"{n2w(h)} phút {n2w(_norm_time_part(m))} giây"
        else:
            # Handle forms like 27h45 (Always HH:MM even if H > 23 for durations)
            return f"{n2w(_norm_time_part(h))} giờ {n2w(_norm_time_part(m))} phút"
    return match.group(0)

def normalize_date(text):
    text = RE_FULL_DATE.sub(_expand_full_date, text)
    text = RE_MONTH_YEAR.sub(_expand_month_year, text)
    text = RE_DAY_MONTH.sub(_expand_day_month, text)
    text = RE_REDUNDANT_NGAY.sub('ngày', text)
    text = RE_REDUNDANT_THANG.sub('tháng', text)
    text = RE_REDUNDANT_NAM.sub('năm', text)
    return text

def normalize_time(text):
    text = RE_FULL_TIME.sub(
        lambda m: f"{n2w(_norm_time_part(m.group(1)))} giờ {n2w(_norm_time_part(m.group(3)))} phút {n2w(_norm_time_part(m.group(5)))} giây",
        text
    )
    text = RE_TIME.sub(_expand_time, text)

    # Contextual rules for 'g' as 'giờ'
    text = RE_HOUR_CONTEXT.sub(lambda m: f"{n2w(m.group(1))} giờ {m.group(2)}", text)
    text = RE_LUC_HOUR.sub(lambda m: f"lúc {n2w(m.group(1))} giờ", text)

    return text
