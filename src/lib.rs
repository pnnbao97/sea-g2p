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

use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
pub mod g2p;
pub mod punc;
pub mod vi_normalizer;

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
}

#[pymethods]
impl G2P {
    #[new]
    fn new(dict_path: &str) -> PyResult<Self> {
        let engine = g2p::G2PEngine::new(dict_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        Ok(G2P { engine })
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
    m.add_class::<vi_normalizer::Normalizer>()?;
    m.add_function(wrap_pyfunction!(punc_norm, m)?)?;
    Ok(())
}
