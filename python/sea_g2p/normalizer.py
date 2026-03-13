import logging
from .sea_g2p_rs import Normalizer as _RustNormalizer

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    Uses a fast Rust core for high performance.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        if lang != "vi":
            logging.getLogger("sea_g2p.Normalizer").warning(
                f"Language '{lang}' is not fully supported for normalization yet. Falling back to 'vi'."
            )
        try:
            self._rust_normalizer = _RustNormalizer()
        except Exception as e:
            logging.getLogger("sea_g2p.Normalizer").error(f"Failed to initialize Rust normalizer: {e}")
            raise
    
    def normalize(self, text: str) -> str:
        """Main normalization pipeline powered by Rust."""
        if not text:
            return ""
        return self._rust_normalizer.normalize(text)
