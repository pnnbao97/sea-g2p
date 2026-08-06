# Thai G2P groundwork: phoneme dictionary + word segmentation

Research deliverable for adding Thai to sea-g2p. Everything here was built
and cross-validated from three independent sources:

1. **Gold lexicon** — [thai-g2p-wiktionary-corpus](https://github.com/PyThaiNLP/thai-g2p-wiktionary-corpus)
   (16,028 human-curated Wiktionary IPA entries, scraped by WikiPron).
2. **tltk** (Thai Language Toolkit, Chulalongkorn University) — rule+dictionary
   G2P used to cover corpus words absent from the gold lexicon.
3. **Corpus** — [pythainlp/thai-wiki-dataset-v4](https://huggingface.co/datasets/pythainlp/thai-wiki-dataset-v4)
   (158,952 articles, 61.0M word tokens after `newmm` segmentation),
   used for frequency ranking and coverage measurement.

Why not espeak-ng: tested empirically, its Thai voice reads character-by-character
with no word segmentation, no preposed-vowel reordering (เขา → "e-kha"), no
silent-ห handling and broken tones. Unusable even as a fallback.

## What is settled, and what is not

Settled, with the measurement that settles it:

| | Status |
|---|---|
| **Word segmentation** | **No longer the bottleneck.** F1 0.906 against human annotation, and — the number that matters — **99.50% of output syllables are identical to what perfect segmentation would produce.** All remaining boundary error costs half a percent. |
| **Silent deletion** | Zero. Every one of the 91,865 dictionary keys produces phonemes; no raw Thai character survives into the output. |
| **The audit** | Reports exactly what the pipeline deletes. It used to fire on every ordinary sentence (tone marks are combining marks, not letters) while staying blind to whole scripts it was dropping. |
| **Dictionary coverage** | 99.4–100% of tokens on edited text, across 25 datasets. |

Not settled:

| | Status |
|---|---|
| **Pronunciation accuracy** | **Not measured, and not measurable from inside.** It needs gold phonemes for running text, which do not exist for Thai. Composing the layers gives ≈97–98% at syllable level, but that is an estimate assembled from parts, not a measurement — do not quote it as one. |
| **17.3% of tokens** | Read from tltk, a tool, not a human. QA'd by an independent tone checker at 97.3%, which is not the same as verified. |
| **Colloquial text** | Dictionary coverage falls to 94.0% on social media, and the segmenter drops to F1 0.877 there against newmm's 0.917. The misses are chat spellings and loanwords a Wikipedia-built lexicon never saw. |
| **Whether it sounds right** | **No native speaker has heard any output.** No measurement in this document addresses this, and no further measurement will. |

A note on why segmentation stopped mattering. It was the headline concern
for Thai — no spaces, so a wrong cut looks up a different word entirely.
That intuition is right about the mechanism and wrong about the scale:
87% of the places this segmenter disagrees with human annotation produce
**identical phonemes**, because Thai compounds are read as the sum of
their parts. `ควรจะเป็น` and `ควร|จะ|เป็น` sound the same. Effort spent
pushing 0.906 toward a neural model's 0.97 would buy roughly one percent
of pronunciation, at two orders of magnitude in speed.

## Measuring segmentation honestly

The segmenter was benchmarked against PyThaiNLP `newmm` and scored F1 0.987.
That number was not what it looked like. `build/freq_count.py` counts the
corpus **with newmm**, so newmm's segmentation is what the cost model learns
— and newmm was also the judge. A metric cannot penalise a system for
inheriting the mistakes of the thing it is being compared to.

Against human annotation the picture is different, and more useful:

| | F1 vs newmm | F1 vs BEST2009 (human) |
|---|---|---|
| frequencies counted with newmm | 0.987 | 0.870 |
| **frequencies counted from human annotation** | — | **0.906** |
| newmm itself | — | 0.868 |

The diagnosis came from the shape of the errors: precision 0.95 but recall
0.80, meaning the segmenter systematically **under-split**. 99.8% of the
boundaries it missed fell inside compounds whose parts were already in the
dictionary — ความสัมพันธ์ for ความ|สัมพันธ์, ทางกฎหมาย for ทาง|กฎหมาย.
Counting with newmm had made each compound look like one frequent word, so
the dynamic program never had reason to split it.

Recounting frequencies from human word boundaries fixes exactly that:
recall 0.801 → 0.864, precision unchanged. Cross-checked on
VISTEC-TP-TH-2021, a corpus from neither source and a different register:
0.865 → 0.877.

**Absence is informative.** A word the annotators never emit is one they
split, so it gets frequency 1. Blending the old newmm counts back in for
those words measured worse (0.9177 at 2% weight, 0.8986 at 10%).

### Which corpus, and why not the obvious one

BEST2009 is the standard benchmark and scores slightly higher — but it is
**CC BY-NC-SA 3.0**, and a frequency table derived from it would carry the
NonCommercial term into `sea_g2p.bin`, making an Apache-2.0 library
unusable commercially. It is used here for **evaluation only**, which
distributes nothing.

The shipped table comes from **Blackboard Treebank** (NECTEC),
[bitbucket.org/kaamanita/blackboard-treebank](https://bitbucket.org/kaamanita/blackboard-treebank/),
**CC BY 3.0** — attribution only, no NonCommercial, no ShareAlike. On
BEST2009 it scores 0.906 against BEST-derived frequencies' 0.925, but that
gap is test-set bias: on the neutral VISTEC corpus the two are level
(0.8767 vs 0.8776). It is `thai/blackboard_freq.tsv.gz`, 23,302 word types
over 858,531 tokens.

Note also that BEST's *test* split carries no gold segmentation — 0.5% of
characters marked word-initial — because it was a shared-task set scored by
submission. Evaluation here uses held-out records from the train split.

## Results at a glance

- `thai_dict.tsv`: **91,865 words** (every corpus word type down to frequency
  2 that passes the QA gates).
- `src/lang/th/rules.rs`: rule-based G2P reading **any** Thai string, so the
  pipeline never gives up on a word. Validated on the gold lexicon: **84%**
  of syllables and **77%** of short words exactly right.
- Segmenter vs **human annotation** (BEST2009, 4,800 records across its four
  genres): boundary **F1 0.906**. PyThaiNLP `newmm` scores 0.868 on the same
  sample. See *Measuring segmentation honestly* below — an earlier figure of
  0.987 measured agreement with newmm, which is not the same thing.
- Out-of-domain check on [pythainlp/thaisum](https://huggingface.co/datasets/pythainlp/thaisum)
  news (3,000 random articles): token coverage **99.92%**, unknown-character
  rate 0.047% — the wiki-built dictionary generalizes to the news register;
  residual OOV is proper-name fragments.
- Messy-web check on [aisingapore/WangchanLION-Web](https://huggingface.co/datasets/aisingapore/WangchanLION-Web)
  (3,395 documents, 10M Thai chars): token coverage **99.78%**,
  unknown-character rate 0.125%. This register surfaced the mark-order
  typos and orphan-mark rules now in `normalize_thai`; remaining OOV is
  Pali phinthu ฺ, truncated fragments, ฿ (normalizer symbol), and a small
  set of genuine colloquial words (งี้ นู่น-style contractions) worth a
  curated addition later.
- Full sweep over the 25-dataset
  [pythainlp Thai-LLM collection](https://huggingface.co/collections/pythainlp/datasets-for-pretrained-thai-llm)
  (streaming samples, `coverage_sweep.tsv`): 23/25 datasets at 99.4–100%
  token coverage (legal/government text ≈100%); pre-modern books are the
  honest floor (99.4–99.5%, archaic spellings like ใม่/เฃา and Pali ฺ).
  The one outlier, WangchanLION-Curated (96.7%), traces to corrupted
  PDF-extraction docs (glyph-mapping garbage, U+FFFD) — 8/471 docs ≥1%
  unknown, one at 39%. Corollary: the per-document unknown-char rate is an
  effective corpus-quality filter for future TTS data curation.
- Independent tone-rule checker agrees with the gold lexicon on **97.3%**
  of judgeable monosyllables; every inspected disagreement is a loanword
  or an irregular function word (details below).

## Files

| file | content |
|---|---|
| `thai_dict.tsv` | word `\t` space-separated canonical syllables, frequency-ordered |
| `thai_phon.py` | canonical inventory + parsers from both source notations |
| `tone_rules.py` | independent tone-rule checker (QA gate) |
| `segment.py` | TCC + maximal-matching word segmenter (Python reference; the shipping implementation is `src/lang/th/segment.rs`) |
| `rule_g2p.py` | rule-based G2P (Python reference for `src/lang/th/rules.rs`) |
| `blackboard_freq.tsv.gz` | word frequencies from human annotation (Blackboard Treebank, CC BY 3.0) — the segmenter's cost model |
| `word_freq.tsv.gz` | full corpus frequency table (143,676 types) |
| `build_report.txt` | build statistics |
| `build_rejects.tsv` | 332 words rejected by QA and why |
| `build_tone_flags.tsv` | 81 tone-rule disagreements kept for review (loanword suffixes) |
| `build/` | reproducible pipeline: `freq_count.py` → `tltk_vs_gold.py` / `tltk_oov.py` → `agreement.py` → `build_dict.py` (run in a work dir containing the downloaded sources) |

## Canonical phoneme scheme

One syllable = `onset + vowel + coda + tone` concatenated, e.g. ครับ →
`kʰrap̚˦˥`; a word is a space-separated syllable list: สวัสดี →
`sa˨˩ wat̚˨˩ diː˧`.

- **Onsets** (21 + clusters): `p pʰ b t tʰ d k kʰ ʔ tɕ tɕʰ f s h m n ŋ r l w j`,
  clusters = C + `r/l/w` (`kr kl kw kʰr kʰl kʰw pr pl pʰr pʰl tr` + loanword
  clusters like `bl dr fr`).
- **Vowels** (18 + 3): short/long pairs `a aː i iː ɯ ɯː u uː e eː ɛ ɛː o oː
  ɔ ɔː ɤ ɤː` and diphthongs `ia ɯa ua` (long for tone purposes).
- **Codas**: `p̚ t̚ k̚` (unreleased stops), `m n ŋ j w`, `ʔ` (word-final short
  open syllable), loanword codas `f s l tɕʰ` (+ optional cluster `s`:
  มินสก์ → `mins˦˥`).
- **Tones** (Chao letters, deliberately distinct from the Vietnamese digit
  convention so the two systems can never be confused in a shared superset):
  `˧` mid, `˨˩` low, `˥˩` falling, `˦˥` high, `˩˩˦` rising.

Segmental symbols overlap the existing VI/EN inventory by design (shared
superset); Thai-only additions are the aspirated stops `pʰ tʰ kʰ`, affricates
`tɕ tɕʰ`, unreleased finals, and the three diphthongs.

## Orthographic rules encoded in the tools

### Tone rules (`tone_rules.py`)

Tone is a function of (consonant class × syllable liveness × tone mark):

- classes: mid ก จ ฎ ฏ ด ต บ ป อ · high ข ฃ ฉ ฐ ถ ผ ฝ ศ ษ ส ห · low = rest
- live = sonorant coda or long open vowel; dead = stop coda or short open
- no mark: mid+live→mid, mid+dead→low, high+live→rising, high+dead→low,
  low+live→mid, low+dead+short→high, low+dead+long→falling
- ่ mai ek: low→falling, else→low · ้ mai tho: low→high, else→falling
- ๊ mai tri: mid→high · ๋ mai chattawa: mid→rising
- leading silent ห + sonorant → high class; อ + ย (อย่า อยู่ อย่าง อยาก) → mid
- cluster class follows its first consonant; pseudo-clusters ทร→/s/, ศร/สร→/s/

Validation: on gold monosyllables the checker agrees 3,318 ok vs 93 mismatches
(97.3%); every inspected mismatch is a loanword (กราฟ การ์ด บล็อก — English
loans conventionally take high tone on dead syllables regardless of spelling)
or an irregular function word (ก็ ค่ะ ฉัน). Native-vocabulary disagreements
from any future source should be treated as bugs.

### Segmentation (`segment.py`)

Thai script has no spaces; segmentation is dictionary-driven:

1. **Unicode normalization** — real-world quirks the Rust normalizer stage
   must fold before anything else:
   - decomposed sara am ํ + า → ำ (ทํา → ทำ), including the variant with a
     tone mark wedged between: ํ + ่้๊๋ + า → tone + ำ (นํ้า → น้ำ);
   - แ typed as two เ characters: เเ → แ (เเละ → และ; found ~100× in a
     3k-article news sample);
   - mark-order typos (all impossible in valid orthography, safe to swap):
     tone after า/ำ → before (นำ้ → น้ำ, ยา่ → ย่า); tone before an
     above/below vowel → after (ท่ี → ที่);
   - orphan dependent marks at the start of a Thai run (base consonant was
     stripped along with markup/emoji) → delete.
2. **TCC pass** — group text into Thai Character Clusters that no correct
   boundary can split: `[preposed vowel เแโใไ]* consonant [dependent mark]*`
   where dependent marks are `ะ ั า ำ ิ ี ึ ื ุ ู ็ ่ ้ ๊ ๋ ์ ํ ๎`.
3. **DP over cluster boundaries** with a trie of dictionary words, minimising
   the total **unigram cost** `-ln P(word)` from corpus frequencies, with
   unknown runs charged per character.

   Frequency weighting matters for correctness, not just for scores: under a
   plain "fewest pieces" objective สากลคน ties between สากล|คน (correct) and
   สาก|ลคน (wrong) — both are two words — and the tie-break picked wrong.
   Weighted by how often each word actually occurs, it is not close. The
   frequencies ship in the binary as `SECTION_TH_FREQ`, built from the same
   wordlist as the pronunciations so the two cannot drift.
4. Non-Thai runs (Latin, digits, punctuation) are natural boundaries and
   pass through untouched — this is where EN code-switching hooks in.
5. `ๆ` (repeat previous word), `ฯ` (abbreviation), `ฯลฯ` (et cetera) are
   split off as single tokens; expanding them is the *normalizer's* job,
   exactly like the Vietnamese abbreviation stage.

## Dictionary build policy (`build/build_dict.py`)

- Gold beats tltk everywhere they overlap. Measured on the 14,790 shared
  words, tltk agrees exactly on 81.9%; disagreements cluster in vowel length
  inside closed syllables, loanword tones, and glottal-stop notation — gold
  is right in the inspected cases.
- Gold variant conflicts (881 words: homograph spelling-readings,
  linking-syllable variants) are resolved by: agreement with tltk →
  majority → first listed. This is what keeps กบ = `kop̚˨˩` (the word) and
  not `kɔː˧ bɔː˧` (its letter-name spelling reading).
- tltk entries are normalised to the gold convention (final ʔ on word-final
  short open syllables) and pass two QA gates:
  - **truncation check**: tltk's own Thai-side syllabification must
    reproduce the input string exactly (catches its dropped-syllable bug);
  - **inventory validation**: every syllable must parse as
    onset·vowel·coda·tone from the canonical sets above.
- Rejected words land in `build_rejects.tsv` — they are *absent* from the
  dict on purpose: at runtime the segmenter should split such strings
  further or fall back, never trust a broken pronunciation.

## Runtime (Rust)

The shipping pipeline lives in `src/lang/th/`:

    raw text -> normalize (8 stages) -> segment (TCC + DP over dict trie)
             -> per token: dictionary section (SECTION_TH) or rules.rs

Latin runs inside Thai text are handed to the same engine that serves English
elsewhere (`Thai::phonemize_with` takes the reader as a callback, so this
module keeps no dependency on `g2p`). Thai code-switches with English
constantly, and without this brand names and tech terms reached the output as
raw letters: ผมใช้ iPhone และ Facebook now reads
`pʰom˩˩˦ tɕʰaj˦˥ ˈaɪfoʊn lɛʔ˦˥ fˈeɪsbʊk`.

`normalizer.rs` follows the Vietnamese staged-pipeline design, including its
stage-order contract table and its silent-deletion audit. Thai-specific
stages: Thai digits ๐-๙ folded to ASCII, `ๆ` repeating the previous word,
abbreviations expanded from `TH_ABBREV` (an instance of the shared
`core::abbrev::AbbrevTable`), Buddhist-era dates, clock times, currency and
percentages, and Thai number-to-words (`num2th.rs`) with the สิบ / ยี่สิบ /
เอ็ด alternations and six-digit ล้าน grouping.

Two ordering facts learned the hard way, both now encoded in the stage table:
abbreviations must run **before** `ๆ` repetition (ฯลฯ expands to "และอื่น ๆ",
whose ๆ itself needs repeating), and the clock-time pattern must try นาฬิกา
before น. or it eats only the leading น and strays าฬิกา into the output.

Known gaps, measured rather than assumed: numeric ranges (10-20) lose their
"ถึง", fractions read as "หนึ่งทับสอง" rather than "ครึ่ง", and URLs keep
their dots as pauses because no Thai URL pass exists yet.

Deliberately unmapped: ม. and อ., whose two readings each (มหาวิทยาลัย /
หมู่ / มัธยม, อำเภอ / อาจารย์) are both common — the same call the Vietnamese
table makes for weekday abbreviations.

Exposed to Python as `G2P.segment_th(text)` and `G2P.phonemize_th(text)`.
The segmenter's wordlist is built from the dictionary section itself, so the
two can never drift apart. Measured at ~2.7M chars/s.

The Rust segmenter was diffed against the Python reference on wiki text and
agrees token for token.

## Shared machinery

Two classes of input are handled by modules in `src/core/`, one implementation
with a per-language word table, so a fix reaches every language at once:

- **`core::numeric`** — minus signs, ranges, powers, superscripts, subscripts,
  multiplication and fractions. Each of these was a *silent deletion* before:
  `-5 องศา` lost its sign and `5 m²` its exponent, with no audible cue.
- **`core::spans`** — emails and URLs, read before any stage can voice the
  punctuation inside them. `https://www.google.com` was coming out as
  "https, ทับ ทับ www.google.com"; it now reads scheme and all, since text
  that says "https://" means it and dropping the scheme is the same silent
  deletion in a different costume.

The silent-deletion audit was itself broken until this: it declared `-`
"intentionally dropped", which hid exactly the loss it exists to catch. It
now runs the numeric stage and checks whether a numeric hyphen survives,
so it stays correct as that stage grows.

## Known limitations / next steps

- **Homographs** need context to disambiguate; the dict stores the citation
  reading chosen by the conflict policy. A future pass can add POS/context
  rules like the VI normalizer's cue-word approach.
- **Linking syllables** in long Sanskrit/Pali compounds are correct only
  where the compound itself is a dict entry; tltk occasionally drops an
  ambisyllabic coda in very long compounds (อสังหาริมทรัพย์ → `ri˦˥` for
  /rim˧/). Frequency-ranked review of `build_tone_flags.tsv` and long-word
  entries is the cheap way to polish.
- Extending 44k -> 92k moved the unknown-character rate only from 0.80% to
  0.75%, but that metric understates the point: the win is *pronunciation
  quality*, not coverage. Head-to-head on gold words, dictionary entries beat
  the rule fallback 87.0% vs 75.8% (word-exact), so every added entry replaces
  a worse reading.
- Adding the ~50k frequency-1 word types was measured and rejected: boundary
  F1 moved 0.9830 -> 0.9826 (slightly worse — the tail is corrupted fragments
  the segmenter would then treat as words), and each such word occurs once in
  61M tokens. Coverage is done; quality now comes from the cost model.
- A rejected gate, recorded so it is not re-invented: requiring the rule
  engine to agree with tltk on syllable count looked like sensible QA but is
  backwards. On the 10.7k words where they disagree, tltk is 72.7% word-exact
  while the rules are 6.2% — the disagreements are the rules failing on long
  compounds, so the gate was letting the weaker source veto the stronger one.
  Only the truncation and inventory gates survive.
- Further extension is not worth it — the tail is corrupted fragments, not
  vocabulary (chasing 99.99% would need ~137k types, most of them hapax junk).
- `ๆ ฯ ฯลฯ`, numbers, dates, Latin tokens → Thai normalizer stage, to be
  designed after the Vietnamese abbreviation-table refactor so both share
  the per-language table structure.
