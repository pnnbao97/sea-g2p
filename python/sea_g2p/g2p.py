import os
import logging
from .sea_g2p_rs import G2P as _RustG2P

logger = logging.getLogger("sea_g2p.G2P")


class G2P:
    """
    Multilingual G2P (Grapheme-to-Phoneme) converter.
    Uses a fast Rust core with binary dictionary lookup for maximum performance.
    """
    def __init__(self, lang: str = "vi", db_path: str = None):
        self.lang = lang
        db_path = os.path.join(os.path.dirname(__file__), "sea_g2p.bin")

        self._rust_engine = _RustG2P(db_path)
        logger.debug(f"Initialized Rust G2P engine with {db_path}")

    def convert(self, text: str | list[str], **kwargs) -> str | list[str]:
        """
        Convert text or a list of texts to phonemes.
        If a list is provided, conversion is done in parallel using Rust's Rayon.
        """
        if isinstance(text, list):
            return self.phonemize_batch(text, **kwargs)
        return self._rust_engine.phonemize(text)

    def phonemize_batch(self, texts: list[str], **kwargs) -> list[str]:
        """Convert a batch of text strings to phonemes."""
        if not texts:
            return []
        return self._rust_engine.phonemize_batch(texts)
