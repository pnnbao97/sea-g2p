"""Structural invariants of the normalizer.

The tests here do not check *how* a specific input is read — `test_normalize.py`
covers that. They check properties that must hold for every input, and they
exist because two whole families of defects slipped past example-based tests:

1. **Silent character deletion.** Normalization ends by stripping anything it
   does not recognise. A symbol whose reading was never declared vanishes, the
   output still reads fluently, and nobody hears the loss. `10⁻³` was read as
   "mười lập phương" — six orders of magnitude off — because the superscript
   minus was deleted rather than spoken.

2. **Silent word deletion.** Rules that collapse a duplicated leading word
   ("ngày ngày 15/3" -> "ngày 15/3") also ate genuine reduplication, so
   "ngày ngày vẫn đông khách" lost a word. Reduplication is pervasive in
   Vietnamese, so this class is both easy to hit and hard to notice.

Both families share a signature: the output is shorter than it should be, yet
perfectly natural to the ear. Only a property test catches that reliably.
"""

import re
import pytest
from sea_g2p import Normalizer

norm = Normalizer()


# ── 1. No character disappears without a reading ────────────────────────────

# Characters that realistically show up in Vietnamese technical prose. Every one
# must either be spoken or be an explicitly declared drop; see the Rust `audit`
# module for the taxonomy.
CHARACTER_INVENTORY = [
    ("maths operators", "+ - × ÷ = ≠ ≈ ≤ ≥ ± ∓ < >"),
    ("maths symbols", "√ ∫ ∑ ∏ ∞ ∈ ∉ ⊂ ∪ ∩ ∀ ∃ ∴ ∵"),
    ("greek letters", "α β γ δ ε θ λ μ π ρ σ τ φ ω Δ ∆ Σ Ω"),
    ("superscripts", "x⁰ x¹ x² x³ x⁴ x⁵ x⁶ x⁷ x⁸ x⁹ xⁿ xⁱ 10⁻³ 10⁺⁶"),
    ("subscripts", "a₀ a₁ a₂ a₃ a₄ a₅ a₆ a₇ a₈ a₉ aₙ aᵢ x₊ x₋"),
    ("fractions", "½ ⅓ ⅔ ¼ ¾ ⅕ ⅖ ⅗ ⅘ ⅙ ⅚ ⅛ ⅜ ⅝ ⅞"),
    ("currency", "$ € £ ¥ ₩ ₫"),
    ("units", "° ℃ ℉ µ Ω Å %"),
    ("number sets", "ℝ ℕ ℤ ℚ ℂ"),
    ("punctuation", ". , ; : ! ? … ‥ – — ‐ ‑"),
    ("quotes", "' \" ‘ ’ “ ” „ « »"),
    ("brackets", "( ) [ ] { } ⟨ ⟩"),
    ("technical", "@ # & / \\ | ~ ^ * _ < >"),
    ("arrows", "→ ← ↔ ⇒ ⇐ ⇔"),
]


@pytest.mark.parametrize("label,sample", CHARACTER_INVENTORY)
def test_no_character_is_silently_dropped(label, sample):
    """Every character in the inventory has a declared fate."""
    unmapped = norm.audit(sample)
    assert not unmapped, (
        f"{label}: {unmapped} would be deleted with no reading. "
        f"Give them a mapping, or declare them in the audit module's "
        f"INTENTIONALLY_DROPPED set if dropping them is correct."
    )


def test_audit_flags_a_genuinely_unknown_symbol():
    """The guard must actually fire — otherwise it proves nothing."""
    assert norm.audit("phím ⌘ và ⌥ trên máy Mac") == ["⌘", "⌥"]


CORPUS_FILES = ["test_output_num.md", "test_output_math.md", "test_output_mix.md"]


@pytest.mark.parametrize("filename", CORPUS_FILES)
def test_corpus_inputs_are_fully_covered(filename, request):
    """No corpus sentence contains a character we would drop in silence."""
    path = request.config.rootpath / filename
    if not path.exists():
        pytest.skip(f"{filename} not generated")
    offenders = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        m = re.match(r"\*\*(\d+)\.\*\*(?:\s*\[[^\]]*\])?\s*(.*)", line)
        if not m:
            continue
        unmapped = norm.audit(m.group(2))
        if unmapped:
            offenders[m.group(1)] = unmapped
    assert not offenders, f"characters with no reading in {filename}: {offenders}"


