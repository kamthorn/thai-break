//! A `BreakIterator` over the word boundaries of a text, in the style of ICU and `Intl.Segmenter`.
//!
//! The boundaries are those of [`Tokenizer::tokenize`] (Viterbi), so whitespace and punctuation
//! are segments of their own. Offsets are byte offsets into the text, always on a character
//! boundary. `None` plays the role of ICU's `DONE`.

use crate::tokenizer::Tokenizer;

#[derive(Clone, Debug)]
pub struct BreakIterator {
    /// Sorted boundary offsets, starting with 0 and ending with the text length
    boundaries: Vec<usize>,
    /// Index into `boundaries` of the current position
    index: usize,
}

impl BreakIterator {
    /// Segment `text` and put the iterator at its start.
    pub fn new(tokenizer: &Tokenizer, text: &str) -> Self {
        let mut boundaries = vec![0];
        let mut pos = 0;
        for token in tokenizer.tokenize(text, true) {
            pos += token.len();
            boundaries.push(pos);
        }
        boundaries.dedup();
        Self { boundaries, index: 0 }
    }

    /// All boundary offsets in bytes, from 0 to the text length.
    pub fn boundaries(&self) -> &[usize] {
        &self.boundaries
    }

    /// The boundaries as offsets in code points (Python `str` indices) or, with `utf16`, in UTF-16
    /// code units (JavaScript string indices). `text` must be the text the iterator was made from.
    pub fn boundaries_in(&self, text: &str, utf16: bool) -> Vec<usize> {
        let mut result = Vec::with_capacity(self.boundaries.len());
        let mut targets = self.boundaries.iter().copied().peekable();
        let mut units = 0;
        for (byte, ch) in text.char_indices().chain(std::iter::once((text.len(), '\0'))) {
            while targets.next_if(|&b| b == byte).is_some() {
                result.push(units);
            }
            units += if utf16 { ch.len_utf16() } else { 1 };
        }
        result
    }

    /// The current boundary.
    pub fn current(&self) -> usize {
        self.boundaries[self.index]
    }

    /// Move to the first boundary (the start of the text).
    pub fn first_boundary(&mut self) -> usize {
        self.index = 0;
        self.current()
    }

    /// Move to the last boundary (the end of the text).
    pub fn last_boundary(&mut self) -> usize {
        self.index = self.boundaries.len() - 1;
        self.current()
    }

    /// Move to the next boundary, or `None` at the end of the text.
    pub fn next_boundary(&mut self) -> Option<usize> {
        if self.index + 1 >= self.boundaries.len() {
            return None;
        }
        self.index += 1;
        Some(self.current())
    }

    /// Move to the previous boundary, or `None` at the start of the text.
    pub fn previous(&mut self) -> Option<usize> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        Some(self.current())
    }

    /// Move to the first boundary after `offset`, or `None` (and go to the end) if there is none.
    pub fn following(&mut self, offset: usize) -> Option<usize> {
        let i = self.boundaries.partition_point(|&b| b <= offset);
        if i >= self.boundaries.len() {
            self.index = self.boundaries.len() - 1;
            return None;
        }
        self.index = i;
        Some(self.current())
    }

    /// Move to the last boundary before `offset`, or `None` (and go to the start) if there is none.
    pub fn preceding(&mut self, offset: usize) -> Option<usize> {
        let i = self.boundaries.partition_point(|&b| b < offset);
        if i == 0 {
            self.index = 0;
            return None;
        }
        self.index = i - 1;
        Some(self.current())
    }

    /// Whether `offset` is a boundary. The iterator moves to the first boundary at or after it.
    pub fn is_boundary(&mut self, offset: usize) -> bool {
        let i = self.boundaries.partition_point(|&b| b < offset);
        self.index = i.min(self.boundaries.len() - 1);
        self.boundaries.get(i) == Some(&offset)
    }
}

/// Yields the boundaries after the current one, so `BreakIterator::new(..)` yields every boundary
/// except the start of the text.
impl Iterator for BreakIterator {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        self.next_boundary()
    }
}
