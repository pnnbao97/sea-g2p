use pyo3::prelude::*;
pub mod g2p;
pub mod normalizer;

#[pyclass]
struct Normalizer {
    inner: normalizer::Normalizer,
}

#[pymethods]
impl Normalizer {
    #[new]
    #[pyo3(signature = (lang = "vi"))]
    fn new(lang: &str) -> Self {
        Normalizer {
            inner: normalizer::Normalizer::new(lang),
        }
    }

    fn normalize(&self, text: &str) -> String {
        self.inner.normalize(text)
    }
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

    fn phonemize(&self, text: &str) -> PyResult<String> {
        Ok(self.engine.phonemize(text))
    }

    fn phonemize_batch(&self, texts: Vec<String>) -> PyResult<Vec<String>> {
        Ok(texts.into_iter().map(|t| self.engine.phonemize(&t)).collect())
    }
}

/// sea_g2p_rs: Rust core for sea-g2p
#[pymodule]
fn sea_g2p_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<G2P>()?;
    m.add_class::<Normalizer>()?;
    Ok(())
}
