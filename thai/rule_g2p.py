# -*- coding: utf-8 -*-
"""Rule-based Thai grapheme-to-phoneme: reads ANY Thai string.

The out-of-vocabulary fallback. The dictionary handles known words; anything
else — new proper names, transliterations, typos — comes here, and here we
must always produce something pronounceable.

# Writing order vs reading order

Every Thai syllable is written in this fixed frame:

    [pre-vowel] C [cluster] [above/below vowel] [tone mark] [post-vowel] [final]
       เแโใไ                    ◌ั◌ิ◌ี◌ึ◌ื◌ุ◌ู◌็        ◌่◌้◌๊◌๋         าะอวย

so the tone mark always sits immediately after the consonant and any
above/below vowel, and BEFORE า/อ/ะ. Getting that slot wrong misreads ข้าว
as "kʰa-wa"; getting it right yields /kʰaːw˥˩/.

# Two rules that carry most of the accuracy

  - **อักษรนำ (leading consonant)**: a high/mid-class consonant with no vowel
    of its own, directly before a low-class sonorant, is read /Ca/ AND lends
    its class to the next syllable — this is why สวัสดี is sa-wàt-dii (low
    tone) and not sa-wát-dii, and why ฉลาด is tɕʰa-làːt.
  - **Tone from (class × liveness × mark)**, never written directly.

# Parsing

Syllable boundaries are ambiguous (ผู้คน is pʰûː-kʰon, not pʰûːk-na), so this
is a DP over the string rather than greedy longest-match: every syllable
reading is an edge with a cost, and the cheapest path wins. Costs prefer
explicit-vowel syllables over inherent-vowel ones, which is exactly what
stops a following onset from being stolen as a coda.
"""
import re

# ── inventories ─────────────────────────────────────────────────────────────
INIT = {
    "ก": "k", "ข": "kʰ", "ฃ": "kʰ", "ค": "kʰ", "ฅ": "kʰ", "ฆ": "kʰ",
    "ง": "ŋ", "จ": "tɕ", "ฉ": "tɕʰ", "ช": "tɕʰ", "ซ": "s", "ฌ": "tɕʰ",
    "ญ": "j", "ฎ": "d", "ฏ": "t", "ฐ": "tʰ", "ฑ": "tʰ", "ฒ": "tʰ",
    "ณ": "n", "ด": "d", "ต": "t", "ถ": "tʰ", "ท": "tʰ", "ธ": "tʰ",
    "น": "n", "บ": "b", "ป": "p", "ผ": "pʰ", "ฝ": "f", "พ": "pʰ",
    "ฟ": "f", "ภ": "pʰ", "ม": "m", "ย": "j", "ร": "r", "ล": "l",
    "ว": "w", "ศ": "s", "ษ": "s", "ส": "s", "ห": "h", "ฬ": "l",
    "อ": "ʔ", "ฮ": "h",
}
FINAL = {
    "ก": "k̚", "ข": "k̚", "ค": "k̚", "ฆ": "k̚", "ง": "ŋ",
    "จ": "t̚", "ช": "t̚", "ซ": "t̚", "ฎ": "t̚", "ฏ": "t̚", "ฐ": "t̚",
    "ฑ": "t̚", "ฒ": "t̚", "ด": "t̚", "ต": "t̚", "ถ": "t̚", "ท": "t̚",
    "ธ": "t̚", "ศ": "t̚", "ษ": "t̚", "ส": "t̚", "ญ": "n", "ณ": "n",
    "น": "n", "ร": "n", "ล": "n", "ฬ": "n", "บ": "p̚", "ป": "p̚",
    "พ": "p̚", "ฟ": "p̚", "ภ": "p̚", "ม": "m", "ย": "j", "ว": "w",
}
CLASS_MID = set("กจฎฏดตบปอ")
CLASS_HIGH = set("ขฃฉฐถผฝศษสห")
SONORANT = set("งญนมยรลวฬณ")
TONE_MARKS = {"่": "ek", "้": "tho", "๊": "tri", "๋": "chattawa"}
THANTHAKHAT = "์"
TRUE_CLUSTERS = {
    "กร", "กล", "กว", "ขร", "ขล", "ขว", "คร", "คล", "คว", "ตร",
    "ปร", "ปล", "ผล", "พร", "พล", "บร", "บล", "ดร", "ฟร", "ฟล",
}
PSEUDO = {"ทร": "s", "จร": "tɕ", "ซร": "s", "สร": "s", "ศร": "s"}
LONG_VOWELS = {"aː", "iː", "ɯː", "uː", "eː", "ɛː", "oː", "ɔː", "ɤː",
               "ia", "ɯa", "ua"}
