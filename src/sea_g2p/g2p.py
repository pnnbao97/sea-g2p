import os
import logging
from typing import List, Dict, Optional

logger = logging.getLogger("sea_g2p.G2P")

try:
    from .sea_g2p_rs import G2P as _RustG2P
    _RUST_AVAILABLE = True
except ImportError:
    _RUST_AVAILABLE = False

class G2P:
    """
    Multilingual G2P (Grapheme-to-Phoneme) converter.
    Uses a fast Rust core with binary dictionary lookup for maximum performance.
    """
    def __init__(self, lang: str = "vi", db_path: str = None):
        self.lang = lang
        
        if not _RUST_AVAILABLE:
            raise RuntimeError(
                "Rust extension (sea_g2p_rs) not found. "
                "Please install the package correctly or rebuild the extension."
            )
            
        # Try to find the binary dictionary
        if db_path is None:
            bin_path = os.path.join(os.path.dirname(__file__), "phone_dict", "phone_dict.bin")
            if os.path.exists(bin_path):
                db_path = bin_path
            else:
                # Fallback lookup in common project locations
                search_paths = [
                    os.path.join(os.getcwd(), "src", "sea_g2p", "phone_dict", "phone_dict.bin"),
                    os.path.join(os.path.dirname(os.path.dirname(__file__)), "phone_dict", "phone_dict.bin")
                ]
                for p in search_paths:
                    if os.path.exists(p):
                        db_path = p
                        break
        
        if not db_path or not os.path.exists(db_path):
            raise FileNotFoundError(
                f"Phoneme dictionary not found. Expected a .bin file. "
                f"Searched in: {db_path or 'default paths'}"
            )

        try:
            self._rust_engine = _RustG2P(db_path)
            logger.debug(f"Initialized Rust G2P engine with {db_path}")
        except Exception as e:
            logger.error(f"Failed to initialize Rust G2P engine: {e}")
            raise

    def convert(self, text: str, **kwargs) -> str:
        """Convert a single text string to phonemes."""
        return self._rust_engine.phonemize(text)

    def phonemize_batch(self, texts: List[str], **kwargs) -> List[str]:
        """Convert a batch of text strings to phonemes."""
        if not texts:
            return []
        return self._rust_engine.phonemize_batch(texts)
