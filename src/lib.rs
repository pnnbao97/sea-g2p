use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
pub mod g2p;
pub mod punc;
pub mod vi_normalizer;

/// Chuẩn hóa dấu câu cuối (punc_norm) như một hàm chuỗi THUẦN, không chạy lại
/// normalize/G2P. Câu < 5 từ bị ép kết thúc bằng đúng một ".", câu dài chỉ thêm
/// "." nếu chưa kết thúc bằng , . ! ?. Dùng chung cho các pipeline cần chốt dấu
/// câu ở ranh giới chunk (text hoặc phoneme) mà không phải chuẩn hóa lại.
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
