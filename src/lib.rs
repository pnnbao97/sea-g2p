use pyo3::prelude::*;
pub mod g2p;
pub mod punc;
pub mod vi_normalizer;

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
    Ok(())
}
