from .sea_g2p_rs import Normalizer as NormalizerRS

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        self._rs_normalizer = NormalizerRS(lang=lang)
    
    def normalize(self, text: str) -> str:
        """Main normalization pipeline delegated to Rust core."""
        return self._rs_normalizer.normalize(text)