MID, LOW, FALL, HIGH, RISE = "˧", "˨˩", "˥˩", "˦˥", "˩˩˦"
SONORANT_CODA = {"m", "n", "ŋ", "j", "w"}


def tone_of(cls, vowel, coda, mark):
    live = coda in SONORANT_CODA or (coda == "" and vowel in LONG_VOWELS)
    short = vowel not in LONG_VOWELS
    if mark == "ek":
        return FALL if cls == "low" else LOW
    if mark == "tho":
        return HIGH if cls == "low" else FALL
    if mark == "tri":
        return HIGH
    if mark == "chattawa":
        return RISE
    if cls == "mid":
        return MID if live else LOW
    if cls == "high":
        return RISE if live else LOW
    return MID if live else (HIGH if short else FALL)


def class_of(letter):
    if letter in CLASS_MID:
        return "mid"
    if letter in CLASS_HIGH:
        return "high"
    return "low"


# ── vowel frames ────────────────────────────────────────────────────────────
# (pre, above/below, post, vowel, needs_final). The regex is assembled so the
# tone slot always lands between the above/below vowel and the post vowel.
FRAMES = [
    # ◌ัว FIRST: ว here is the vowel's own glide, never a final consonant.
    # Listed ahead of the closed frames because both match ตัว and the DP
    # keeps whichever is found first at equal cost — "tua" is the right one.
    ("", "ั", "ว", "ua", False),
    ("", "ั", "วะ", "ua", False),
    # closed (an explicit final consonant follows)
    ("เ", "ี", "ย", "ia", True),
    ("เ", "ื", "อ", "ɯa", True),
    ("", "ั", "ว", "ua", True),
    ("เ", "็", "", "e", True),
    ("แ", "็", "", "ɛ", True),
    ("เ", "ิ", "", "ɤː", True),
    ("เ", "", "", "eː", True),
    ("แ", "", "", "ɛː", True),
    ("โ", "", "", "oː", True),
    ("", "", "อ", "ɔː", True),
    ("", "ั", "", "a", True),
    ("", "", "า", "aː", True),
    # /ua/ before a final consonant drops its ◌ั and is written bare -ว-
    # (สวย sǔaj, ด้วย dûaj, ห่วย hùaj). Placed after the า frames so the
    # cluster reading of ควาย (kʰwaːj) still wins.
    ("", "", "ว", "ua", True),
    ("", "ิ", "", "i", True),
    ("", "ี", "", "iː", True),
    ("", "ึ", "", "ɯ", True),
    ("", "ื", "", "ɯː", True),
    ("", "ุ", "", "u", True),
    ("", "ู", "", "uː", True),
    # open (no final consonant); some end in a written ะ
    ("เ", "ี", "ยะ", "ia", False),
    ("เ", "ี", "ย", "ia", False),
    ("เ", "ื", "อะ", "ɯa", False),
    ("เ", "ื", "อ", "ɯa", False),
    ("", "ั", "วะ", "ua", False),
    ("", "ั", "ว", "ua", False),
    ("เ", "", "อะ", "ɤ", False),
    ("เ", "ิ", "", "ɤː", False),
    ("เ", "", "อ", "ɤː", False),
    ("เ", "", "าะ", "ɔ", False),
    ("เ", "", "า", "aw", False),
    ("เ", "", "ะ", "e", False),
    ("แ", "", "ะ", "ɛ", False),
    ("โ", "", "ะ", "o", False),
    ("เ", "", "", "eː", False),
    ("แ", "", "", "ɛː", False),
    ("โ", "", "", "oː", False),
    ("ใ", "", "", "aj", False),
    ("ไ", "", "", "aj", False),
    ("", "", "อ", "ɔː", False),
    ("", "", "ำ", "am", False),
    ("", "ั", "", "a", False),
    ("", "", "า", "aː", False),
    ("", "ิ", "", "i", False),
    ("", "ี", "", "iː", False),
    ("", "ึ", "", "ɯ", False),
    # ือ without a leading เ is plain /ɯː/ (มือ, คือ, สือ); the อ is only a
    # carrier. With เ it is the diphthong /ɯa/ (เสือ) — handled above.
    ("", "ื", "อ", "ɯː", False),
    ("", "ื", "", "ɯː", False),
    ("", "ุ", "", "u", False),
    ("", "ู", "", "uː", False),
    ("", "", "ะ", "a", False),
]

_TONE = r"(?P<t>[่้๊๋]?)"


