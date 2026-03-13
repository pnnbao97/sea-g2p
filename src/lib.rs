use pyo3::prelude::*;
pub mod g2p;
pub mod normalizer;

#[pyclass]
struct G2P {
    engine: g2p::G2PEngine,
    normalizer: normalizer::Normalizer,
}

#[pymethods]
impl G2P {
    #[new]
    fn new(dict_path: &str) -> PyResult<Self> {
        let engine = g2p::G2PEngine::new(dict_path)
            .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?;
        let normalizer = normalizer::Normalizer::new();
        Ok(G2P { engine, normalizer })
    }

    fn phonemize(&self, text: &str) -> PyResult<String> {
        Ok(self.engine.phonemize(text))
    }

    fn phonemize_batch(&self, texts: Vec<String>) -> PyResult<Vec<String>> {
        Ok(texts.into_iter().map(|t| self.engine.phonemize(&t)).collect())
    }

    fn normalize(&self, text: &str) -> PyResult<String> {
        Ok(self.normalizer.normalize(text))
    }

    fn run_pipeline(&self, text: &str) -> PyResult<String> {
        let normalized = self.normalizer.normalize(text);
        Ok(self.engine.phonemize(&normalized))
    }
}

#[pyclass]
struct Normalizer {
    inner: normalizer::Normalizer,
}

#[pymethods]
impl Normalizer {
    #[new]
    fn new() -> Self {
        Normalizer { inner: normalizer::Normalizer::new() }
    }

    fn normalize(&self, text: &str) -> String {
        self.inner.normalize(text)
    }
}

/// sea_g2p_rs: Rust core for sea-g2p
#[pymodule]
fn sea_g2p_rs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<G2P>()?;
    m.add_class::<Normalizer>()?;
    Ok(())
}
