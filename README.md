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

# Single text
result = pipeline.run("Giá SP500 hôm nay là 4.200,5 điểm.")
print(result)
#zˈaːɜ ˈɛɜt̪ pˈe nˈam tʃˈam hˈom nˈaj lˌaː2 bˈoɜn ŋˈi2n hˈaːj tʃˈam fˈəɪ4 nˈam ɗˈiɛ4m.

# Batch processing (Parallel)
texts = ["Giá cổ phiếu tăng từ $0.000045 lên $1,234.5678 trong 3.5×10^6 giao dịch.", "Hãy gửi email đến support@example.com."] * 1000
results = pipeline.run(texts)
```

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
- **Smart Normalization**: Specialized for Vietnamese (numbers, dates, technical terms).
- **Bilingual Support**: Handles mixed Vietnamese/English text seamlessly.

## 📊 Performance

The following benchmarks were conducted on a dataset of **1,000,000 sentences**:

| Module | Implementation | Throughput | 
| :--- | :--- | :--- | 
| **Normalizer** | Rust Core (Parallel) | **~41,000 sentences/s** |
| **G2P** | Rust Core (Parallel) | **~415,000 sentences/s** |

**Total Pipeline Throughput**: **~37,000 sentences/s**
*(Tested on CPython 3.12, Windows 11, Multithreaded)*

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
