# -*- coding: utf-8 -*-
"""Independent Thai tone-rule checker.

Implements the standard orthographic tone rules from scratch (no tltk, no
gold data) and predicts the tone of structurally unambiguous monosyllables.
Used as a cross-check on dictionary entries: a systematic disagreement means
either the entry or our understanding is wrong — both worth catching.

Rules implemented (standard Thai school grammar):

  consonant classes
    mid  (อักษรกลาง): ก จ ฎ ฏ ด ต บ ป อ
    high (อักษรสูง) : ข ฃ ฉ ฐ ถ ผ ฝ ศ ษ ส ห
    low  (อักษรต่ำ) : the remaining 24

  syllable liveness
    live: ends in a sonorant (m n ŋ j w) or a long open vowel
    dead: ends in a stop (p̚ t̚ k̚) or a short open vowel (ʔ)

  tone table (mark × class × liveness)
    no mark : mid+live -> mid | mid+dead -> low
              high+live -> rising | high+dead -> low
              low+live -> mid | low+dead+short -> high | low+dead+long -> falling
    mai ek  ( ่): mid -> low | high -> low | low -> falling
    mai tho ( ้): mid -> falling | high -> falling | low -> high
    mai tri ( ๊): mid -> high   (others: rare, loan/colloquial)
    mai chattawa ( ๋): mid -> rising

  leading-consonant overrides
    ห + sonorant (ง ญ น ม ย ร ล ว): silent ห, syllable takes HIGH class
    อ + ย (อย่า อยู่ อย่าง อยาก):   silent อ, syllable takes MID class
    cluster C+r/l/w: class follows the first consonant
"""
MID = set("กจฎฏดตบปอ")
HIGH = set("ขฃฉฐถผฝศษสห")
LOW = set("คฅฆงชซฌญฑฒณทธนพฟภมยรลวฬฮ")
SONORANT_TH = set("งญนมยรลวณฬ")

MARKS = {"่": "ek", "้": "tho", "๊": "tri", "๋": "chattawa"}

# canonical tone symbols
MID_T, LOW_T, FALL, HIGH_T, RISE = "˧", "˨˩", "˥˩", "˦˥", "˩˩˦"

SONORANT_CODA = {"m", "n", "ŋ", "j", "w"}
STOP_CODA = {"p̚", "t̚", "k̚"}
LONG_VOWELS = {"aː", "iː", "ɯː", "uː", "eː", "ɛː", "oː", "ɔː", "ɤː",
               # the three diphthongs count as long for tone purposes
               "ia", "ɯa", "ua"}


def consonant_class(thai_syllable):
    """Class of a Thai-script syllable, or None if not confidently derivable."""
    s = thai_syllable
    # skip preposed vowels
    i = 0
    while i < len(s) and s[i] in "เแโใไ":
        i += 1
    if i >= len(s):
        return None
    c = s[i]
    if c == "ห" and i + 1 < len(s) and s[i + 1] in SONORANT_TH:
        return "high"
    if c == "อ" and i + 1 < len(s) and s[i + 1] == "ย":
        return "mid"
    if c in MID:
        return "mid"
    if c in HIGH:
        return "high"
    if c in LOW:
        # low-class letter followed by a high/mid letter? (e.g. ทร-) —
        # cluster class still follows first letter for true clusters; the
        # เ...ทร case ทร->s is a pseudo-cluster we refuse to judge.
        return "low"
    return None


def liveness(vowel, coda):
    if coda in SONORANT_CODA:
        return "live"
    if coda in STOP_CODA:
        return "dead"
    if coda in ("", "ʔ"):
        return "live" if vowel in LONG_VOWELS else "dead"
    return None  # loanword codas (f s l tɕʰ): no rule judgement


def predict(cls, live, mark, vowel):
    short = vowel not in LONG_VOWELS
    if mark == "none":
        if cls == "mid":
            return MID_T if live == "live" else LOW_T
        if cls == "high":
            return RISE if live == "live" else LOW_T
        if cls == "low":
            if live == "live":
                return MID_T
            return HIGH_T if short else FALL
    elif mark == "ek":
        return FALL if cls == "low" else LOW_T
    elif mark == "tho":
        return HIGH_T if cls == "low" else FALL
    elif mark == "tri":
        return HIGH_T if cls == "mid" else None
    elif mark == "chattawa":
        return RISE if cls == "mid" else None
    return None


def check_word(thai, syls, check_syl):
    """Check a MONOsyllabic dict entry; return None (no judgement),
    'ok', or ('mismatch', predicted, actual)."""
    if len(syls) != 1:
        return None
    marks = [MARKS[c] for c in thai if c in MARKS]
    if len(marks) > 1:
        return None
    mark = marks[0] if marks else "none"
    # refuse pseudo-clusters and special letters our class logic can't judge
    for bad in ("ทร", "ฤ", "ฦ", "รร"):
        if bad in thai:
            return None
    # ศร สร ซร: ร silent, class follows the ศ/ส/ซ letter itself -> fine.
    cls = consonant_class(thai)
    if cls is None:
        return None
    onset, vowel, coda, tone = check_syl(syls[0])
    live = liveness(vowel, coda)
    if live is None:
        return None
    pred = predict(cls, live, mark, vowel)
    if pred is None:
        return None
    return "ok" if pred == tone else ("mismatch", pred, tone)
