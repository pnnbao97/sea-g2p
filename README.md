# 🦭 SEA-G2P

<img width="1221" height="656" alt="image" src="https://github.com/user-attachments/assets/01220177-815b-4012-8f65-8a2a86beddf9" />

Fast multilingual text-to-phoneme converter for South East Asian languages.  
>**Author**: [Pham Nguyen Ngoc Bao](https://github.com/pnnbao97)

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
