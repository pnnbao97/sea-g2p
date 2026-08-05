# -*- coding: utf-8 -*-
"""Context classifier for the Indonesian ⟨e⟩: /ə/ or /e/.

Indonesian spells both with one letter and the choice is lexical, so this is
the last resort — used only for words no marked source covers. Trained on
the gold lexicon, it scores 80.1% on held-out data against 66.4% for the
"always schwa" default.

Features are the neighbouring letters plus two positional flags, backing off
from the most specific context to the least. Nothing is predicted from a
context seen fewer than eight times; those fall through to schwa.
"""
from collections import Counter, defaultdict

from id_g2p import g2p_word


class SchwaClassifier:
    def __init__(self, gold):
        self.table = defaultdict(Counter)
        for w, phones in gold.items():
            if not w.isalpha() or "e" not in w:
                continue
            r = g2p_word(w)
            if len(r) != len(phones):
                continue
            labels = [b for a, b in zip(r, phones) if a == "ə"]
            idx = [i for i, c in enumerate(w) if c == "e"]
            if len(labels) != len(idx):
                continue
            for i, lab in zip(idx, labels):
                if lab not in ("ə", "e"):
                    continue
                for key in self._keys(w, i):
                    self.table[key][lab] += 1

    @staticmethod
    def _keys(w, i):
        prev = w[i - 1] if i else "^"
        nxt = w[i + 1] if i + 1 < len(w) else "$"
        early = i <= 2
        last = not any(c in "aiueo" for c in w[i + 1:])
        return [(prev, nxt, early, last), (prev, nxt), ("N", nxt), (prev, "N")]

    def predict(self, w, i):
        for key in self._keys(w, i):
            c = self.table.get(key)
            if c and sum(c.values()) >= 8:
                return c.most_common(1)[0][0]
        return "ə"

    def predict_word(self, w):
        """Indices among the word's ⟨e⟩ letters that are /e/."""
        return {
            n for n, i in enumerate(i for i, c in enumerate(w) if c == "e")
            if self.predict(w, i) == "e"
        }
