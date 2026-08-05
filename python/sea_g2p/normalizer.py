import os
from .sea_g2p_rs import Normalizer as NormalizerRS
from .sea_g2p_rs import G2P as _RustG2P

SUPPORTED_LANGS = ("vi", "th", "id")


class Normalizer:
    """
    A text normalizer for South East Asian Text-to-Speech systems.
    Converts numbers, dates, units, and special characters into readable words.

    ``lang`` selects the pipeline: ``"vi"`` for Vietnamese (17 stages, with
    English code-switching), ``"th"`` for Thai (8 stages, Thai digits,
    Buddhist-era dates), ``"id"`` for Indonesian (6 stages, rupiah, the
    period-for-thousands convention, chat contractions).
    """

    def __init__(self, lang: str = "vi") -> None:
        if lang not in SUPPORTED_LANGS:
            raise ValueError(f"lang must be one of {SUPPORTED_LANGS}, got {lang!r}")
        self.lang = lang
        # Phoneme dictionary, used to look words up when reading paths, URLs and
        # emails inside Vietnamese sentences.
        db_path = os.path.join(os.path.dirname(__file__), "sea_g2p.bin")
        self._rs_normalizer = NormalizerRS(lang=lang, dict_path=db_path)
        # The Thai normalizer lives on the G2P object, which owns the
        # dictionary section its abbreviation and segmentation passes need.
        # Thai and Indonesian normalization lives on the G2P object, which
        # owns the dictionary sections those pipelines need.
        self._rs_lang = _RustG2P(db_path) if lang in ("th", "id") else None
    
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
            return self.normalize_batch(text, punc_norm=punc_norm)
        if self.lang == "th":
            return self._rs_lang.normalize_th(text)
        if self.lang == "id":
            return self._rs_lang.normalize_id(text)
        return self._rs_normalizer.normalize(text, punc_norm)

    def normalize_batch(self, texts: list[str], punc_norm: bool = False) -> list[str]:
        """Normalize multiple texts in parallel using Rust's Rayon."""
        if not texts:
            return []
        if self.lang == "th":
            return self._rs_lang.normalize_th_batch(texts)
        if self.lang == "id":
            return self._rs_lang.normalize_id_batch(texts)
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

        The audit follows ``lang``. It used to always run the Vietnamese one,
        which meant a symbol missing from the Thai or Indonesian tables was
        checked against Vietnamese rules and cleared — the guard staying
        silent in exactly the two pipelines that most needed it.
        """
        if self.lang == "th":
            return self._rs_lang.audit_th(text)
        if self.lang == "id":
            return self._rs_lang.audit_id(text)
        return self._rs_normalizer.audit(text)
