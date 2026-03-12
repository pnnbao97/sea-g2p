from .sea_g2p_rs import Normalizer as NormalizerRS

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    Wrapped around a high-performance Rust implementation.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.inner = NormalizerRS(lang)
    
    def normalize(self, text: str) -> str:
        """Main normalization pipeline."""
        return self.inner.normalize(text)
