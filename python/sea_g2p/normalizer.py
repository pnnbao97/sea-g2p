import os
from .sea_g2p_rs import Normalizer as NormalizerRS

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        # Phoneme dictionary, used to look words up when reading paths, URLs and
        # emails inside Vietnamese sentences.
        db_path = os.path.join(os.path.dirname(__file__), "sea_g2p.bin")
        self._rs_normalizer = NormalizerRS(lang=lang, dict_path=db_path)
    
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

    def audit(self, text: str) -> list[str]:
        """Report characters that normalization would drop without speaking them.

        Normalization ends by stripping every character it does not recognise.
        That keeps output clean, but a symbol whose reading was never declared
        disappears silently — the result still sounds fluent, so the loss is
        inaudible. ``10⁻³`` once came out as "mười lập phương", six orders of
        magnitude off, because the superscript minus was simply deleted.

        Returns the offending characters, de-duplicated and in order of first
        appearance; an empty list means the input is fully covered. Use it in
        tests over new corpora before shipping them.
        """
        return self._rs_normalizer.audit(text)
