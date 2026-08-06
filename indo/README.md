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
| KBBI + schwa classifier | 42,314 | ~80% — the weakest |
| Affix derivation | 19,244 | 92.1% |
| Compounds | 28,042 | inherited from the parts |
| Reduplication | 9,078 | inherited from the parts |

## What is settled, and what is not

Settled:

| | Status |
|---|---|
| **Segmentation** | **Not a problem at all.** Indonesian is written with spaces. What Thai spends most of its machinery on does not exist here. |
| **Orthography** | Regular, and the rules cover it: digraphs, pre-1972 spellings, ⟨ai⟩/⟨au⟩. The one genuine ambiguity is ⟨e⟩, below. |
| **Silent deletion** | Zero across all 172,557 entries — no empty pronunciations, no orthography surviving into the output. A diacritic used to take its base letter with it (`ārati` read as "rati") while the audit approved; both are fixed. |
| **Morphology** | Affixed forms are derived from roots rather than guessed, at 97.3% on the schwa decision — measured by deriving gold words from other gold roots. |

Not settled:

| | Status |
|---|---|
| **⟨e⟩ = /ə/ or /e/** | The whole problem, and it is lexical: spelling carries no signal, so it needs an oracle rather than a rule. 70.2% of running text is covered by human transcription, 9.4% has no ⟨e⟩ to get wrong, 7.7% is derivable from a gold root — and **6.6% is verified by nothing**. |
| **51,320 entries** | Unverified. They are rare, but `schwa_review.tsv` ships the 2,000 that carry weight; the first 500 rows cover 77% of the group's exposure and are an evening's work for a native speaker. |
| **Pronunciation accuracy** | Not measured end to end. Every number here is intrinsic — coverage, agreement between sources, cross-validated derivation. |
| **Whether it sounds right** | **No native speaker has heard any output.** |

The contrast with Thai is worth stating, because it decides where effort
goes. Thai's hard problem was structural — no spaces, so word boundaries
had to be recovered, and the judge for that was itself compromised. That
is now settled: boundary error costs 0.50% of output syllables.
Indonesian's hard problem is lexical, and no amount of engineering
resolves it — a vowel that spelling does not write can only come from a
dictionary or from a speaker.

## How much of this is actually verified

The table above says where each entry came from. This one says how much
anything outside the build has confirmed, which is a different and less
comfortable question. Entry counts and running text answer it differently,
and the gap between the two columns is the whole point:

| | Entries | Share of real tokens |
|---|---:|---:|
| Transcribed by a human (gold) | 17,793 — 10.3% | **70.2%** |
| No ⟨e⟩, so nothing to get wrong | 71,965 — 41.7% | 9.4% |
| Root is in gold, so derivable (97.3%) | 21,508 — 12.5% | 7.7% |
| **Nothing has verified this** | **61,291 — 35.5%** | **6.6%** |
| Not in the dictionary, read by rule (76.2%) | — | 6.1% |

Token shares are measured over 251k tokens of Indonesian Wikipedia.

A third of the dictionary is unverified and it accounts for one token in
fifteen, because those entries are overwhelmingly rare words. Reading the
first column alone overstates the problem; reading the second alone hides
that the tail exists at all.

**Do not read the 97.7% coverage figure as accuracy.** Coverage says a word
was found. It says nothing about whether the ⟨e⟩ in it was resolved
correctly, and those are the two independent things this dictionary has to
get right.

## Inheriting the root's schwa instead of guessing it

Every affix has a fixed pronunciation, so in an affixed word the only
unknown is the root — and if the root is in gold, the whole word is
derivable rather than classifiable. Cross-validated by deriving gold words
from *other* gold roots (4,320 cases): **97.3% correct on the schwa
decision**, against the classifier's ~80%.

Applied to the 21,508 analysable entries outside gold, 86.3% already agreed
— itself evidence the classifier is broadly sound — and 2,080 did not:

    e -> ə  1,101      ə -> e  800      j -> i  216      i -> j  94

Those 2,080 now follow the root. The substitution is deliberately narrow:
only ə/e and i/j are touched, and an entry is skipped whole if it differs
anywhere else, so WikiPron's narrower notation — unreleased stops, ɪ ʊ ɛ ɔ —
cannot leak into a phonemic dictionary.

There is no post-hoc measurement that this improved the dictionary. The
patch derives from gold, so re-measuring against gold would be circular. The
justification is the method's cross-validated accuracy, fixed before the
patch was generated.

### The i/j half was a rule bug

The 310 i/j changes are one cause seen from both sides. "⟨ai⟩ is a diphthong
only word-finally" is right about roots and wrong about affixed forms:

    menguasai   = meN + kuasa + i     the final i is a SUFFIX, not a glide
    serangkaian = se + rangkai + an   the root's diphthong is no longer final

Morphology settles both. Fixing the rule in `id_g2p.py` would generalise to
words never seen; the patch only corrects the entries.

## What still needs a human: `schwa_review.tsv`

51,320 entries remain unverified after the derivation above. They are rare,
but not uniformly so, and the weight is extremely concentrated:

| Words reviewed | Share of the unverified group's weight |
|---:|---:|
| 100 | 52.7% |
| 500 | **77.1%** |
| 2,000 | 95.3% |

So the practical task is not 51,320 words. It is **500**, which is an
evening's work, and `schwa_review.tsv` ships the top 2,000 ordered by
corpus frequency:

    freq  word     pronunciation  schwa_decision  correct
    1239  ke       k ə           e1=ə
     520  israel   i s r a ə l   e1=ə
     380  sistem   s i s t ə m   e1=ə

A reviewer needs no IPA. The `schwa_decision` column isolates the only
question being asked — is this ⟨e⟩ the vowel of *enak* or of *sedang*? —
numbered by order of appearance for words with more than one. Put `y` or
`n` in the last column, leave it blank when unsure.

Expect roughly 80–100 corrections out of 500, not 500: the pool is already
around 80–85% right. The value of the other 400 rows is turning an estimate
into a measurement.

One warning, from this project's own history. The rows most likely to be
wrong are loanwords ending in `-el`, `-er`, `-en` — and those are exactly
where hand annotation scored **68.4%**, below the classifier it was meant to
improve on. A reviewer who is not a native speaker will make it worse.

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
| `schwa_review.tsv` | the 2,000 unverified words worth a human's time |

## Output format

Phonemes are grouped one syllable per space — `saya makan` is `sa ja ma kan`,
not nine separate phonemes — matching what the Vietnamese and Thai front ends
emit. The dictionary stores one phoneme per token, as WikiPron does, and
`src/lang/id/syllable.rs` groups them at read time.

## Shared machinery

`src/core/numeric` and `src/core/spans` handle mathematical notation and
address-like spans for every language from one implementation, with only the
words supplied per language. Before them `10^6 orang` read as "sepuluh enam"
— six orders of magnitude lost with nothing audible to signal it.

## Known limitations

- 51,320 entries have a schwa nothing outside the build has checked — 6.6%
  of running text. `schwa_review.tsv` is the shortlist; a source that marks
  ⟨é⟩ more widely than KBBI would settle them wholesale, and that remains
  the single highest-value contribution.
- **No native speaker has heard any of this.** Every number here is
  intrinsic: coverage, agreement between sources, cross-validated
  derivation. None of them measures whether the output sounds right, and no
  amount of further measurement from inside the system will.
- 42 entries have non-ASCII headwords (`arrivée`, `führer`) whose vowels the
  build already dropped: `arrivée` is stored as `a r r i v ə`. The
  normalizer folds diacritics before lookup, so these keys are unreachable
  and the words route to the English engine instead — accidentally the
  better reading. Dead weight, 0.024%.
- Javanese names follow Javanese phonology, not Indonesian: Pakubuwana is
  /pakubuwɔnɔ/ to a Javanese speaker, /pakubuwana/ by these rules.
- Final ⟨k⟩ is /k/ or /ʔ/ by lexical choice (887 vs 250 in gold); /k/ is the
  default and the dictionary carries the rest.
