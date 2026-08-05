import os
import logging
from .sea_g2p_rs import G2P as _RustG2P

logger = logging.getLogger("sea_g2p.G2P")


SUPPORTED_LANGS = ("vi", "th", "id")


class G2P:
    """
    Multilingual G2P (Grapheme-to-Phoneme) converter.
    Uses a fast Rust core with binary dictionary lookup for maximum performance.

    ``lang`` selects the front end:

    - ``"vi"`` — Vietnamese with English code-switching, the default.
    - ``"th"`` — Thai. Because Thai is written without spaces, this path
      normalizes and segments the text before looking words up, and reads
      Latin runs with the same English engine, so mixed Thai/English input
      comes out as one phoneme string.
    - ``"id"`` — Indonesian. Latin script with spaces, so no segmentation;
      words not in the Indonesian dictionary go to the English engine when
      it knows them (code-switching) and to the Indonesian rules otherwise,
      which is what carries proper names.
    """
    def __init__(self, lang: str = "vi", db_path: str = None):
        if lang not in SUPPORTED_LANGS:
            raise ValueError(f"lang must be one of {SUPPORTED_LANGS}, got {lang!r}")
        self.lang = lang
        db_path = os.path.join(os.path.dirname(__file__), "sea_g2p.bin")

        self._rust_engine = _RustG2P(db_path)
        logger.debug(f"Initialized Rust G2P engine with {db_path}")

    def convert(self, text: str | list[str], punc_norm: bool = False, **kwargs) -> str | list[str]:
        """
        Convert text or a list of texts to phonemes.
        If a list is provided, conversion is done in parallel using Rust's Rayon.

        If ``punc_norm`` is True, the input text's trailing punctuation is
        normalized before phonemization (a sentence ends with a single "."; a
        short sentence — fewer than 5 words — is forced to ".").
        """
        if isinstance(text, list):
            return self.phonemize_batch(text, punc_norm=punc_norm, **kwargs)
        if self.lang == "th":
            return self._rust_engine.phonemize_th(text)
        if self.lang == "id":
            return self._rust_engine.phonemize_id(text)
        return self._rust_engine.phonemize(text, punc_norm)

    def phonemize_batch(self, texts: list[str], punc_norm: bool = False, **kwargs) -> list[str]:
        """Convert a batch of text strings to phonemes."""
        if not texts:
            return []
        if self.lang == "th":
            return self._rust_engine.phonemize_th_batch(texts)
        if self.lang == "id":
            return self._rust_engine.phonemize_id_batch(texts)
        return self._rust_engine.phonemize_batch(texts, punc_norm)
