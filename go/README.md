# ThaiBreak for Go

[![Go Reference](https://pkg.go.dev/badge/github.com/kamthorn/thai-break/go.svg)](https://pkg.go.dev/github.com/kamthorn/thai-break/go)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**ThaiBreak** is a high-performance, dictionary-based Thai word segmentation and typographic line breaking engine for Go. It strictly implements the **Unicode Line Breaking Algorithm (UAX #14)** and **W3C Thai Layout Requirements**, making it ideal for PDF generation, terminal rendering, HTML typesetting, and NLP text processing.

---

## Features

- ⚡ **Microsecond-Level Latency:** ~30–45 µs per sentence in Go (>25,000 sentences/sec per core).
- 🎯 **High Accuracy:** Viterbi dynamic programming on a Directed Acyclic Graph (DAG) with Bigram transition probabilities, scoring **96.88%** word-boundary F1-score on standard benchmarks.
- 📐 **Unicode UAX #14 & W3C Typographic Rules:**
  - **LB13:** Never break before punctuation, closing brackets, or solidus (`/`).
  - **LB23:** Keep Latin letters and digits together (e.g., `WP01`, `ISO29110` never break mid-code).
  - Keeps leading vowels (`เ`, `แ`, `โ`, `ใ`, `ไ`) attached to their initial consonants.
- 📏 **Thai Visual Display Width:** Accurately calculates visual column width by treating upper/lower combining vowels and tone marks as zero-width.
- 🏷️ **HTML Preservation:** Safely inserts break markers in HTML content while ignoring tags, comments, and scripts (`<script>`, `<style>`).
- 🛡️ **Clean & Open:** 100% Public Domain clean-room dictionary (40,800+ entries) released under Apache-2.0.

---

## Installation

```bash
go get github.com/kamthorn/thai-break/go
```

---

## Quick Start

```go
package main

import (
	"fmt"

	thaibreak "github.com/kamthorn/thai-break/go"
)

func main() {
	// 1. Word Tokenization
	words := thaibreak.Words("ภาษาไทยเข้าใจง่าย")
	fmt.Printf("%v\n", words)
	// Output: [ภาษา ไทย เข้าใจ ง่าย]

	// 2. Join words with delimiter
	joined := thaibreak.Join("ภาษาไทยเข้าใจง่าย", "|")
	fmt.Println(joined)
	// Output: ภาษา|ไทย|เข้าใจ|ง่าย

	// 3. Typographic Line Breaking (inserts Zero-Width Space \u200B)
	broken := thaibreak.Lines("ข้อความภาษาไทยขนาดยาวเพื่อทดสอบการตัดบรรทัด", false)
	fmt.Println(broken)

	// 4. HTML-Safe Line Breaking
	html := "<p>สวัสดี <b>ประเทศไทย</b></p>"
	brokenHtml := thaibreak.Lines(html, true)
	fmt.Println(brokenHtml)

	// 5. Thai Visual Display Width (ignoring zero-width tone marks)
	width := thaibreak.DisplayWidth("ภาษาไทย")
	fmt.Printf("Visual Width: %d\n", width)
	// Output: Visual Width: 7 (8 characters minus 1 upper vowel)

	// 6. Word Wrapping to Column Width
	wrapped := thaibreak.Wrap("ฉันรักภาษาไทยมากที่สุดในโลก", 12)
	fmt.Println(wrapped)
}
```

---

## Custom Dictionary & Tokenizer

You can load a custom wordlist or bigram model:

```go
trie, err := thaibreak.LoadTsvFile("path/to/custom_words.txt")
if err != nil {
	log.Fatal(err)
}

tokenizer := thaibreak.NewTokenizer(trie, nil)
words := tokenizer.Tokenize("ข้อความทดสอบ", false)
```

---

## Benchmark

| Metric | ThaiBreak (Go) |
| :--- | :--- |
| **Speed (per sentence)** | **~30–45 µs** |
| **Throughput** | **~25,000 sentences/sec** |
| **Memory per Tokenizer** | **~14 MB** |
| **Word Segmentation F1** | **96.88%** |

---

## License

ThaiBreak is open-sourced under the [Apache License 2.0](LICENSE).
The bundled wordlist is 100% Public Domain.
