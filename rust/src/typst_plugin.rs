#[cfg(feature = "typst-plugin")]
use wasm_minimal_protocol::*;

#[cfg(feature = "typst-plugin")]
initiate_protocol!();

#[cfg(feature = "typst-plugin")]
#[wasm_func]
pub fn break_lines(text: &[u8]) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(text).map_err(|e| e.to_string())?;
    let out = crate::lines(s, "\u{200B}", false);
    Ok(out.into_bytes())
}

#[cfg(feature = "typst-plugin")]
#[wasm_func]
pub fn break_lines_with_marker(text: &[u8], marker: &[u8]) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(text).map_err(|e| e.to_string())?;
    let m = std::str::from_utf8(marker).map_err(|e| e.to_string())?;
    let out = crate::lines(s, m, false);
    Ok(out.into_bytes())
}

#[cfg(feature = "typst-plugin")]
#[wasm_func]
pub fn segment_words(text: &[u8]) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(text).map_err(|e| e.to_string())?;
    let words = crate::words(s);
    let out = words.join("\n");
    Ok(out.into_bytes())
}

#[cfg(feature = "typst-plugin")]
#[wasm_func]
pub fn segment_breaks(text: &[u8]) -> Result<Vec<u8>, String> {
    let s = std::str::from_utf8(text).map_err(|e| e.to_string())?;
    let out = crate::lines(s, "\n", false);
    Ok(out.into_bytes())
}
