#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::bigram::BigramModel;
#[cfg(feature = "wasm")]
use crate::linebreaker::{thai_display_width, LineBreaker};
#[cfg(feature = "wasm")]
use crate::tokenizer::Tokenizer;
#[cfg(feature = "wasm")]
use crate::trie::ThaiTrie;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct WasmThaiBreak {
    tokenizer: Tokenizer,
    breaker: LineBreaker,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl WasmThaiBreak {
    #[wasm_bindgen(constructor)]
    pub fn new(dict_tsv: &str, bigrams_tsv: Option<String>) -> Result<WasmThaiBreak, JsValue> {
        let trie = ThaiTrie::load_tsv(dict_tsv.as_bytes())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let bigrams = match bigrams_tsv {
            Some(ref b) => BigramModel::load_tsv(b.as_bytes(), 0.15).ok(),
            None => None,
        };

        let tokenizer = Tokenizer::new(trie, bigrams);
        let breaker = LineBreaker::new(tokenizer.clone());

        Ok(WasmThaiBreak { tokenizer, breaker })
    }

    #[wasm_bindgen]
    pub fn words(&self, text: &str) -> Vec<String> {
        self.tokenizer.tokenize(text, false)
    }

    #[wasm_bindgen]
    pub fn lines(&self, text: &str, marker: Option<String>, is_html: Option<bool>) -> String {
        let m = marker.as_deref().unwrap_or("\u{200B}");
        let h = is_html.unwrap_or(false);
        self.breaker.insert_line_breaks(text, m, h)
    }

    #[wasm_bindgen]
    pub fn wrap(&self, text: &str, width: usize, is_html: Option<bool>) -> String {
        let h = is_html.unwrap_or(false);
        self.breaker.wrap(text, width, h)
    }

    #[wasm_bindgen(js_name = displayWidth)]
    pub fn display_width(&self, text: &str) -> usize {
        thai_display_width(text)
    }
}
