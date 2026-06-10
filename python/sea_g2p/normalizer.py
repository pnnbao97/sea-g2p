from .sea_g2p_rs import Normalizer as NormalizerRS

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        self._rs_normalizer = NormalizerRS(lang=lang)
    
    def normalize(self, text: str | list[str], punc_norm: bool = False) -> str | list[str]:
        """
        Normalize text or a list of texts using the Rust core.
        If a list is provided, normalization is done in parallel using Rayon.

        If ``punc_norm`` is True, the trailing punctuation is normalized after
        normalization: a sentence is forced to end with a single ".". A short
        sentence (fewer than 5 words) always ends with ".", replacing any
        existing trailing punctuation; a longer sentence only gets a "."
        appended when it does not already end with one of , . ! ?
        """
        if isinstance(text, list):
            return self._rs_normalizer.normalize_batch(text, punc_norm)
        return self._rs_normalizer.normalize(text, punc_norm)

    def normalize_batch(self, texts: list[str], punc_norm: bool = False) -> list[str]:
        """Normalize multiple texts in parallel using Rust's Rayon."""
        return self._rs_normalizer.normalize_batch(texts, punc_norm)
