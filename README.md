# 🦭 SEA-G2P

<img width="1221" height="656" alt="image" src="https://github.com/user-attachments/assets/01220177-815b-4012-8f65-8a2a86beddf9" />

Fast multilingual text-to-phoneme converter for South East Asian languages.  
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
result = pipeline.run("Giá SP500 hôm nay là 4.200,5 điểm.")
print(result)
#zˈaːɜ ˈɛɜt̪ pˈe nˈam tʃˈam hˈom nˈaj lˌaː2 bˈoɜn ŋˈi2n hˈaːj tʃˈam fˈəɪ4 nˈam ɗˈiɛ4m.
```

### Individual Modules

```python
from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

text = "Giá SP500 hôm nay là 4.200,5 điểm"
normalized = normalizer.normalize(text)
print(normalized)
phonemes = g2p.convert(normalized)
print(phonemes)
#giá ét pê năm trăm hôm nay là bốn nghìn hai trăm phẩy năm điểm.
#zˈaːɜ ˈɛɜt̪ pˈe nˈam tʃˈam hˈom nˈaj lˌaː2 bˈoɜn ŋˈi2n hˈaːj tʃˈam fˈəɪ4 nˈam ɗˈiɛ4m.
```

## Features

- **Blazing Fast**: Core engine rewritten in Rust with binary mmap lookup.
- **Zero Dependency**: Pre-compiled wheels for Windows, Linux, and macOS.
- **Smart Normalization**: Specialized for Vietnamese (numbers, dates, technical terms).
- **Bilingual Support**: Handles mixed Vietnamese/English text seamlessly.
- **Character Fallback**: Built-in intelligent fallback for unknown words.

## Technical Architecture

SEA-G2P is designed for maximum performance in production environments:

- **Memory Mapping (mmap)**: Instead of loading a huge JSON/SQLite into RAM, we use a custom binary format (`.bin`) mapped directly into memory. This allows near-instant startup and extremely low memory overhead.
- **String Pooling**: To minimize file size, all unique strings (words and phonemes) are stored once in a global string pool and referenced by 4-byte IDs.
- **Binary Search**: Words are pre-sorted during the build process, allowing `O(log n)` lookup speeds directly on the memory-mapped data.

For full details on the specification, see [src/g2p/mod.rs](src/g2p/mod.rs).

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
