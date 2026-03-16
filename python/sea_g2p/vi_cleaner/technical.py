"""Technical string normalization: URLs, emails, file paths, and slash-separated numbers."""
import re
from .num2vi import n2w, n2w_single
from .vi_resources import _vi_letter_names, _common_email_domains, _DOMAIN_SUFFIX_MAP

RE_TECHNICAL = re.compile(r'''
    \b(?:https?|ftp)://[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b(?:www\.)[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]+\b
    |
    \b[A-Za-z0-9.\-]+(?:\.com|\.vn|\.net|\.org|\.gov|\.io|\.biz|\.info)(?:/[A-Za-z0-9.\-_~:/?#\[\]@!$&\'()*+,;=]*)?\b
    |
    (?<!\w)/[a-zA-Z0-9._\-/]{2,}\b
    |
    \b[a-zA-Z]:\\[a-zA-Z0-9._\\\-]+\b
    |
    \b[a-zA-Z0-9._\-]+\.(?:txt|log|tar|gz|zip|sh|py|js|cpp|h|json|xml|yaml|yml|md|csv|pdf|docx|xlsx|exe|dll|so|config)\b
    |
    \b[a-zA-Z][a-zA-Z0-9]*(?:[._\-][a-zA-Z0-9]+){2,}\b
    |
    \b(?:[a-fA-F0-9]{1,4}:){3,7}[a-fA-F0-9]{1,4}\b
''', re.VERBOSE | re.IGNORECASE)

RE_EMAIL = re.compile(r'\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b')
RE_TECH_SPLIT = re.compile(r'([./:?&=/_ \-\\#])')
RE_EMAIL_SPLIT = re.compile(r'([._\-+])')
RE_SLASH_NUMBER = re.compile(r'(?<![\d,.])(\d+)/(\d+)(?![\d,.])')
_DOMAIN_SUFFIXES_RE = re.compile(r'\.(com|vn|net|org|edu|gov|io|biz|info)\b', re.IGNORECASE)

def normalize_technical(text: str) -> str:
    """Normalize URLs, file paths, and other technical strings into spoken Vietnamese."""
    import re as std_re
    def _repl_tech(m):
        orig = m.group(0)

        # Protocol handling
        rest = orig
        res = []
        if '://' in orig.lower():
            p_idx = orig.lower().find('://')
            protocol = orig[:p_idx]
            # Space out protocol if it looks like an acronym (short & uppercase)
            if protocol.isupper() and len(protocol) <= 4:
                p_norm = " ".join(protocol.lower())
            elif len(protocol) <= 3:
                p_norm = " ".join(protocol.lower())
            else:
                p_norm = protocol.lower()
            res.append(f"__start_en__{p_norm}__end_en__")
            rest = orig[p_idx+3:]
        elif orig.startswith('/'):
            res.append('gạch')
            rest = orig[1:]

        # Simple segments based on delimiters including backslash for Windows paths and # for URL fragments
        segments = RE_TECH_SPLIT.split(rest)
        idx = 0
        while idx < len(segments):
            s = segments[idx]
            if not s: # Empty string from split, e.g., between delimiter and next char
                idx += 1
                continue

            elif s == '.':
                # Peek next segment for suffix map
                next_seg = ""
                for j in range(idx + 1, len(segments)):
                    if segments[j] and segments[j] not in './:?&=/_ -\\':
                        next_seg = segments[j]
                        break
                if next_seg.lower() in _DOMAIN_SUFFIX_MAP:
                    res.append('chấm')
                    res.append(_DOMAIN_SUFFIX_MAP[next_seg.lower()])
                    # Move idx forward to consume the segment we just peeked
                    idx += 1
                    while idx < len(segments) and (not segments[idx] or segments[idx].lower() != next_seg.lower()):
                        idx += 1
                    idx += 1 # move past the next_seg
                    continue
                res.append('chấm')
            elif s == '/':
                res.append('gạch')
            elif s == '\\':
                res.append('gạch')
            elif s == '-':
                res.append('gạch ngang')
            elif s == '_':
                res.append('gạch dưới')
            elif s == ':':
                res.append('hai chấm')
            elif s == '?':
                res.append('hỏi')
            elif s == '&':
                res.append('và')
            elif s == '=':
                res.append('bằng')
            elif s == '#':
                res.append('thăng')
            elif s.lower() in _DOMAIN_SUFFIX_MAP:
                res.append(_DOMAIN_SUFFIX_MAP[s.lower()])
            elif s.isalnum() and s.isascii():
                if s.isdigit():
                    # Technical IDs, hex segments, versions usually read as individual digits
                    res.append(" ".join(n2w_single(c) for c in s))
                else:
                    # Split into letters and digits
                    sub_tokens = std_re.findall(r'[a-zA-Z]+|\d+', s)
                    if len(sub_tokens) > 1:
                        for t in sub_tokens:
                            if t.isdigit():
                                res.append(" ".join(n2w_single(c) for c in t))
                            else:
                                val = t.lower()
                                if t.isupper() and len(t) <= 4:
                                    val = " ".join(val.lower())
                                elif len(val) <= 2 and len(val) > 0:
                                    val = " ".join(val.lower())
                                res.append(f"__start_en__{val}__end_en__")
                    else:
                        if s.isdigit():
                            res.append(" ".join(n2w_single(c) for c in s))
                        else:
                            val = s.lower()
                            if s.isupper() and len(s) <= 4:
                                val = " ".join(val)
                            elif len(val) <= 2 and len(val) > 0: # e.g. 'io' -> 'i o'
                                val = " ".join(val)
                            res.append(f"__start_en__{val}__end_en__")
            else:
                for char in s.lower():
                    if char.isalnum():
                        if char.isdigit(): res.append(n2w_single(char))
                        else: res.append(_vi_letter_names.get(char, char))
                    else: res.append(char)
            idx += 1

        return " ".join(res).replace("  ", " ").strip()
    return RE_TECHNICAL.sub(_repl_tech, text)

