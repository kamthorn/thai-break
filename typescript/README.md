# ThaiBreak for JavaScript & TypeScript

[![npm version](https://img.shields.io/npm/v/thai-break.svg)](https://www.npmjs.com/package/thai-break)
[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

**ThaiBreak** (`thai-break`) is a high-performance, dictionary-based Thai word segmentation and typographic line breaking engine for JavaScript and TypeScript (Node.js, Deno, Bun, and modern browsers).

It implements the **Unicode Line Breaking Algorithm (UAX #14)** and **W3C Thai Layout Requirements**, making it ideal for web typesetting, PDF generation, terminal text formatting, and NLP applications.

---

## Features

- ⚡ **Ultra Fast:** ~0.3 ms per sentence in Node.js (~25,000 sentences/sec per core).
- 🎯 **High Accuracy:** Viterbi dynamic programming on a Directed Acyclic Graph (DAG) with Bigram transition scoring, achieving **96.88%** word-boundary F1-score.
- 📐 **Unicode UAX #14 Typographic Rules:**
  - **LB13:** Never break before punctuation, closing brackets, or solidus (`/`).
  - **LB23:** Keep Latin letters and numbers together (e.g. `WP01`, `ISO29110` never break mid-code).
  - Attached leading vowels (`เ`, `แ`, `โ`, `ใ`, `ไ`) to initial consonants.
- 📏 **Thai Visual Display Width:** Accurately computes visual terminal/column width by treating above/below combining vowels and tone marks as zero-width.
- 🏷️ **HTML Safe:** Safely inserts soft breaks inside HTML strings while preserving tags, comments, and scripts (`<script>`, `<style>`).
- 🛡️ **Clean & Open:** 100% Public Domain dictionary (40,800+ entries) released under Apache-2.0.

---

## Installation

```bash
npm install thai-break
```

---

## Usage

### 1. Word Tokenization

```typescript
import { words, tokenize, join } from 'thai-break';

// Segment into word tokens
const tokens = words('ภาษาไทยเข้าใจง่าย');
console.log(tokens);
// Output: ['ภาษา', 'ไทย', 'เข้าใจ', 'ง่าย']

// Join with delimiter
console.log(join('ภาษาไทยเข้าใจง่าย', '|'));
// Output: ภาษา|ไทย|เข้าใจ|ง่าย
```

### 2. Typographic Line Breaking

Inserts Zero-Width Space (`\u200B`) at valid line-break positions according to Unicode UAX #14:

```typescript
import { lines } from 'thai-break';

// Plain text (inserts Zero-Width Space \u200B)
const broken = lines('ข้อความภาษาไทยขนาดยาวเพื่อทดสอบการตัดบรรทัด');

// With HTML tag preservation (or pass custom marker like '<wbr>')
const html = '<p>สวัสดี <b>ประเทศไทย</b></p>';
const brokenHtml = lines(html, '<wbr>', true);
console.log(brokenHtml);
// Output: <p>สวัสดี <b>ประเทศ<wbr>ไทย</b></p>
```

### 3. Visual Display Width & Text Wrapping

```typescript
import { displayWidth, wrap } from 'thai-break';

// Thai combining vowels and tone marks do not consume visual column width
console.log(displayWidth('ภาษาไทย')); // 7 columns (8 characters minus 1 upper vowel)

// Soft-wrap text to fit target visual width
const wrapped = wrap('ฉันรักภาษาไทยมากที่สุดในโลก', 12);
console.log(wrapped);
```

---

## API Reference

| Function | Description |
| :--- | :--- |
| `words(text, keepWhitespace?)` | Tokenize Thai string into an array of words |
| `tokenize(text, keepWhitespace?)` | Alias of `words` |
| `lines(text, marker?, isHtml?)` | Insert break markers (default `\u200B`) adhering to UAX #14 |
| `wrap(text, width, delimiter?, isHtml?)` | Soft-wrap text to maximum visual column width |
| `displayWidth(text)` | Calculate visual column width ignoring combining marks |
| `join(text, delimiter)` | Tokenize and join words with delimiter |

---

## License

[Apache-2.0](LICENSE) © Kamthorn Krairaksa
