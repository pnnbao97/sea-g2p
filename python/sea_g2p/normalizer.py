import logging

logger = logging.getLogger("sea_g2p.Normalizer")

try:
    from .sea_g2p_rs import Normalizer as _RustNormalizer
    _RUST_AVAILABLE = True
except ImportError:
    _RUST_AVAILABLE = False

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    Uses a fast Rust core for maximum performance.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        if not _RUST_AVAILABLE:
            raise RuntimeError(
                "Rust extension (sea_g2p_rs) not found. "
                "Please install the package correctly or rebuild the extension."
            )

        if lang != "vi":
            logger.warning(f"Language '{lang}' is not fully supported for normalization yet. Falling back to 'vi'.")

        try:
            self._rust_normalizer = _RustNormalizer(lang=lang)
        except Exception as e:
            logger.error(f"Failed to initialize Rust Normalizer: {e}")
            raise

    def normalize(self, text: str) -> str:
        """Main normalization pipeline."""
        if not text:
            return ""
        return self._rust_normalizer.normalize(text)
