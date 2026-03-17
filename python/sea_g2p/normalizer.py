from .sea_g2p_rs import Normalizer as NormalizerRS

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        self._rs_normalizer = NormalizerRS(lang=lang)
    
    def normalize(self, text: str | list[str]) -> str | list[str]:
        """
        Normalize text or a list of texts using the Rust core.
        If a list is provided, normalization is done in parallel using Rayon.
        """
        if isinstance(text, list):
            return self._rs_normalizer.normalize_batch(text)
        return self._rs_normalizer.normalize(text)

    def normalize_batch(self, texts: list[str]) -> list[str]:
        """Normalize multiple texts in parallel using Rust's Rayon."""
        return self._rs_normalizer.normalize_batch(texts)
