import re
from .num2vi import n2w

day_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
_date_seperator = r"(\/|-|\.)"
_short_date_seperator = r"(\/|-)"

# Compiled Regular Expressions
# Grouping date patterns: Full date (D/M/Y), Month/Year (M/Y), Day/Month (D/M)
RE_DATE_COMBINED = re.compile(r'''
    \b(\d{1,2})([/\-.])(\d{1,2})\2(\d{4})\b  # D/M/Y
    |
    \b(\d{1,2})([/\-.])(\d{4})\b             # M/Y
    |
    \b(\d{1,2})([/\-])(\d{1,2})\b             # D/M
''', re.VERBOSE | re.IGNORECASE)

RE_TIME_COMBINED = re.compile(r'''
    \b(\d+)([g:h])(\d{1,2})(?:([p:m])(\d{1,2})(?:\s*(giây|s|g))?|(?:\s*(phút|p|m))?)\b
''', re.VERBOSE | re.IGNORECASE)

RE_REDUNDANT_NGAY = re.compile(r'\bngày\s+ngày\b', re.IGNORECASE)

def _is_valid_date(day, month):
    try:
        day, month = int(day), int(month)
        return 1 <= month <= 12 and 1 <= day <= day_in_month[month - 1]
    except (ValueError, IndexError): return False

def _expand_full_date(match):
    day, sep1, month, sep2, year = match.groups()
    if _is_valid_date(day, month):
        day = str(int(day))
        month = str(int(month))
        return f"ngày {n2w(day)} tháng {n2w(month)} năm {n2w(year)}"
    return match.group(0)

def _expand_day_month(match):
    day, sep, month = match.groups()
    if _is_valid_date(day, month):
        day = str(int(day))
        month = str(int(month))
        return f"ngày {n2w(day)} tháng {n2w(month)}"
    return match.group(0)

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
    def _repl_date(m):
        if m.group(1): # D/M/Y
            d, sep, m_val, y = m.group(1), m.group(2), m.group(3), m.group(4)
            if _is_valid_date(d, m_val):
                return f"ngày {n2w(str(int(d)))} tháng {n2w(str(int(m_val)))} năm {n2w(y)}"
        elif m.group(5): # M/Y
            m_val, sep, y = m.group(5), m.group(6), m.group(7)
            return f"tháng {n2w(str(int(m_val)))} năm {n2w(y)}"
        elif m.group(8): # D/M
            d, sep, m_val = m.group(8), m.group(9), m.group(10)
            if _is_valid_date(d, m_val):
                return f"ngày {n2w(str(int(d)))} tháng {n2w(str(int(m_val)))}"
        return m.group(0)

    text = RE_DATE_COMBINED.sub(_repl_date, text)
    text = RE_REDUNDANT_NGAY.sub('ngày', text)
    text = re.sub(r'\b(tháng|năm)\s+\1\b', r'\1', text, flags=re.IGNORECASE)
    return text

def normalize_time(text):
    def _repl_time(m):
        h, sep1, m_val = m.group(1), m.group(2), m.group(3)
        sep2, s_val, s_unit = m.group(4), m.group(5), m.group(6)
        m_unit = m.group(7)

        try:
            h_int = int(h)
            m_int = int(m_val)
        except ValueError:
            return m.group(0)

        if not (0 <= m_int < 60):
            return m.group(0)

        if s_val: # Full time with seconds
            return f"{n2w(_norm_time_part(h))} giờ {n2w(_norm_time_part(m_val))} phút {n2w(_norm_time_part(s_val))} giây"

        # Time with only hours and minutes
        if sep1 == ':':
            if h_int < 24:
                return f"{n2w(_norm_time_part(h))} giờ {n2w(_norm_time_part(m_val))} phút"
            else:
                return f"{n2w(h)} phút {n2w(_norm_time_part(m_val))} giây"
        else:
            return f"{n2w(_norm_time_part(h))} giờ {n2w(_norm_time_part(m_val))} phút"

    text = RE_TIME_COMBINED.sub(_repl_time, text)
    return text
