// Package thaibreak provides high-performance, dictionary-based Thai word segmentation
// and typographic line breaking conforming to the Unicode Line Breaking Algorithm (UAX #14)
// and W3C Thai Layout Requirements.
//
// Basic usage:
//
//	import thaibreak "github.com/kamthorn/thai-break/go"
//
//	// Tokenize Thai sentence into words:
//	words := thaibreak.Words("ภาษาไทยเข้าใจง่าย")
//
//	// Insert typographic line break markers (Zero-Width Space \u200B):
//	broken := thaibreak.Lines("ข้อความขนาดยาว", false)
//
//	// Wrap Thai text to fit visual column width:
//	wrapped := thaibreak.Wrap("ข้อความขนาดยาว", 40)
//
//	// Calculate Thai visual display width:
//	width := thaibreak.DisplayWidth("ภาษาไทย")
package thaibreak