def _build(pre, ab, post, needs_final):
    rx = re.escape(pre) if pre else ""
    rx += r"(?P<c1>[ก-ฮ])(?P<c2>[รลว]?)"
    rx += re.escape(ab) if ab else ""
    rx += _TONE
    rx += re.escape(post) if post else ""
    if needs_final:
        rx += r"(?P<f>[ก-ฮ])"
    return re.compile(rx)


COMPILED = [(_build(p, a, o, f), v, f, bool(a or o or p)) for p, a, o, v, f in FRAMES]
GLIDE = {"ย": "j", "ว": "w"}
# a written vowel/tone right after a candidate glide means the glide is the
# NEXT syllable's onset, not this syllable's coda
_VOWEL_CHARS = set("ะัาำิีึืุูเแโใไ็่้๊๋์")


def _onset(c1, c2):
    """Returns (phoneme, class_letter, consumed_c2)."""
    if c2:
        pair = c1 + c2
        if pair in PSEUDO:
            return PSEUDO[pair], c1, True
        if pair in TRUE_CLUSTERS:
            return INIT[c1] + INIT[c2], c1, True
        return INIT.get(c1, ""), c1, False
    return INIT.get(c1, ""), c1, False


def _readings(s, i):
    """All syllable readings starting at s[i] as (syllable, next_i, cost)."""
    out = []
    rest = s[i:]
    # leading silent ห / อ — including when a pre-vowel precedes it (เหลือ)
    forced, skip = None, 0
    m = re.match(r"^([เแโใไ]?)(ห)([งญนมยรลวฬณ])", rest)
    if m:
        forced, skip = "high", 1
        rest = m.group(1) + rest[len(m.group(1)) + 1:]
    else:
        m = re.match(r"^([เแโใไ]?)(อ)(ย)", rest)
        if m:
            forced, skip = "mid", 1
            rest = m.group(1) + rest[len(m.group(1)) + 1:]

    for rx, vowel, needs_final, explicit in COMPILED:
        mt = rx.match(rest)
        if not mt:
            continue
        c1, c2 = mt.group("c1"), mt.group("c2")
        onset, cls_letter, used = _onset(c1, c2)
        if not onset:
            continue
        if c2 and not used:
            continue  # c2 belongs to the next syllable; another frame covers it
        end = mt.end()
        coda = ""
        v = vowel
        if needs_final:
            f = mt.group("f")
            if f not in FINAL:
                continue
            coda = FINAL[f]
        else:
            if v == "am":
                v, coda = "a", "m"
            elif v == "aw":
                v, coda = "a", "w"
            elif v == "aj":
                v, coda = "a", "j"
                # ไ-ย spells a silent ย (ไทย, ไชย): consume it, add no sound
                if end < len(rest) and rest[end] == "ย":
                    end += 1
            elif end < len(rest) and rest[end] in GLIDE:
                nxt = rest[end + 1: end + 2]
                if not (nxt and nxt in _VOWEL_CHARS):
                    coda = GLIDE[rest[end]]
                    end += 1
        cls = forced or class_of(cls_letter)
        mark = TONE_MARKS.get(mt.group("t") or "", "none")
        syl = f"{onset}{v}{coda}{tone_of(cls, v, coda, mark)}"
        cost = 1 if explicit else 2
        out.append((syl, i + skip + end, cost))
    return out


def _inherent(s, i):
    """Vowel-less syllable readings: C+/a/ (open) or C+/o/+coda (closed).

    Two spellings override the inherent /o/:
      - ◌รร (ro han) is /a/ + /n/, or bare /a/ when a final follows
        (ธรรม tʰam, สรรค์ san, ครรภ์ kʰan);
      - a lone ร final takes /ɔː/, not /o/ (ละคร la-kʰɔːn, นคร na-kʰɔːn).
    """
    out = []
    c = s[i]
    if c not in INIT:
        return out
    cls = class_of(c)
    nxt = s[i + 1: i + 2]
    nxt2 = s[i + 2: i + 3]
    if s[i + 1: i + 3] == "รร":
        after = s[i + 3: i + 4]
        if after and after in FINAL and after not in _VOWEL_CHARS:
            coda = FINAL[after]
            out.append((f"{INIT[c]}a{coda}{tone_of(cls,'a',coda,'none')}", i + 4, 1))
        else:
            out.append((f"{INIT[c]}an{tone_of(cls,'a','n','none')}", i + 3, 1))
        return out
    if nxt == "ร" and not (nxt2 and (nxt2 in _VOWEL_CHARS or nxt2 in FINAL)):
        out.append((f"{INIT[c]}ɔːn{tone_of(cls,'ɔː','n','none')}", i + 2, 2))
    if nxt in FINAL and not (nxt2 and nxt2 in _VOWEL_CHARS):
        coda = FINAL[nxt]
        out.append((f"{INIT[c]}o{coda}{tone_of(cls, 'o', coda, 'none')}", i + 2, 2))
    out.append((f"{INIT[c]}a{tone_of(cls, 'a', '', 'none')}", i + 1, 3))
    return out


