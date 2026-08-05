# -*- coding: utf-8 -*-
"""Thai and Indonesian normalization, exercised through the Python API.

Until this file existed the whole Python suite was Vietnamese. That mattered
more than a coverage number: `Normalizer.audit` ignored `lang` and always ran
the Vietnamese audit, so the guard against silent deletion was checking Thai
and Indonesian text against Vietnamese rules — and nothing here noticed,
because nothing here ran them.
"""

import pytest

from sea_g2p import Normalizer


@pytest.fixture(scope="module")
def th():
    return Normalizer(lang="th")


@pytest.fixture(scope="module")
def idn():
    return Normalizer(lang="id")


# ── the pipeline is selected by lang ────────────────────────────────────────

def test_lang_selects_the_pipeline(th, idn):
    # a stale install once made every call fall through to Vietnamese; these
    # two outputs share no vocabulary with it
    assert th.normalize("50%") == "ห้าสิบ เปอร์เซ็นต์"
    assert idn.normalize("50%") == "lima puluh persen"


def test_unsupported_lang_is_rejected():
    with pytest.raises(ValueError):
        Normalizer(lang="xx")


# ── audit follows lang ──────────────────────────────────────────────────────

def test_audit_uses_the_matching_language(th, idn):
    # ∮ is declared by neither table, so both must report it
    assert th.audit("∮") == ["∮"]
    assert idn.audit("∮") == ["∮"]
    # and each language's own inventory must come back clean
    assert th.audit("฿500 30°C 50% ๑๒๓ ๆ ฯ") == []
    assert idn.audit("Rp1.250 30° 50% 3,14") == []


def test_audit_reports_a_prime_nothing_reads(th, idn):
    # a lone prime is a measurement with no pair and no degree: unread
    assert th.audit("5'") == ["'"]
    assert idn.audit("5'") == ["'"]
    # a quotation mark is genuinely droppable and must not cry wolf
    assert th.audit('"คำ"') == []
    assert idn.audit('"kata"') == []


def test_audit_reports_a_numeric_hyphen_only_when_unread(th, idn):
    for n in (th, idn):
        assert n.audit("-5") == []
        assert n.audit("10-20") == []


# ── units, primes, ratios ───────────────────────────────────────────────────

def test_units_read_as_words_not_letters(th, idn):
    assert th.normalize("60 km/h") == "หกสิบ กิโลเมตร ต่อ ชั่วโมง"
    assert idn.normalize("60 km/jam") == "enam puluh kilometer per jam"
    assert th.normalize("50 m2") == "ห้าสิบ ตารางเมตร"
    assert idn.normalize("50 m2") == "lima puluh meter persegi"


def test_prime_marks_are_measurements(th, idn):
    assert th.normalize("5'6\"") == "ห้า ฟุต หก นิ้ว"
    assert idn.normalize("5'6\"") == "lima kaki enam inci"


def test_e_notation(th, idn):
    assert "คูณ สิบ ยกกำลัง" in th.normalize("1.5e10")
    assert "kali sepuluh pangkat" in idn.normalize("1,5e10")


# ── Indonesian orthography ──────────────────────────────────────────────────

def test_indonesian_ordinals_and_hyphens(idn):
    assert idn.normalize("ke-3") == "ketiga"
    assert idn.normalize("abad ke-20") == "abad kedua puluh"
    assert idn.normalize("COVID-19") == "COVID sembilan belas"
    # reduplication keeps its hyphen
    assert idn.normalize("orang-orang") == "orang-orang"
    assert idn.normalize("do'a Jum'at") == "doa Jumat"


# ── ordinary prose is left alone ────────────────────────────────────────────

@pytest.mark.parametrize("text,expected", [
    ("เขาอายุ 25 ปี และมีลูก 2 คน", "เขาอายุ ยี่สิบห้า ปี และมีลูก สอง คน"),
    ("หน้า 5 บทที่ 2", "หน้า ห้า บทที่ สอง"),
])
def test_thai_prose_is_untouched(th, text, expected):
    assert th.normalize(text) == expected


@pytest.mark.parametrize("text,expected", [
    ("Saya makan 2 kali sehari", "Saya makan dua kali sehari"),
    ("Halaman 5 bab 2", "Halaman lima bab dua"),
])
def test_indonesian_prose_is_untouched(idn, text, expected):
    assert idn.normalize(text) == expected


def test_batch_matches_single(th, idn):
    texts = ["50%", "60 km/h", "14:30"]
    assert th.normalize(texts) == [th.normalize(t) for t in texts]
    texts = ["50%", "ke-3", "COVID-19"]
    assert idn.normalize(texts) == [idn.normalize(t) for t in texts]


# ── weekdays and lone letters ───────────────────────────────────────────────

def test_weekday_abbreviations_need_a_cue(th, idn):
    assert th.normalize("วันจ.") == "วันจันทร์"
    assert idn.normalize("hari Sen") == "hari Senin"
    # uncued, the same letters mean something else entirely
    assert "ศาสตราจารย์" in th.normalize("ศ.ดร.สมชาย")
    assert idn.normalize("Sen depan") == "Sen depan"


def test_lone_latin_letter_is_spelled_in_thai(th):
    assert th.normalize("วิตามิน C") == "วิตามิน ซี"
    assert th.normalize("กระดาษ A4") == "กระดาษ เอ สี่"
    assert "iPhone" in th.normalize("ผมใช้ iPhone")


def test_thai_letter_names_keep_the_phonemes_thai():
    """The point of spelling the letter: staying inside the Thai inventory."""
    from sea_g2p import G2P

    out = G2P(lang="th").convert("วิตามิน C")
    # a Thai tone letter on the spelled name, not the English primary stress
    assert "ˈ" not in out, out
    assert "siː" in out, out
