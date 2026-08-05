import pytest
from sea_g2p import SEAPipeline

@pytest.fixture
def pipeline():
    return SEAPipeline(lang="vi")

def test_pipeline_primes(pipeline):
    # Tests combined normalization + phonemization
    res_a = pipeline.run("A'")
    assert "fˈəɪ4" in res_a
    
    res_1 = pipeline.run("1'")
    assert "mˈo6t̪ fˈəɪ4" in res_1

def test_pipeline_english_contractions(pipeline):
    res_dont = pipeline.run("I don't care")
    assert "dˈoʊnt" in res_dont
    
    res_its = pipeline.run("It's a gift")
    assert "ɪts" in res_its

def test_pipeline_full_sentence(pipeline):
    text = "Giá SP500 hôm nay là 4.200,5 điểm"
    res = pipeline.run(text)
    
    # Check for presence of key words instead of full exact phonemes
    assert "zˈaː" in res       # Giá
    assert "nˈam" in res       # năm
    assert "tʃˈam" in res      # trăm (Northern pronunciation)
    assert "fˈəɪ4" in res      # phẩy
    assert "ɗˈiɛ" in res       # điểm

def test_pipeline_en_tag_protection(pipeline):
    # Tags should be preserved as English even if mixed with VI
    text = "Học <en>machine learning</en> rất cool"
    res = pipeline.run(text)
    
    # machine learning phonemes (espeak)
    assert "məʃˈiːn" in res or "məkˈiːn" in res
    assert "lˈɜːnɪŋ" in res
