# Indonesian G2P: dictionary, rules and how they were chosen

Research deliverable for adding Indonesian to sea-g2p. Every number below was
measured, and the decisions that were tested and rejected are recorded so the
same ground is not covered twice.

## Sources

1. **WikiPron gold** — [ind_latn_broad](https://github.com/CUNY-CL/wikipron),
   18,590 human-transcribed IPA entries.
2. **KBBI v6.1.0** — the official Indonesian dictionary,
   [kbbi-v6-full-csv](https://github.com/aryakdaniswara/kbbi-v6-full-csv),
   194,692 headwords. Its `pelafalan` field marks ⟨ê⟩ for /ə/, which settles
   the one thing Indonesian spelling hides.
3. **Corpus** — Indonesian Wikipedia (`wikimedia/wikipedia`, 20231101.id),
   42.1M tokens, for frequency and coverage.

## Results

- `id_dict.tsv`: **172,557 words**, covering **85.9%** of corpus tokens.
  With the English engine taking the 11.8% of tokens that are English words,
  **97.7% of tokens have a proper handling path**; the rest goes to rules.
- Rules alone score **76.2%** exact against the gold lexicon.

| Source | Entries | Confidence |
|---|---|---|
| WikiPron gold | 17,777 | human-transcribed |
| KBBI `pelafalan` | 15,096 | 91.5% agreement with gold |
| KBBI words with no ⟨e⟩ | 41,006 | no ambiguity to resolve |
| KBBI + schwa classifier | 42,314 | ~80% — the weakest, replace first |
| Affix derivation | 19,244 | 92.1% |
| Compounds | 28,042 | inherited from the parts |
| Reduplication | 9,078 | inherited from the parts |

## The one thing spelling does not say

⟨e⟩ writes both /ə/ and /e/ — 7,449 against 2,018 in the gold lexicon — and
nothing in the spelling distinguishes them. Five approaches were measured:

| Approach | Accuracy |
|---|---|
| Always schwa | 66.4% |
| espeak-ng | 76.5% |
| Context classifier trained on gold | 80.1% |
| **Hand annotation (mine)** | **68.4%** |
| **KBBI `pelafalan`** | **91.5%** |

Two of those are worth explaining.

**espeak-ng was rejected as a source.** It scores the same as our own rules
(76.5% vs 76.2%) and errs in *both* directions on the schwa — 1,702 times
/e/→/ə/ and 1,494 times /ə/→/e/. Generating entries with it would freeze a
coin flip into the data.

**Hand annotation was tried and failed its own validation.** Annotating the
highest-frequency uncovered words by hand scored 68.4% when checked against
gold words sharing a stem — *worse than the automatic classifier*. The errors
were systematic: loanwords were marked /e/ throughout when the non-final
syllables actually reduce (molekuler is /moləkulɛr/, prosedural
/prosədural/). Finding KBBI's `pelafalan` field replaced hundreds of hours of
that work with a three-minute download and a better result.

## Rejected hypotheses, with their numbers

- **"⟨e⟩ in the final syllable is /e/"** — inferred from a handful of
  loanwords, then measured across the lexicon: 49% ə / 51% e. A coin flip.
- **"KBBI not marking a word means schwa"** — 58.7% correct, worse than the
  classifier. KBBI's silence is not evidence.
- **"Loanword shape predicts /e/"** — 56% vs 23%, real but too weak to use
  alone.
- **`nj` → /ɲ/ as a pre-1972 spelling** — cost 0.89 points of gold accuracy.
  In modern spelling it is nearly always a morpheme boundary: menjadi is
  meN- + jadi, /mən.d͡ʒa.di/.

## Rules

Indonesian orthography is otherwise regular:

- digraphs `ng` `ngg` `ny` `sy` `kh`, and `c` = /t͡ʃ/, `j` = /d͡ʒ/, `y` = /j/;
- ⟨ai⟩ and ⟨au⟩ are diphthongs **only at the end of a word**: 154 glide vs 39
  hiatus word-finally, but 124 hiatus vs 53 glide before a consonant, because
  those are two syllables (ma-in, ka-in). Applying the diphthong everywhere
  read "main" as /majn/ and cost 1.8 points;
- **pre-1972 spellings** are read, because names kept them: `dj`→/d͡ʒ/,
  `tj`→/t͡ʃ/, `sj`→/ʃ/, `ch`→/x/, `oe`→/u/. Soeharto, Gadjah Mada, Achmad.
  Proper names are 77% of what the dictionary does not cover.

Tense/lax pairs (i/ɪ, u/ʊ, e/ɛ, o/ɔ) from the gold lexicon are folded to the
phonemic six-vowel system /a i u e o ə/. In the unambiguous closed-final
context only 16-27% of /i u o/ are transcribed lax, and 21 words carry two
records differing in nothing else (Kertosono is both kərtosono and
kərtɔsɔnɔ) — the distinction is transcriber noise, not signal.

## Morphology

Indonesian is agglutinative, so the gold lexicon's 17.8k roots cover only 63%
of tokens on their own. Affixed forms are derived rather than guessed: every
affix has a fixed pronunciation, so the only unknown — the root's schwa
pattern — comes from the dictionary.

The hard part is meN-/peN-, which assimilate to the root's first consonant
*and delete it*: meN- + sapu -> menyapu. Recovering the root means restoring
a consonant that is not written, and the restored consonant must then be
dropped from the pronunciation — keeping it produced pəm-pərintah for
pemerintah instead of pə-mərintah.

Which endings are real morphemes was determined from data, not memory: scan
every 2-6 letter ending over uncovered words and count how many *distinct*
known roots it attaches to. `-ku` (151 roots) and `-mu` (86) stood far above
the noise, which is how the missing possessive clitics were found. The same
scan surfaced the productive pieces of Indonesian toponyms — `ci-` (176
roots), `-sari` (68), `kali-` (51), `-rejo` (44), `karang-` (38) — which is
why compound splitting is in the build at all.

## Files

| file | content |
|---|---|
| `id_dict.tsv` | word `\t` space-separated phonemes, frequency-ordered |
| `id_g2p.py` | rules + gold-lexicon normalizer (reference for `src/lang/id/rules.rs`) |
| `id_morph.py` | affix analysis |
| `id_schwa_clf.py` | context classifier for ⟨e⟩ |
| `build_id_dict.py` | the build pipeline |
| `id_freq.tsv.gz` | corpus frequency table |

## Output format

Phonemes are grouped one syllable per space — `saya makan` is `sa ja ma kan`,
not nine separate phonemes — matching what the Vietnamese and Thai front ends
emit. The dictionary stores one phoneme per token, as WikiPron does, and
`src/lang/id/syllable.rs` groups them at read time.

## Known limitations

- The 42k classifier-derived entries are the least certain part of the
  dictionary. A source that marks ⟨é⟩ more widely than KBBI would replace
  them wholesale; that is the single highest-value contribution.
- Javanese names follow Javanese phonology, not Indonesian: Pakubuwana is
  /pakubuwɔnɔ/ to a Javanese speaker, /pakubuwana/ by these rules.
- Final ⟨k⟩ is /k/ or /ʔ/ by lexical choice (887 vs 250 in gold); /k/ is the
  default and the dictionary carries the rest.
