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
| `vi_audit.rs` | Vietnamese silent-deletion audit |

Python tests need the built extension: `maturin build --release`, then copy
`sea_g2p_rs.pyd`/`.so` from the wheel into `python/sea_g2p/`.
