# -*- coding: utf-8 -*-
"""Indonesian affix analysis, used to derive pronunciations from known roots.

The gold lexicon holds 17.8k words but covers only 63% of Wikipedia tokens.
The gap is almost entirely morphology: `sebuah`, `terdapat`, `disebut`,
`termasuk`, `berbagai` are all affixed forms of roots the lexicon already
has. Deriving them beats guessing them — every affix has a fixed
pronunciation, so the only unknown, the root's schwa pattern, comes from
gold rather than from a heuristic.

# The one hard part: meN- and peN-

These prefixes assimilate to the root's first consonant AND delete it when
it is voiceless:

    meN- + pukul -> memukul     (p disappears, prefix becomes mem-)
    meN- + tulis -> menulis     (t disappears, men-)
    meN- + kirim -> mengirim    (k disappears, meng-)
    meN- + sapu  -> menyapu     (s disappears, meny-)
    meN- + baca  -> membaca     (b stays, mem-)

So recovering the root from a surface form means *restoring* a consonant
that is not written. Each candidate restoration is checked against the
lexicon, and only a hit is accepted — a wrong guess simply fails to resolve.
"""

# prefix -> (phonemes, consonant to restore or None, whether it was deleted)
SIMPLE_PREFIXES = [
    ("memper", "m ə m p ə r", None),
    ("mempel", "m ə m p ə l", None),
    ("diper", "d i p ə r", None),
    ("keber", "k ə b ə r", None),
    ("ber", "b ə r", None),
    ("ter", "t ə r", None),
    ("per", "p ə r", None),
    ("di", "d i", None),
    ("se", "s ə", None),
    ("ke", "k ə", None),
    ("be", "b ə", None),
    ("te", "t ə", None),
    ("pe", "p ə", None),
    ("me", "m ə", None),
]

# meN-/peN- surface form -> (phonemes, consonants that may have been deleted,
# consonants that may simply follow)
NASAL_PREFIXES = [
    ("meng", "m ə ŋ", ["k", ""], ["g", "h", "a", "e", "i", "o", "u"]),
    ("meny", "m ə ɲ", ["s"], []),
    ("mem", "m ə m", ["p"], ["b", "f", "v"]),
    ("men", "m ə n", ["t"], ["d", "c", "j", "z"]),
    ("peng", "p ə ŋ", ["k", ""], ["g", "h", "a", "e", "i", "o", "u"]),
    ("peny", "p ə ɲ", ["s"], []),
    ("pem", "p ə m", ["p"], ["b", "f", "v"]),
    ("pen", "p ə n", ["t"], ["d", "c", "j", "z"]),
]

# (suffix, phonemes, minimum stem length). The derivational suffixes need a
# four-letter stem, since -i and -an are short enough that a three-letter
# remainder is usually coincidence — that is how "perak" (silver) came out as
# per- + ak. The clitics -nya, -lah and -kah attach to anything, including
# three-letter roots: ibu -> ibunya, isi -> isinya, air -> airnya.
SUFFIXES = [
    ("kannya", "k a n ɲ a", 4),
    ("annya", "a n ɲ a", 4),
    ("nya", "ɲ a", 3),
    # -ku and -mu were found by scanning which endings attach to the most
    # distinct known roots: 151 and 86 respectively, far above the noise
    # floor. -pun is the same class of enclitic (apa+pun, mana+pun).
    ("ku", "k u", 3),
    ("mu", "m u", 3),
    ("pun", "p u n", 3),
    ("kan", "k a n", 4),
    ("lah", "l a h", 3),
    ("kah", "k a h", 3),
    ("an", "a n", 4),
    ("i", "i", 4),
]


def analyse(word, lookup):
    """Pronunciation of `word` derived from `lookup`, or None.

    `lookup(root)` returns the root's phonemes as a list, or None.
    Tries suffix stripping and prefix stripping, including the meN-/peN-
    consonant restorations, and accepts the first analysis whose root is
    known.
    """
    w = word.lower()
    direct = lookup(w)
    if direct:
        return direct

    # Suffixes first: they do not interact with the prefix rules. The stem
    # must be at least four letters — the productive suffixes -i and -an are
    # short enough that a three-letter remainder is usually a coincidence,
    # which is how "perak" (silver) came out as per- + ak.
    for suf, phones, min_stem in SUFFIXES:
        if w.endswith(suf) and len(w) - len(suf) >= min_stem:
            stem = w[: -len(suf)]
            inner = analyse(stem, lookup)
            if inner:
                return inner + phones.split()

    # nasal prefixes, with the deleted consonant restored
    for pre, phones, deleted, plain in NASAL_PREFIXES:
        if not w.startswith(pre) or len(w) - len(pre) < 3:
            continue
        rest = w[len(pre):]
        for c in deleted:
            root = c + rest
            if len(root) >= 3:
                got = lookup(root)
                if got:
                    # The restored consonant is written in the root but NOT
                    # pronounced in the derived word — it assimilated into the
                    # prefix nasal. Keeping it yields pəm-pərintah for
                    # pemerintah instead of pə-mərintah.
                    body = got[1:] if c and got else got
                    return phones.split() + body
        if rest[:1] in plain:
            got = lookup(rest)
            if got:
                return phones.split() + got

    for pre, phones, _ in SIMPLE_PREFIXES:
        if w.startswith(pre) and len(w) - len(pre) >= 4:
            got = lookup(w[len(pre):])
            if got:
                return phones.split() + got
    return None
