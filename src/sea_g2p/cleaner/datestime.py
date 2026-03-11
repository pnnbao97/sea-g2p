import re
from .num2vi import n2w

day_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
_date_seperator = r"(\/|-|\.)"
_short_date_seperator = r"(\/|-)"

# Compiled Regular Expressions
# Grouping date patterns: Full date (D/M/Y), Month/Year (M/Y), Day/Month (D/M)
# Patterns are structured to avoid overlapping alternatives and catastrophic backtracking
RE_DATE_COMBINED = re.compile(r'''
    \b(\d{1,2})                 # Day or Month (group 1)
    (?:
        ([/\-.])(\d{1,2})       # Separator (group 2) and Month/Day (group 3)
        \2(\d{4})               # Same separator and Year (group 4)
        |
        ([/\-])(\d{1,2})\b       # Separator (group 5) and Month/Day (group 6) (only / or -)
        |
        ([/\-.])(\d{4})\b        # Separator (group 7) and Year (group 8)
    )
''', re.VERBOSE | re.IGNORECASE)

RE_TIME_COMBINED = re.compile(r'''
    \b(\d+)([g:h])(\d{1,2})     # Hour (group 1), sep (group 2), Minute (group 3)
    (?:
        ([p:m])(\d{1,2})        # sep (group 4), Second (group 5)
        (?:\s*(giây|s|g))?      # Unit (group 6)
        |
        (?:\s*(phút|p|m))       # Unit (group 7)
    )?\b
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
        # Full date (group 1, 2, 3, 4)
        if m.group(4):
            d, m_val, y = m.group(1), m.group(3), m.group(4)
            if _is_valid_date(d, m_val):
                return f"ngày {n2w(str(int(d)))} tháng {n2w(str(int(m_val)))} năm {n2w(y)}"
        # Day/Month (group 1, 5, 6)
        elif m.group(6):
            d, m_val = m.group(1), m.group(6)
            if _is_valid_date(d, m_val):
                return f"ngày {n2w(str(int(d)))} tháng {n2w(str(int(m_val)))}"
        # Month/Year (group 1, 7, 8)
        elif m.group(8):
            m_val, y = m.group(1), m.group(8)
            return f"tháng {n2w(str(int(m_val)))} năm {n2w(y)}"
        return m.group(0)

    text = RE_DATE_COMBINED.sub(_repl_date, text)
    text = RE_REDUNDANT_NGAY.sub('ngày', text)
    text = re.sub(r'\b(tháng|năm)\s+\1\b', r'\1', text, flags=re.IGNORECASE)
    return text

def normalize_time(text):
    def _repl_time(m):
        h, sep1, m_val = m.group(1), m.group(2), m.group(3)
        sep2, s_val, s_unit = m.group(4), m.group(5), m.group(6)

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
