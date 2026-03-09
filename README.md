# SEA-G2P

Fast multilingual text-to-phoneme converter for South East Asian languages.

## Installation

```bash
pip install sea-g2p
```

Requires `espeak-ng` only for fallback (built-in dictionary already covers ~99.9% of words).

## Usage

### Simple Pipeline

```python
from sea_g2p import SEAPipeline

pipeline = SEAPipeline(lang="vi")
result = pipeline.run("Giá SP500 hôm nay là 4.200,5 điểm")
print(result)
```

### Individual Modules

```python
from sea_g2p import Normalizer, G2P

normalizer = Normalizer(lang="vi")
g2p = G2P(lang="vi")

text = "Giá SP500 hôm nay là 4.200,5 điểm"
normalized = normalizer.normalize(text)
phonemes = g2p.convert(normalized)
print(phonemes)
```

## Features

- Fast dictionary-based lookup using SQLite.
- Vietnamese text normalization (numbers, dates, units).
- Bilingual support (Vietnamese/English).
- Batch processing for efficiency.
- eSpeak-NG fallback for unknown words.

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
