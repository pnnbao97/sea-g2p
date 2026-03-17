from .normalizer import Normalizer
from .g2p import G2P

class SEAPipeline:
    def __init__(self, lang="vi"):
        self.normalizer = Normalizer(lang=lang)
        self.g2p = G2P(lang=lang)
    
    def run(self, text: str | list[str]) -> str | list[str]:
        """
        Run the full text-to-phoneme pipeline: normalization followed by phonemization.
        If a list of strings is provided, processing is done in parallel.
        """
        if not text:
            return "" if isinstance(text, str) else []
        normalized_text = self.normalizer.normalize(text)
        phonemes = self.g2p.convert(normalized_text)
        return phonemes
