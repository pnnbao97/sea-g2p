import pytest
from sea_g2p import Normalizer, G2P, SEAPipeline


@pytest.fixture
def normalizer():
    return Normalizer()


@pytest.fixture
def g2p():
    return G2P()


@pytest.fixture
def pipeline():
    return SEAPipeline(lang="vi")


# ── Ellipsis / multi-dot normalization (always on, không phụ thuộc punc_norm) ──

ELLIPSIS_CASES = [
    # "…" (U+2026), "‥" (U+2025), "․" (U+2024) và "..." đều quy về một "."
    ("Xin chào các bạn… hôm nay trời đẹp lắm nhé",
     "xin chào các bạn. hôm nay trời đẹp lắm nhé"),
    ("Tôi nghĩ.... ừ thì vậy đó các bạn ạ",
     "tôi nghĩ. ừ thì vậy đó các bạn ạ"),
    ("Đợi đã‥ chờ chút nha mọi người ơi",
     "đợi đã. chờ chút nha mọi người ơi"),
    # Hai dấu chấm trở lên (kể cả xen khoảng trắng) -> một dấu chấm
    ("a.... b nhé các bạn ơi", "a. bê nhé các bạn ơi"),
    ("a. . . b nhé các bạn ơi", "a. bê nhé các bạn ơi"),
]


@pytest.mark.parametrize("text,expected", ELLIPSIS_CASES)
def test_ellipsis_and_multidot_normalized(normalizer, text, expected):
    assert normalizer.normalize(text) == expected


# ── punc_norm: câu dài (>= 5 từ) ──

def test_long_sentence_appends_dot_when_missing(normalizer):
    out = normalizer.normalize("tôi đi học mỗi ngày vào buổi sáng", punc_norm=True)
    assert out == "tôi đi học mỗi ngày vào buổi sáng."


@pytest.mark.parametrize("ending", [".", "!", "?"])
def test_long_sentence_keeps_existing_terminator(normalizer, ending):
    text = f"hôm nay trời thật là đẹp phải không{ending}"
    assert normalizer.normalize(text, punc_norm=True) == text


def test_long_sentence_trailing_comma_becomes_dot(normalizer):
    # Normalizer luôn cắt dấu phẩy cuối câu (hành vi sẵn có); punc_norm sau đó
    # thấy câu không còn dấu kết thúc nên thêm "." — hợp lý cho TTS.
    out = normalizer.normalize("hôm nay trời thật là đẹp phải không,", punc_norm=True)
    assert out == "hôm nay trời thật là đẹp phải không."


# ── punc_norm: câu siêu ngắn (< 5 từ) luôn ép về "." ──

@pytest.mark.parametrize("text", [
    "xin chào",
    "xin chào!",
    "xin chào?",
    "xin chào.",
    "xin chào !",
    "xin chào…",
])
def test_short_sentence_forced_to_dot(normalizer, text):
    assert normalizer.normalize(text, punc_norm=True) == "xin chào."


# ── punc_norm mặc định tắt: không đổi gì ở phần dấu cuối ──

def test_punc_norm_off_by_default(normalizer):
    assert normalizer.normalize("xin chào") == "xin chào"
    assert normalizer.normalize("tôi đi học mỗi ngày vào buổi sáng") == \
        "tôi đi học mỗi ngày vào buổi sáng"


# ── Batch áp dụng punc_norm cho từng phần tử ──

def test_normalize_batch_punc_norm(normalizer):
    out = normalizer.normalize(
        ["xin chào", "cảm ơn bạn rất nhiều vì đã giúp đỡ tôi"],
        punc_norm=True,
    )
    assert out == ["xin chào.", "cảm ơn bạn rất nhiều vì đã giúp đỡ tôi."]


# ── G2P: punc_norm đẩy dấu "." vào tận chuỗi phoneme ──

def test_g2p_punc_norm_adds_terminal_punct(g2p):
    without = g2p.convert("cảm ơn nhé")
    with_punc = g2p.convert("cảm ơn nhé", punc_norm=True)
    assert not without.endswith(".")
    assert with_punc == without + "."


def test_g2p_batch_punc_norm(g2p):
    out = g2p.convert(["cảm ơn nhé"], punc_norm=True)
    assert out[0].endswith(".")


# ── Pipeline: punc_norm chạy ở bước normalize và đi suốt tới phoneme ──

def test_pipeline_punc_norm(pipeline):
    without = pipeline.run("xin chào")
    with_punc = pipeline.run("xin chào", punc_norm=True)
    assert not without.endswith(".")
    assert with_punc == without + "."
