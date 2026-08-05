//! Rust core of sea-g2p: Vietnamese text normalization and grapheme-to-phoneme
//! conversion, exposed to Python through PyO3.
//!
//! Two independent stages, usable together or apart:
//!
//!   - [`vi_normalizer`] rewrites raw text into something pronounceable —
//!     numbers, dates, units, abbreviations, formulas, URLs. Its module docs
//!     describe the staged pipeline and the ordering contract between stages.
//!   - [`g2p`] maps normalized text to phonemes, resolving Vietnamese and
//!     English readings for the same token from surrounding context.
//!
//! [`punc`] holds the shared trailing-punctuation rule, applied at chunk
//! boundaries by both stages.
//!
//! # Layout
//!
//! - [`core`] — language-agnostic infrastructure: the memory-mapped phoneme
//!   dictionary and the generic abbreviation table.
//! - [`lang`] — one module per language (`vi`, `en`, `th`), each owning its
//!   own tables and rules.
//! - [`g2p`] — the shared engine that turns normalized Latin-script text into
//!   phonemes, resolving Vietnamese/English per token from context.
//!
//! Tests live in `tests/`, not beside the code: `tests/*.rs` for Rust,
//! `tests/python/` for the Python end-to-end suite.

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
pub mod core;
pub mod g2p;
pub mod lang;
pub mod punc;

/// Normalize trailing punctuation as a **pure string operation**, without
/// re-running normalization or G2P.
///
/// Sentences shorter than five words are forced to end in exactly one ".";
/// longer ones only get a "." appended when they do not already end in
/// `,` `.` `!` `?`. Pipelines that need to settle punctuation at a chunk
/// boundary — text or phonemes — can call this instead of normalizing again.
#[pyfunction]
fn punc_norm(text: &str) -> String {
    crate::punc::apply_punc_norm(text)
}

#[pyclass]
struct G2P {
    engine: g2p::G2PEngine,
    thai: std::sync::OnceLock<lang::th::Thai>,
}

#[pymethods]
impl G2P {
    #[new]
    fn new(dict_path: &str) -> PyResult<Self> {
        let engine = g2p::G2PEngine::new(dict_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        Ok(G2P { engine, thai: std::sync::OnceLock::new() })
    }

    /// Phonemize Indonesian text: normalize, then read each token from the
    /// Indonesian dictionary, the English engine, or the rules.
    fn phonemize_id(&self, text: &str) -> String {
        let id = lang::id::Indonesian::new();
        id.phonemize_with(text, &self.engine.dict, |latin| self.engine.phonemize(latin))
    }

    /// Normalize Indonesian text without phonemizing.
    fn normalize_id(&self, text: &str) -> String {
        lang::id::normalizer::normalize(text)
    }

    /// Phonemize a batch of Indonesian texts in parallel.
    fn phonemize_id_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        let id = lang::id::Indonesian::new();
        py.allow_threads(|| {
            use rayon::prelude::*;
            texts
                .into_par_iter()
                .map(|t| id.phonemize_with(&t, &self.engine.dict, |l| self.engine.phonemize(l)))
                .collect()
        })
    }

    /// Normalize a batch of Indonesian texts in parallel.
    fn normalize_id_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        py.allow_threads(|| {
            use rayon::prelude::*;
            texts.into_par_iter().map(|t| lang::id::normalizer::normalize(&t)).collect()
        })
    }

    /// Normalize Thai text without phonemizing: numbers, dates, abbreviations
    /// and symbols become Thai words. The `ๆ` repetition mark is intentionally
    /// left in place — it repeats the previous *word*, which only exists after
    /// segmentation, so `phonemize_th` applies it.
    fn normalize_th(&self, text: &str) -> String {
        lang::th::normalizer::normalize(text)
    }

    /// Normalize a batch of Thai texts in parallel.
    fn normalize_th_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        py.allow_threads(|| {
            use rayon::prelude::*;
            texts
                .into_par_iter()
                .map(|t| lang::th::normalizer::normalize(&t))
                .collect()
        })
    }

    /// Report characters of `text` that Thai normalization would delete
    /// without speaking them.
    ///
    /// The counterpart of the Vietnamese `Normalizer.audit`. Without it the
    /// Python wrapper had no way to reach this pipeline's audit and fell back
    /// to the Vietnamese one, so a symbol missing from the *Thai* tables was
    /// checked against Vietnamese rules and reported as fine.
    fn audit_th(&self, text: &str) -> Vec<String> {
        lang::th::normalizer::audit_unmapped(text)
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    /// Report characters of `text` that Indonesian normalization would delete
    /// without speaking them.
    fn audit_id(&self, text: &str) -> Vec<String> {
        lang::id::normalizer::audit_unmapped(text)
            .into_iter()
            .map(|c| c.to_string())
            .collect()
    }

    /// Phonemize a batch of Thai texts in parallel.
    fn phonemize_th_batch(&self, py: Python<'_>, texts: Vec<String>) -> Vec<String> {
        let th = self.thai.get_or_init(|| lang::th::Thai::new(&self.engine.dict));
        py.allow_threads(|| {
            use rayon::prelude::*;
            texts
                .into_par_iter()
                .map(|t| th.phonemize_with(&t, &self.engine.dict, |l| self.engine.phonemize(l)))
                .collect()
        })
    }

    /// Segment Thai text into words. Returns `(text, known)` pairs, where
    /// `known` is `None` for non-Thai runs passed through verbatim.
    fn segment_th(&self, text: &str) -> Vec<(String, Option<bool>)> {
        let th = self.thai.get_or_init(|| lang::th::Thai::new(&self.engine.dict));
        th.segment(text).into_iter().map(|t| (t.text, t.known)).collect()
    }

    /// Phonemize Thai text: normalize, segment, then read each token — Thai
    /// words from the dictionary (or by rule), Latin runs through the same
    /// engine that serves English elsewhere, so code-switched text comes out
    /// as one phoneme string.
    fn phonemize_th(&self, text: &str) -> String {
        let th = self.thai.get_or_init(|| lang::th::Thai::new(&self.engine.dict));
        th.phonemize_with(text, &self.engine.dict, |latin| self.engine.phonemize(latin))
    }

    #[pyo3(signature = (text, punc_norm=false))]
    fn phonemize(&self, text: &str, punc_norm: bool) -> PyResult<String> {
        let input = if punc_norm { crate::punc::apply_punc_norm(text) } else { text.to_string() };
        Ok(self.engine.phonemize(&input))
    }

    #[pyo3(signature = (texts, punc_norm=false))]
    fn phonemize_batch(&self, py: Python<'_>, texts: Vec<String>, punc_norm: bool) -> PyResult<Vec<String>> {
        py.allow_threads(|| {
            use rayon::prelude::*;
            Ok(texts.into_par_iter().map(|t| {
                let input = if punc_norm { crate::punc::apply_punc_norm(&t) } else { t };
                self.engine.phonemize(&input)
            }).collect())
        })
    }
}

/// sea_g2p_rs: Rust core for sea-g2p
#[pymodule]
fn sea_g2p_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<G2P>()?;
    m.add_class::<lang::vi::Normalizer>()?;
    m.add_function(wrap_pyfunction!(punc_norm, m)?)?;
    Ok(())
}
