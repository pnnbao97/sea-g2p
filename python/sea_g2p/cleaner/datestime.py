import re
from .num2vi import n2w

day_in_month = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
_date_seperator = r"(\/|-|\.)"
_short_date_seperator = r"(\/|-)"

# Compiled Regular Expressions
RE_FULL_DATE = re.compile(r"\b(\d{1,2})" + _date_seperator + r"(\d{1,2})" + _date_seperator + r"(\d{4})\b", re.IGNORECASE)
RE_DAY_MONTH = re.compile(r"\b(\d{1,2})" + _short_date_seperator + r"(\d{1,2})\b", re.IGNORECASE)
RE_MONTH_YEAR = re.compile(r"\b(\d{1,2})" + _date_seperator + r"(\d{4})\b", re.IGNORECASE)
RE_FULL_TIME = re.compile(r"\b(\d+)(g|:|h)(\d{1,2})(p|:|m)(\d{1,2})(?:\s*(giây|s|g))?\b", re.IGNORECASE)
RE_TIME = re.compile(r"\b(\d+)(g|:|h)(\d{1,2})(?:\s*(phút|p|m))?\b", re.IGNORECASE)
RE_REDUNDANT_NGAY = re.compile(r'\bngày\s+ngày\b', re.IGNORECASE)
RE_REDUNDANT_THANG = re.compile(r'\btháng\s+tháng\b', re.IGNORECASE)
RE_REDUNDANT_NAM = re.compile(r'\bnăm\s+năm\b', re.IGNORECASE)

def _is_valid_date(day, month):
    try:
        day, month = int(day), int(month)
        return 1 <= month <= 12 and 1 <= day <= day_in_month[month - 1]
    except (ValueError, IndexError): return False

def _expand_full_date(match):
    day, sep1, month, sep2, year = match.groups()
    if _is_valid_date(day, month):
        day = str(int(day))
        m_val = "tư" if int(month) == 4 else n2w(str(int(month)))
        return f"ngày {n2w(day)} tháng {m_val} năm {n2w(year)}"
    return match.group(0)

def _expand_day_month(match):
    day, sep, month = match.groups()
    if _is_valid_date(day, month):
        day = str(int(day))
        m_val = "tư" if int(month) == 4 else n2w(str(int(month)))
        return f"ngày {n2w(day)} tháng {m_val}"
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
    text = RE_FULL_DATE.sub(_expand_full_date, text)
    text = RE_MONTH_YEAR.sub(
        lambda m: f"tháng { 'tư' if int(m.group(1)) == 4 else n2w(str(int(m.group(1)))) } năm {n2w(m.group(3))}",
        text
    )
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
    return text
