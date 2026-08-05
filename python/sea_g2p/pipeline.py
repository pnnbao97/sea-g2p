from .normalizer import Normalizer
from .g2p import G2P

class SEAPipeline:
    """Normalization followed by phonemization, for ``lang`` in ("vi", "th")."""

    def __init__(self, lang="vi"):
        self.lang = lang
        self.normalizer = Normalizer(lang=lang)
        self.g2p = G2P(lang=lang)
    
    def run(self, text: str | list[str], punc_norm: bool = False) -> str | list[str]:
        """
        Run the full text-to-phoneme pipeline: normalization followed by phonemization.
        If a list of strings is provided, processing is done in parallel.

        If ``punc_norm`` is True, trailing punctuation is normalized during the
        normalization step (sentence ends with a single "."; a short sentence —
        fewer than 5 words — is forced to "."). The phonemes therefore carry the
        normalized terminal punctuation through to the output.
        """
        if not text:
            return "" if isinstance(text, str) else []
        if self.lang == "th":
            # The Thai front end must normalize and segment together: word
            # boundaries only exist after segmentation, and passes such as the
            # ๆ repetition mark depend on them. Normalizing here as well would
            # run the numeric and abbreviation stages twice.
            return self.g2p.convert(text)
        normalized_text = self.normalizer.normalize(text, punc_norm=punc_norm)
        phonemes = self.g2p.convert(normalized_text)
        return phonemes
