#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
use crate::bigram::BigramModel;
#[cfg(feature = "python")]
use crate::linebreaker::{thai_display_width, LineBreaker};
#[cfg(feature = "python")]
use crate::tokenizer::Tokenizer;
#[cfg(feature = "python")]
use crate::trie::ThaiTrie;

#[cfg(feature = "python")]
#[pyclass(name = "ThaiBreak")]
pub struct PyThaiBreak {
    tokenizer: Tokenizer,
    breaker: LineBreaker,
}

#[cfg(feature = "python")]
#[pymethods]
impl PyThaiBreak {
    #[new]
    #[pyo3(signature = (dict_path=None, bigram_path=None))]
    pub fn new(dict_path: Option<String>, bigram_path: Option<String>) -> PyResult<Self> {
        let trie = match dict_path {
            Some(p) => ThaiTrie::load_tsv_file(p)
                .map_err(|e| pyo3::exceptions::PyIOError::new_err(e.to_string()))?,
            None => ThaiTrie::new(),
        };

        let bigrams = match bigram_path {
            Some(p) => BigramModel::load_tsv_file(p, 0.15).ok(),
            None => None,
        };

        let tokenizer = Tokenizer::new(trie, bigrams);
        let breaker = LineBreaker::new(tokenizer.clone());

        Ok(Self { tokenizer, breaker })
    }

    #[pyo3(signature = (text, keep_whitespace=false))]
    pub fn words(&self, text: &str, keep_whitespace: bool) -> Vec<String> {
        self.tokenizer.tokenize(text, keep_whitespace)
    }

    #[pyo3(signature = (text, marker="\u{200B}", is_html=false))]
    pub fn lines(&self, text: &str, marker: &str, is_html: bool) -> String {
        self.breaker.insert_line_breaks(text, marker, is_html)
    }

    #[pyo3(signature = (text, width, is_html=false))]
    pub fn wrap(&self, text: &str, width: usize, is_html: bool) -> String {
        self.breaker.wrap(text, width, is_html)
    }

    pub fn display_width(&self, text: &str) -> usize {
        thai_display_width(text)
    }
}

#[cfg(feature = "python")]
#[pymodule]
fn thaibreak(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyThaiBreak>()?;
    Ok(())
}
