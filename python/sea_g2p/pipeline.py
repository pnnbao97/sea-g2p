from .normalizer import Normalizer
from .g2p import G2P

class SEAPipeline:
    def __init__(self, lang="vi"):
        self.normalizer = Normalizer(lang=lang)
        self.g2p = G2P(lang=lang)
    
    def run(self, text: str) -> str:
        """
        Run the full text-to-phoneme pipeline: normalization followed by phonemization.
        Uses a single call to the Rust core for maximum efficiency.
        """
        if not text:
            return ""
        return self.g2p._rust_engine.pipeline(text)
