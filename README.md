# 🦭 SEA-G2P

<img width="1221" height="656" alt="image" src="https://github.com/user-attachments/assets/01220177-815b-4012-8f65-8a2a86beddf9" />

Fast multilingual text-to-phoneme converter for South East Asian languages.  
Vietnamese, Thai and Indonesian, all with English code-switching.  
>**Author**: [Pham Nguyen Ngoc Bao](https://github.com/pnnbao97)

## 🚀 Used By

SEA-G2P is the core phonemization engine powering:

- [**VieNeu-TTS**](https://github.com/pnnbao97/VieNeu-TTS): An advanced on-device Vietnamese Text-to-Speech model with instant voice cloning.

By using SEA-G2P, VieNeu-TTS achieves high-fidelity pronunciation and seamless Vietnamese-English code-switching.

## Installation

```bash
pip install sea-g2p
```

## Usage

### Simple Pipeline

```python
from sea_g2p import SEAPipeline

pipeline = SEAPipeline(lang="vi")

# Single text
result = pipeline.run("Giá SP500 hôm nay là 4.200,5 điểm.")
print(result)
#zˈaːɜ ˈɛɜt̪ pˈe nˈam tʃˈam hˈom nˈaj lˌaː2 bˈoɜn ŋˈi2n hˈaːj tʃˈam fˈəɪ4 nˈam ɗˈiɛ4m.

# Batch processing (Parallel)
texts = ["Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "Hãy gửi email đến support@example.com."] * 1000
results = pipeline.run(texts)
```

### Thai

Thai is written without spaces, so the Thai front end normalizes, **segments**,
and looks words up in one pass. Latin runs go through the same English engine
used elsewhere, so code-switched text comes out as a single phoneme string.

```python
from sea_g2p import SEAPipeline

th = SEAPipeline(lang="th")

th.run("เขาฉลาดพอที่จะซ่อนสติปัญญา")
# 'kʰaw˩˩˦ tɕʰa˨˩ laːt̚˨˩ pʰɔː˧ tʰiː˥˩ tɕaʔ˨˩ sɔːn˥˩ sa˨˩ ti˨˩ pan˧ jaː˧'

th.run("ผมใช้ iPhone ราคา ฿1,250")
# 'pʰom˩˩˦ tɕʰaj˦˥ ˈaɪfoʊn raː˧ kʰaː˧ nɯŋ˨˩ pʰan˧ sɔːŋ˩˩˦ rɔːj˦˥ haː˥˩ sip̚˨˩ baːt̚˨˩'

# Normalization alone: numbers, Thai digits, dates, abbreviations
from sea_g2p import Normalizer
Normalizer(lang="th").normalize("วันที่ 6 ม.ค. ๒๕๖๐")
# 'วันที่ หก มกราคม สองพันห้าร้อยหกสิบ'
```

Thai phonemes use IPA with **Chao tone letters** (`˧` mid, `˨˩` low, `˥˩`
falling, `˦˥` high, `˩˩˦` rising), deliberately distinct from the digit
convention used for Vietnamese tones so the two can share one inventory
without ambiguity. Details in [thai/README.md](thai/README.md).

### Indonesian

```python
from sea_g2p import SEAPipeline

id = SEAPipeline(lang="id")

id.run("dia cukup cerdas untuk menyembunyikan kecerdasannya")
# 'di a t͡ʃu kup t͡ʃər das un tuʔ mə ɲəm bu ɲi kan kə t͡ʃər da san ɲa'

id.run("Saya membeli buku seharga Rp1.250.000")
# '... sa tu d͡ʒu ta du a ra tus li ma pu luh ri bu ru pi ah'

# chat contractions, which look like pronounceable words to a rule engine
from sea_g2p import Normalizer
Normalizer(lang="id").normalize("yg penting tdk lupa dgn tugasnya")
# 'yang penting tidak lupa dengan tugasnya'
```

Phonemes are grouped one **syllable** per space, the same convention the
Vietnamese and Thai outputs use, so a downstream TTS sees one format for the
whole library.

Indonesian spelling is regular except for one thing: ⟨e⟩ writes both /ə/ and
/e/ and nothing distinguishes them. The dictionary settles it from KBBI, the
official Indonesian dictionary, whose pronunciation field marks the schwa —
see [indo/README.md](indo/README.md) for how the sources were chosen and
which approaches were measured and rejected.

### Individual Modules

```python
from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

# Automatic parallel processing when list is passed
texts = ["Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "Hãy gửi email đến support@example.com."]
normalized = normalizer.normalize(texts)
print(normalized)
#['giá cổ phiếu tăng từ không chấm không không không không bốn lăm <en>u s d</en> lên một nghìn hai trăm ba mươi bốn phẩy năm sáu bảy tám <en>u s d</en> trong ba chấm năm nhân mười mũ sáu giao dịch.', 'hãy gửi email đến <en>support</en> a còng <en>example</en> chấm com.']
phonemes = g2p.convert(normalized)
print(phonemes)
#['zˈaːɜ kˈo4 fˈiɛɜw t̪ˈaŋ t̪ˌy2 xˌoŋ tʃˈəɜm xˌoŋ xˌoŋ xˌoŋ xˌoŋ bˈoɜn lˈam jˈuː ˈɛs dˈiː lˈen mˈo6t̪ ŋˈi2n hˈaːj tʃˈam bˈaː mˈyəj bˈoɜn fˈəɪ4 nˈam sˈaɜw bˈa4j t̪ˈaːɜm jˈuː ˈɛs dˈiː tʃˈɔŋ bˈaː tʃˈəɜm nˈam ɲˈən mˈyə2j mˈu5 sˈaɜw zˈaːw zˈi6c.', 'hˈa5j ɣˈy4j ˈiːmeɪl ɗˌeɜn səpˈɔːɹt ˈaː kˈɔ2ŋ ɛɡzˈæmpəl tʃˈəɜm kˈɔm.']
```

## Features

- **Blazing Fast**: Core engine rewritten in Rust with binary mmap lookup.
- **Multithreading**: Automatic parallel processing using Rayon/Rust for batch inputs.
- **Zero Dependency**: Pre-compiled wheels for Windows, Linux, and macOS.
- **Smart Normalization**: Staged pipelines per language — 17 stages for
  Vietnamese (numbers, dates, units, formulas, technical terms), 8 for Thai
  (Thai digits ๐-๙, Buddhist-era dates, `ๆ` repetition, abbreviation table).
- **Thai word segmentation**: no-space script handled with a 91,865-word
  dictionary and a unigram-cost dynamic program; boundary F1 0.987 against
  PyThaiNLP `newmm`.
- **Indonesian morphology**: 172,557-word dictionary built from WikiPron and
  KBBI, extended by affix derivation, compounding and reduplication rather
  than by machine-generated guesses.
- **Never gives up on a word**: Thai text outside the dictionary is read by
  orthographic rule, so new names and transliterations still get phonemes.
- **Bilingual Support**: Handles mixed Vietnamese/English and Thai/English
  text seamlessly.
- **Markup tags**: Wrap a span to control reading:
  - `<en>...</en>` — keep the content for the English phonemizer (e.g. `<en>hello</en>`).
  - `<math>...</math>` — read as a math formula: variable clusters are spelled
    letter-by-letter and operators/symbols are voiced, while function names
    (`sin`, `cos`, `log`, `lim`, ...) are preserved.
    `<math>b² - 4ac</math>` → *"bê bình phương trừ bốn a xê"*,
    `<math>∫f dx</math>` → *"tích phân ép đê ích"*.

## 📊 Performance

The following benchmarks were conducted on a dataset of **1,000,000 sentences**:

| Language | Module | Throughput |
| :--- | :--- | :--- |
| Vietnamese | Normalizer | **~41,000 sentences/s** |
| Vietnamese | G2P | **~415,000 sentences/s** |
| Vietnamese | **Full pipeline** | **~37,000 sentences/s** |
| Thai | Normalizer | **~1,000,000 sentences/s** |
| Thai | **Full pipeline** (normalize + segment + G2P) | **~180,000 sentences/s** |
| Indonesian | **Full pipeline** | **~500,000 sentences/s** |

*(Tested on CPython 3.12, Windows 11, Multithreaded)*

## Technical Architecture

SEA-G2P is designed for maximum performance in production environments:

- **Memory Mapping (mmap)**: Instead of loading a huge JSON/SQLite into RAM, we use a custom binary format (`.bin`) mapped directly into memory. This allows near-instant startup and extremely low memory overhead.
- **String Pooling**: To minimize file size, all unique strings (words and phonemes) are stored once in a global string pool and referenced by 4-byte IDs.
- **Binary Search**: Words are pre-sorted during the build process, allowing `O(log n)` lookup speeds directly on the memory-mapped data.
- **Per-language sections**: one binary holds every language. Scripts that
  cannot collide with the Latin keyspace get their own namespace, so the Thai
  dictionary and its word frequencies ship beside the Vietnamese/English
  tables and can never fall out of sync with them.

### Source layout

| path | contents |
| :--- | :--- |
| `src/core/` | language-agnostic: the mmap dictionary loader, the generic abbreviation table |
| `src/lang/vi/` | Vietnamese normalizer, number-to-words, syllable data |
| `src/lang/en/` | English frequency wordlist used to settle ambiguous splits |
| `src/lang/th/` | Thai normalizer, segmenter, rule-based G2P, number-to-words |
| `src/lang/id/` | Indonesian normalizer, rule-based G2P, number-to-words |
| `src/g2p/` | the shared engine for Latin-script text |
| `tests/` | `*.rs` integration tests and `python/` end-to-end tests |

For the binary format specification, see [src/core/dict.rs](src/core/dict.rs).
For the Thai and Indonesian data pipelines and their measurements, see
[thai/README.md](thai/README.md) and [indo/README.md](indo/README.md).

## Development

To install for development purposes:

1. Clone the repository:
   ```bash
   git clone https://github.com/pnnbao97/sea-g2p
   cd sea-g2p
   ```

2. Install in editable mode:
   ```bash
   pip install -e .
   ```

3. Run the tests:
   ```bash
   cargo test --release      # Rust integration tests
   python -m pytest tests/   # Python end-to-end tests
   ```
