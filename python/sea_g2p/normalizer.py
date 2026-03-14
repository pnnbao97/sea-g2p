import re
import unicodedata
from .vi_cleaner import clean_vietnamese_text, RE_EXTRA_SPACES

class Normalizer:
    """
    A text normalizer for Vietnamese Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable Vietnamese text.
    """
    
    def __init__(self, lang: str = "vi") -> None:
        self.lang = lang
        if lang != "vi":
            import logging
            logging.getLogger("sea_g2p.Normalizer").warning(f"Language '{lang}' is not fully supported for normalization yet. Falling back to 'vi'.")
    
    def normalize(self, text: str) -> str:
        """Main normalization pipeline with EN tag protection."""
        if not text:
            return ""

        # Pre-normalization: Ensure NFC format for Vietnamese characters
        text = unicodedata.normalize('NFC', text)

        # Step 1: Detect and protect EN tags
        en_contents = []
        placeholder_pattern = "ENTOKEN{}"
        
        def extract_en(match):
            en_contents.append(match.group(0))
            return placeholder_pattern.format(len(en_contents) - 1)
        
        text = re.sub(r'<en>.*?</en>', extract_en, text, flags=re.IGNORECASE)
        
        # Step 2: Core Normalization
        text = clean_vietnamese_text(text)
        
        # Final cleanup
        text = text.lower()
        # Single pass for common whitespace fixes
        text = RE_EXTRA_SPACES.sub(' ', text).strip()
        
        # Step 3: Restore EN tags
        if en_contents:
            for idx, en_content in enumerate(en_contents):
                placeholder = placeholder_pattern.format(idx).lower()
                text = text.replace(placeholder, en_content)
        
        # Final whitespace cleanup - already mostly done, just one more pass to be safe
        if ' ' in text: # Simple check to avoid regex if not needed
             text = RE_EXTRA_SPACES.sub(' ', text).strip()
        
        return text
