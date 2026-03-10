import pytest
from sea_g2p import G2P

@pytest.fixture
def g2p():
    return G2P()

def test_phonemize_vietnamese(g2p):
    text = "Xin chào Việt Nam"
    phonemes = g2p.convert(text)
    assert isinstance(phonemes, str)
    assert len(phonemes) > 0

def test_phonemize_english_tag(g2p):
    text = "Học <en>machine learning</en> rất hay"
    phonemes = g2p.convert(text)
    assert isinstance(phonemes, str)
    assert len(phonemes) > 0
    assert any(c in phonemes for c in "əˈʃ")

def test_phonemize_with_custom_dict(g2p):
    custom_dict = {"robot": "ro-bot-phi-diệu"}
    text = "Tôi là robot"
    phonemes = g2p.convert(text, phoneme_dict=custom_dict)
    assert "ro-bot-phi-diệu" in phonemes

def test_phonemize_batch_consistency(g2p):
    texts = ["Xin chào", "Việt Nam", "Chào <en>world</en>"]
    results = g2p.phonemize_batch(texts)

    assert len(results) == 3
    for res in results:
        assert isinstance(res, str)
        assert len(res) > 0

    assert results[0] == g2p.convert(texts[0])
    assert results[1] == g2p.convert(texts[1])
    assert results[2] == g2p.convert(texts[2])

def test_primes_and_apostrophes(g2p):
    # Note: G2P class expects normalized text. 
    # Normalization of A' -> 'a phẩy' and 1' -> 'một phẩy' is tested in test_normalize.py
    
    # Test Phonemization of the expanded prime forms
    assert "fˈəɪ4" in g2p.convert("a phẩy")
    assert "mˈo6t̪ fˈəɪ4" in g2p.convert("một phẩy")
    
    # Test English apostrophes within words (these are kept by normalizer and handled by G2P)
    res_dont = g2p.convert("don't")
    assert "dˈoʊnt" in res_dont
    
    res_its = g2p.convert("it's")
    assert "ɪts" in res_its