def _leading(s, i):
    """อักษรนำ: high/mid consonant + low sonorant, no vowel between. Produces
    TWO syllables at once, the second inheriting the first's class."""
    if i + 1 >= len(s):
        return []
    c1, c2 = s[i], s[i + 1]
    if c1 not in INIT or class_of(c1) not in ("high", "mid"):
        return []
    if c2 not in SONORANT or c1 + c2 in TRUE_CLUSTERS or c1 + c2 in PSEUDO:
        return []
    if s[i + 2: i + 3] in ("", *_VOWEL_CHARS) and s[i + 2: i + 3] == "":
        return []
    cls1 = class_of(c1)
    first = f"{INIT[c1]}a{tone_of(cls1, 'a', '', 'none')}"
    out = []
    # read the second syllable normally, then re-tone it with c1's class
    for syl, j, cost in _readings(s, i + 1) + _inherent(s, i + 1):
        m = re.match(r"^(.*?)([˩˨˧˦˥]+)$", syl)
        if not m:
            continue
        body = m.group(1)
        vm = re.match(rf"^(?:{'|'.join(sorted(set(INIT.values()) | {'kʰr','kʰl','kʰw','kr','kl','kw','tr','pr','pl','pʰr','pʰl','br','bl','dr','fr','fl'}, key=len, reverse=True))})", body)
        if not vm:
            continue
        rest_body = body[vm.end():]
        vmm = re.match(r"^(ia|ɯa|ua|aː|iː|ɯː|uː|eː|ɛː|oː|ɔː|ɤː|a|i|ɯ|u|e|ɛ|o|ɔ|ɤ)(.*)$", rest_body)
        if not vmm:
            continue
        v, coda = vmm.group(1), vmm.group(2)
        retoned = f"{body}{tone_of(cls1, v, coda, 'none')}"
        out.append(((first, retoned), j, cost + 1))
    return out


def g2p_word(word):
    """Read a whole Thai string; returns a list of canonical syllables."""
    word = word.replace("ํา", "ำ")
    word = re.sub(r"[ก-ฮ][ิีุูั]?" + THANTHAKHAT, "", word)   # silent letters
    n = len(word)
    INF = float("inf")
    best = [INF] * (n + 1)
    back = [None] * (n + 1)
    best[0] = 0
    for i in range(n):
        if best[i] == INF:
            continue
        edges = []
        for syls, j, cost in _leading(word, i):
            edges.append((list(syls), j, cost))
        for syl, j, cost in _readings(word, i) + _inherent(word, i):
            edges.append(([syl], j, cost))
        if not edges:                      # unreadable char: skip at a price
            edges.append(([], i + 1, 10))
        for syls, j, cost in edges:
            if j <= i or j > n:
                continue
            if best[i] + cost < best[j]:
                best[j] = best[i] + cost
                back[j] = (i, syls)
    if best[n] == INF:
        return []
    out, k = [], n
    while k > 0:
        i, syls = back[k]
        out = list(syls) + out
        k = i
    return out


if __name__ == "__main__":
    import sys
    sys.stdout.reconfigure(encoding="utf-8")
    tests = {
        "สวัสดี": "sa˨˩ wat̚˨˩ diː˧", "ครับ": "kʰrap̚˦˥", "เขา": "kʰaw˩˩˦",
        "ฉลาด": "tɕʰa˨˩ laːt̚˨˩", "หลอก": "lɔːk̚˨˩", "ให้": "haj˥˩",
        "ประเมิน": "pra˨˩ mɤːn˧", "น้ำ": "naːm˦˥", "ผู้คน": "pʰuː˥˩ kʰon˧",
        "คน": "kʰon˧", "เดิน": "dɤːn˧", "ข้าว": "kʰaːw˥˩", "ไทย": "tʰaj˧",
        "แม่": "mɛː˥˩", "หนังสือ": "naŋ˩˩˦ sɯː˩˩˦", "เสียว": "siaw˩˩˦",
    }
    for w, want in tests.items():
        got = " ".join(g2p_word(w))
        print(f"{'OK ' if got == want else 'NO '} {w:10} got={got:24} want={want}")