# ── 2. No word disappears ───────────────────────────────────────────────────

# Plain Vietnamese prose: no digits, no symbols, no abbreviations. Normalization
# may lowercase and re-space, but it must not add or remove a single word.
WORD_PRESERVING = [
    # Reduplication, the pattern that actually broke. Words that double as
    # date/time units are the dangerous ones, so they are over-represented.
    "ngày ngày vẫn đông khách xếp hàng từ sáng sớm",
    "nhóc ngân ngày ngày năn nỉ anh nam đèo đi chơi",
    "suốt năm năm trời anh ấy vẫn đợi một lời hồi âm",
    "tháng tháng bà đều đóng tiền học cho cháu",
    "tuần tuần đều họp giao ban vào buổi sáng",
    "đêm đêm nghe tiếng sóng vỗ ngoài xa",
    "chiều chiều ra đứng ngõ sau trông về quê mẹ",
    "sáng sáng ông cụ vẫn đi bộ quanh hồ",
    "người người nhà nhà thi đua sản xuất",
    "giờ giờ phút phút đều trôi qua thật chậm",
    "hôm hôm nào cũng thấy chị ấy ngồi đó",
    "mùa mùa lúa chín vàng khắp cánh đồng",
    # Ordinary prose, to catch collateral damage from any collapse rule.
    "cô giáo dặn cả lớp giữ trật tự trong giờ kiểm tra",
    "anh ấy nói rằng sẽ quay lại vào một ngày không xa",
    "bà nội kể chuyện ngày xưa cho các cháu nghe",
    "trời mưa rất to nên cả nhà ở lại thêm một hôm",
    "chúng tôi đi qua những cánh rừng bạt ngàn",
    # Double negation: "âm âm" must survive because no number follows.
    "giá trị âm âm sẽ thành giá trị dương",
]


@pytest.mark.parametrize("sentence", WORD_PRESERVING)
def test_plain_prose_keeps_every_word(sentence):
    """Word-for-word identity on input that needs no rewriting at all."""
    got = norm.normalize(sentence).rstrip(".").split()
    expected = sentence.split()
    assert got == expected, (
        f"word count {len(expected)} -> {len(got)}; normalization must not "
        f"add or drop words in plain prose"
    )


# Reduplicated leading words are collapsed *only* when the duplicate was
# inserted by a date pass, i.e. a number follows.
COLLAPSE_ONLY_BEFORE_NUMBERS = [
    ("cuộc họp vào ngày 15/3", "ngày mười lăm tháng ba"),
    ("hôm 15/3 cả nhà đi chơi", "hôm mười lăm tháng ba"),
    ("mùng 5/5 là tết đoan ngọ", "mùng năm tháng năm"),
]


@pytest.mark.parametrize("sentence,expected_fragment", COLLAPSE_ONLY_BEFORE_NUMBERS)
def test_lead_word_collapse_still_works_with_numbers(sentence, expected_fragment):
    """The collapse rules must remain effective where they are correct."""
    assert expected_fragment in norm.normalize(sentence)


# ── 3. Normalization is idempotent on its own output ────────────────────────

IDEMPOTENT_SAMPLES = [
    "cuộc họp lúc 14h30 ngày 15/3 tại phòng 5",
    "nhiệt độ âm 5 độ c và gió cấp 11",
    "chiếc xe biển 51h-123.45 chạy 120 km/h",
    "công thức e = mc² của einstein",
    "nồng độ 10⁻³ mol trên lít",
]


@pytest.mark.parametrize("sentence", IDEMPOTENT_SAMPLES)
def test_normalizing_twice_changes_nothing(sentence):
    """Output is a fixed point: feeding it back through must be a no-op.

    A rule that keeps firing on its own output is a rule with an unbounded
    effect, and that is how "âm âm" and duplicated lead words appear.
    """
    once = norm.normalize(sentence)
    assert norm.normalize(once) == once
