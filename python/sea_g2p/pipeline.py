from .normalizer import Normalizer
from .g2p import G2P

class SEAPipeline:
    def __init__(self, lang="vi"):
        self.normalizer = Normalizer(lang=lang)
        self.g2p = G2P(lang=lang)
    
    def run(self, text: str) -> str:
        """
        Run the full text-to-phoneme pipeline: normalization followed by phonemization.
        This call is optimized to run entirely in Rust to reduce cross-language overhead.
        """
        if not text:
            return ""
        # The G2P class already uses the Rust engine, and we've added a unified run_pipeline method
        return self.g2p._rust_engine.run_pipeline(text)
