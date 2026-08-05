# Tests

Two suites, one directory:

| path | suite | run with |
|---|---|---|
| `tests/*.rs` | Rust integration tests | `cargo test --release` |
| `tests/python/test_*.py` | Python end-to-end tests | `python -m pytest tests/` |

The Rust files must sit directly in `tests/` — Cargo only discovers
integration tests there. They exercise the crate through its **public API**,
the same surface the Python bindings use, so a test passing here means the
behaviour is reachable from outside the crate rather than only from inside
its own module.

`src/` deliberately contains no `#[cfg(test)]` blocks: every test lives here.

| file | covers |
|---|---|
| `core_dict.rs` | SEAP v2 loader against the real shipped binary |
| `punc.rs` | shared trailing-punctuation rule |
| `th_normalizer.rs` | Thai normalizer stages + silent-deletion audit |
| `th_num2th.rs` | Thai number-to-words (สิบ / ยี่สิบ / เอ็ด alternations) |
| `th_rules.rs` | rule-based Thai G2P, the out-of-vocabulary fallback |
| `th_segment.rs` | Thai spelling normalization and word segmentation |
| `th_codeswitch.rs`, `th_repetition.rs`, `th_punctuation.rs` | Latin runs inside Thai, `ๆ`, punctuation |
| `id_normalizer.rs`, `id_g2p.rs`, `id_syllable.rs` | the Indonesian pipeline |
| `vi_audit.rs` | Vietnamese silent-deletion audit |

| file | covers |
|---|---|
| `python/test_normalize.py` | Vietnamese normalization, end to end |
| `python/test_lang_normalize.py` | Thai and Indonesian through the Python API, including that `audit` follows `lang` |
| `python/test_phonemize.py`, `test_pipeline.py`, `test_bilingual_phonemize.py` | G2P and the full pipeline |

## Running the Python suite

`pyproject.toml` sets `pythonpath = ["python"]`, so pytest imports the
**source tree** `python/sea_g2p/` — never whatever is in `site-packages`.
That package needs the compiled extension sitting beside it, and an editable
install is what puts it there:

```
uv sync --all-extras --dev      # builds the extension into python/sea_g2p/
uv run pytest
```

**The extension goes stale silently.** `cargo build`, `maturin build` and
`pip install <wheel>` all leave `python/sea_g2p/sea_g2p_rs.pyd` untouched, so
after editing Rust the suite keeps testing the previous build and keeps
passing. Re-run `uv sync` (or copy `target/release/sea_g2p_rs.dll` over the
`.pyd`) before trusting a green run. CI is immune: it starts from a clean
checkout, so `uv sync` always builds fresh.