def normalize_emails(text: str) -> str:
    """Normalize email addresses into a spoken Vietnamese format."""
    def _repl_email(m):
        email = m.group(0)
        parts = email.split('@')
        if len(parts) != 2: return email

        user_part, domain_part = parts

        def _norm_segment(s):
            if not s: return ""
            if s.isdigit(): return n2w(s)
            if s.isalnum() and s.isascii():
                sub_tokens = re.findall(r'[a-zA-Z]+|\d+', s)
                if len(sub_tokens) > 1:
                    res_parts = []
                    for t in sub_tokens:
                        if t.isdigit():
                            res_parts.append(n2w(t))
                        else:
                            res_parts.append(f"__start_en__{t.lower()}__end_en__")
                    return " ".join(res_parts)
                val = s.lower()
                # Use English tags for segments to avoid character-by-character spelling
                return f"__start_en__{val}__end_en__"

            # Character-by-character fallback for mixed/unrecognized segments
            res = []
            for char in s.lower():
                if char.isalnum():
                    if char.isdigit(): res.append(n2w_single(char))
                    else: res.append(_vi_letter_names.get(char, char))
                else: res.append(char)
            return " ".join(res)

        def _process_part(p, is_domain=False):
            # Split by delimiters but keep them: . _ - +
            segments = RE_EMAIL_SPLIT.split(p)
            res = []
            idx = 0
            while idx < len(segments):
                s = segments[idx]
                if not s:
                    idx += 1
                    continue
                if s == '.':
                    # Special check for domain suffixes
                    if is_domain:
                        next_seg = None
                        peek_idx = -1
                        for j in range(idx + 1, len(segments)):
                            if segments[j] and segments[j] not in '._-+':
                                next_seg = segments[j]
                                peek_idx = j
                                break

                        if next_seg and next_seg.lower() in _DOMAIN_SUFFIX_MAP:
                            res.append('chấm')
                            res.append(_DOMAIN_SUFFIX_MAP[next_seg.lower()])
                            idx = peek_idx + 1
                            continue
                    res.append('chấm')
                elif s == '_': res.append('gạch dưới')
                elif s == '-': res.append('gạch ngang')
                elif s == '+': res.append('cộng')
                else:
                    res.append(_norm_segment(s))
                idx += 1
            return " ".join(res)

        user_norm = _process_part(user_part)

        domain_part_lower = domain_part.lower()
        if domain_part_lower in _common_email_domains:
            domain_norm = _common_email_domains[domain_part_lower]
        else:
            domain_norm = _process_part(domain_part, is_domain=True)

        return f"{user_norm} a còng {domain_norm}".replace("  ", " ").strip()

    return RE_EMAIL.sub(_repl_email, text)

def normalize_slashes(text: str) -> str:
    """Normalize slash-separated number pairs (e.g. '3/4') to 'ba trên bốn'."""
    def _repl(m):
        n1 = m.group(1)
        n2 = m.group(2)
        # Fallback for all slashes (including technical ratios, fractions, addresses, etc.)
        return f"{n2w(n1)} trên {n2w(n2)}"
    return RE_SLASH_NUMBER.sub(_repl, text)
