# ThaiBreak — Fast Thai Word Segmenter & Typographic Line Breaker

ระบบตัดคำและตัดแบ่งบรรทัดภาษาไทยประสิทธิภาพสูงแบบ **Multi-Language Engine** รองรับ **PHP / Laravel**, **Go**, **TypeScript / Node.js**, **Rust Core**, **C / C++**, **Python**, และ **WebAssembly**  
ใช้อัลกอริทึม **Shortest Path Viterbi DAG** ร่วมกับ **Theeramunkong 30-Rule Thai Character Cluster (TCC)** บนพจนานุกรมมาตรฐานภาษาไทย (`data/words.txt` 25,907 คำ)

🚀 **Zero External Corpus Dependency:** ไม่พึ่งพาคลังข้อความที่มีข้อจำกัดทางลิขสิทธิ์ เป็น Open Source Apache-2.0 แท้ 100% ใช้งานเชิงพาณิชย์ได้อย่างสบายใจ  
⚡ **ความเร็วสูงระดับไมโครวินาที:** ~30-45 µs ใน Go/Rust, ~0.3 ms ใน PHP/Node.js (~25,000 ประโยค/วินาทีต่อ core)  
💾 **Ultra-Lightweight & Low Memory:** ใช้หน่วยความจำน้อยมากเพียง ~10-15 MB ขนาดพจนานุกรมเพียง ~500 KB  
📄 **Typographic Line Breaker & Soft Wrapping:** ตัดแบ่งบรรทัดป้องกันสระลอย/ตกขอบ ตามมาตรฐาน **W3C tlreq** และ **Unicode UAX #14** สำหรับ PDF (dompdf, mPDF, TCPDF, Typst) และ Web  
🔥 **พร้อมใช้งานกับ Laravel:** Auto-Discovery, Facade, Blade Directives (`@thaibreak`, `@thaiwrap`), `Str` Macros

---

## คุณสมบัติเด่น

1. **Shortest Path Graph Viterbi Algorithm:** อัลกอริทึมค้นหาเส้นทางคำที่เหมาะสมที่สุดบนกราฟ ค้นหาคำที่ยาวและถูกต้องสมบูรณ์ตามธรรมชาติ
2. **Theeramunkong et al. TCC Grammar (30 Rules):** คำนวณจุดตัดคลัสเตอร์ภาษาไทยระดับไบต์ออฟเซ็ต ป้องกันการตัดแยกสระ สระบน-ล่าง วรรณยุกต์ หรือพยัญชนะนำ 100%
3. **Flat Prefix Hash Map Trie:** โครงสร้างข้อมูลแบบ Flat Hash Map ค้นหาคำได้เร็วกว่า Nested Tree 2 เท่า พร้อมประหยัด RAM ลงกว่า 80%
4. **Smart OOV & Abbreviation Handling:** รู้จักคำย่อภาษาไทย (`รพ.`, `พ.ศ.`, `มิ.ย.`), ตัวเลขคั่นจุลภาค (`10,000`), ทศนิยม (`3.14`), เปอร์เซ็นต์ (`40%`)
5. **ThaiLineBreaker (UAX #14 & W3C Thai Text Layout):** ตัดแบ่งบรรทัดสำหรับทำ PDF หรือเว็บ ไม่ตัดกลางคำ ไม่ทิ้งวรรคไว้หน้าบรรทัดใหม่ ป้องกันเครื่องหมายตกค้าง (`ๆ`, `ฯ`, วงเล็บปิด)
6. **HTML / EPUB Safe:** รักษาแท็ก HTML (`<p>`, `<b>`, `<span>`) และ HTML Entities (`&amp;`, `&quot;`) ให้คงอยู่สมบูรณ์ ไม่แทรกสัญลักษณ์ตัดคำเข้าไปภายในแท็ก

---

## รองรับหลายภาษาโปรแกรม (Multi-Language Architecture)

ThaiBreak ได้รับการออกแบบสถาปัตยกรรมแบบ Monorepo เพื่อรองรับการใช้งานในทุก Stack โดยใช้คลังคำศัพท์มาตรฐาน (`data/words.txt`: 25,907 คำ) เป็น Single Source of Truth:

```
PHPThaiNLP / ThaiBreak
├── data/                 # Shared Dictionary (data/words.txt)
├── src/                  # Native PHP & Laravel Package (Composer: kamthorn/thai-break)
├── go/                   # Native Go Package (go get github.com/kamthorn/thai-break/go)
├── typescript/           # Native TypeScript / Node.js Package (npm: thai-break)
├── rust/                 # High-Performance Rust Core (Cargo: thaibreak)
│   ├── include/          # C / C++ Header (thaibreak.h)
│   └── src/              # Core Engine + C FFI + Python (PyO3) + Wasm (wasm-bindgen)
└── python/               # Python Package (pip: thaibreak)
```

---

## 1. PHP & Laravel

### การติดตั้งผ่าน Composer
```bash
composer require kamthorn/thai-break
```

### การใช้งานทั่วไป (One-liner Quick Start)
```php
use ThaiBreak\ThaiBreak;

// 1. ตัดคำเป็น Array
$words = ThaiBreak::words('ฉันรักภาษาไทย');
// → ['ฉัน', 'รัก', 'ภาษา', 'ไทย']

// 2. ตัดคำคั่นด้วยเครื่องหมาย
$joined = ThaiBreak::join('สวัสดีครับคุณลูกค้า', '|');
// → "สวัสดี|ครับ|คุณ|ลูกค้า"

// 3. แทรกจุดตัดบรรทัด (Zero-Width Space U+200B) สำหรับ Render PDF/HTML
$lines = ThaiBreak::lines('ข้อความยาวๆ ที่ต้องการจัดหน้าใน PDF');

// 4. ตัดแบ่งบรรทัดสำหรับ HTML (รักษาแท็กและ entities)
$html = ThaiBreak::lines('<p>สวัสดี <b>ประเทศไทย</b> &amp; กรุงเทพมหานคร</p>', isHtml: true);

// 5. Hard Wrap ตัดข้อความขึ้นบรรทัดใหม่ตามความกว้างคอลัมน์ (Display Width)
$wrapped = ThaiBreak::wrap('ข้อความภาษาไทยขนาดยาว...', width: 40);
```

### การใช้งานใน Laravel
ThaiBreak รองรับ Laravel Package Auto-Discovery อัตโนมัติ:

```php
use ThaiBreak; // Facade

// เรียกผ่าน Facade
$words = ThaiBreak::words('ข้อความ');

// หรือเรียกผ่าน Str Macro
$words = Str::thaiWords('ฉันรักภาษาไทย');
$html  = Str::thaiLines('ข้อความในหน้าเว็บ');
$pdf   = Str::thaiWrap($text, 40);
```

ใน Blade Template:
```blade
{{-- แทรก Zero-Width Space เพื่อให้ Browser/PDF ตัดคำได้สวยงาม --}}
@thaibreak($post->content)

{{-- ตัดบรรทัดจำกัดความกว้าง 40 ตัวอักษร --}}
@thaiwrap($report->summary, 40)
```

---

## 2. Go (Golang) — Native Implementation

เขียนด้วย Pure Go 100% ไม่พึ่งพา CGO (`CGO_ENABLED=0`) ความเร็วสูง ~30-45 µs ต่อประโยค:

```bash
go get github.com/kamthorn/thai-break/go
```

```go
package main

import (
    "fmt"
    tb "github.com/kamthorn/thai-break/go"
)

func main() {
    // ตัดคำ
    words := tb.Words("ฉันรักภาษาไทย")
    fmt.Println(words) // [ฉัน รัก ภาษา ไทย]

    // แทรกจุดตัดบรรทัด (Zero-Width Space U+200B)
    lines := tb.Lines("สวัสดีครับคุณลูกค้า", false)

    // ป้องกันแท็ก HTML และ Entity
    html := tb.Lines("<b>สวัสดี</b> &amp; ประเทศไทย", true)

    // ตัดบรรทัดตามความกว้างหน้าจอ (คำนวณสระ/วรรณยุกต์ไม่คิดความกว้าง)
    wrapped := tb.Wrap("ข้อความภาษาไทยยาวๆ...", 40)
    fmt.Println(wrapped)
}
```

---

## 3. TypeScript & JavaScript (Node.js / Browser) — Native Implementation

เขียนด้วย TypeScript (ES2022 / NodeNext) ทำงานได้ทั้ง Node.js, Bun, Deno และ Browser:

```bash
npm install thai-break
```

```typescript
import { words, lines, wrap, displayWidth } from 'thai-break';

// ตัดคำ
const tokens = words('ฉันรักภาษาไทย');
console.log(tokens); // ['ฉัน', 'รัก', 'ภาษา', 'ไทย']

// ตัดบรรทัดสำหรับ HTML (รักษาแท็กและ entities ไม่เสียหาย)
const htmlWithZwsp = lines('<div class="header"><b>สวัสดี</b> &amp; ประเทศไทย</div>', true);

// ตัดบรรทัดจำกัดความกว้างคอลัมน์ (Display Width)
const wrapped = wrap('ฉันรักภาษาไทยมากที่สุดในโลก', 12);
console.log(wrapped);

// คำนวณความกว้างตัวอักษรจริง (สระบน-ล่าง/วรรณยุกต์ = 0, CJK = 2)
console.log(displayWidth('ภาษาไทย')); // 7
```

---

## 4. Rust Core Engine (`thaibreak`)

ประสิทธิภาพสูงสุดระดับ Native Machine Code ผ่าน Cargo:

```toml
[dependencies]
thaibreak = { path = "./rust" }
```

```rust
use thaibreak::{words, lines, wrap, display_width, DEFAULT_BREAK_MARKER};

fn main() {
    let tokens = words("ฉันรักภาษาไทย");
    println!("{:?}", tokens); // ["ฉัน", "รัก", "ภาษา", "ไทย"]

    let html = lines("<p>สวัสดีชาวโลก</p>", DEFAULT_BREAK_MARKER, true);
    println!("{}", html);

    let width = display_width("ภาษาไทย");
    println!("Width: {}", width); // 7
}
```

---

## 5. C & C++ (ผ่าน C-ABI Shared / Static Library)

เชื่อมต่อได้ทั้ง C, C++, Qt, หรือภาษาใดๆ ที่รองรับ C-ABI:

```c
#include <stdio.h>
#include "thaibreak.h"

int main() {
    // โหลดพจนานุกรม
    thaibreak_init("data/words.txt", NULL);

    // ตัดคำ
    size_t count = 0;
    char **tokens = thaibreak_tokenize("ฉันรักภาษาไทย", &count);
    for (size_t i = 0; i < count; i++) {
        printf("[%zu] %s\n", i, tokens[i]);
    }
    thaibreak_free_tokens(tokens, count);

    // แทรกจุดตัดบรรทัดใน HTML
    char *broken = thaibreak_lines("<b>สวัสดี</b> ประเทศไทย", NULL, 1);
    printf("%s\n", broken);
    thaibreak_free_string(broken);

    return 0;
}
```

คอมไพล์ด้วย GCC / Clang:
```bash
gcc -I rust/include main.c -L rust/target/release -lthaibreak -o app
```

---

## 6. Python (ผ่าน C-FFI / PyO3)

```bash
pip install ./python
```

```python
import thaibreak

# เริ่มต้นด้วยพจนานุกรม
thaibreak.init("data/words.txt")

# ตัดคำ
tokens = thaibreak.words("ฉันรักภาษาไทย")
print(tokens) # ['ฉัน', 'รัก', 'ภาษา', 'ไทย']

# แทรกจุดตัดบรรทัดสำหรับ HTML
html = thaibreak.lines("<b>สวัสดี</b> &amp; ประเทศไทย", is_html=True)

# ตัดบรรทัดตามความกว้าง
wrapped = thaibreak.wrap("ฉันรักภาษาไทยมากที่สุดในโลก", width=12)
print(wrapped)
```

---

## 7. WebAssembly (Wasm)

คอมไพล์เป็น Wasm สำหรับ Edge/Cloudflare Workers หรือเบราว์เซอร์:

```bash
cd rust && wasm-pack build --target web --features wasm
```

```javascript
import init, { WasmThaiBreak } from './pkg/thaibreak.js';

await init();
const tb = new WasmThaiBreak(dictTextString);
console.log(tb.words("ฉันรักภาษาไทย"));
```

---

## กฎเกณฑ์สำคัญของการตัดบรรทัด (ThaiLineBreaker)

ปัญหาคลาสสิกของการ Render PDF และ EPUB ภาษาไทยคือโปรแกรมตัดคำไทยไม่เป็น ทำให้ข้อความล้นตกขอบ หรือตัดคำแยกกลางสระ/กลางพยางค์

`ThaiBreak` มีระบบตัดบรรทัดที่ปฏิบัติตามมาตรฐาน **W3C Thai Text Layout** และ **Unicode UAX #14**:

1. **ห้ามขึ้นต้นบรรทัด (No Line-Start):** ไม้ยมก (`ๆ`), ไปยาลน้อย (`ฯ`), ไปยาลใหญ่ (`ฯลฯ`), เครื่องหมายวรรคตอนปิด (`)`, `]`, `}`, `”`, `’`), เครื่องหมายจุลภาค/มหัพภาค (`,`, `.`, `:`) จะต้องไม่ไปอยู่โดดเดี่ยวที่ต้นบรรทัดใหม่
2. **ห้ามลงท้ายบรรทัด (No Line-End):** วงเล็บเปิด (`(`, `[`, `{`, `“`), สัญลักษณ์สกุลเงิน (`฿`, `$`) จะต้องไม่ค้างอยู่ท้ายบรรทัดโดยไม่มีข้อความตาม
3. **ป้องกัน Orphan Spaces:** จุดที่มีช่องว่าง (` `) อยู่แล้ว จะไม่ใส่ break marker ติดกับช่องว่าง ป้องกันไม่ให้เกิดวรรคนำหน้าในบรรทัดใหม่
4. **คงความถูกต้องของ HTML/EPUB:** แท็ก HTML (`<p>`, `<b>`, `<span>`) และ HTML Entities (`&amp;`, `&quot;`) จะถูกรักษาไว้อย่างสมบูรณ์ ไม่ถูกแทรกสัญลักษณ์ตัดคำเข้าไปภายในแท็ก
5. **คำนวณความกว้างคอลัมน์ถูกต้อง:** สระบน-ล่าง วรรณยุกต์ ไม่นับความกว้างคอลัมน์ (`thaiDisplayWidth`) ทำให้ตัดบรรทัดได้พอดีความกว้างจริง

---

## การรันชุดทดสอบ (Tests Across All Languages)

```bash
# 1. PHP & Laravel Tests
./vendor/bin/phpunit
php tests/TokenizerTest.php

# 2. Go Tests & Benchmarks
cd go && go test -v ./... && go test -bench=. ./...

# 3. TypeScript / Node.js Tests
cd typescript && npm run build && node --test tests/index.test.ts

# 4. Rust Core Tests & Build
cd rust && cargo test && cargo build --release

# 5. Python Tests
python3 python/tests/test_thaibreak.py
```

---

## เอกสารอ้างอิง

- **Royal Institute Dictionary:** พจนานุกรมฉบับราชบัณฑิตยสถาน (Public Domain Standard Thai Wordlist)
- **Thai Character Cluster (TCC):** Theeramunkong et al., *Multi-segmentation for Thai word extraction*, 2000
- **Unicode Line Breaking Algorithm:** [Unicode Standard Annex #14 (UAX #14)](https://www.unicode.org/reports/tr14/)
- **W3C Requirements for Thai Text Layout:** [W3C Working Group Note (tlreq)](https://www.w3.org/TR/tlreq/)
- **License:** Apache-2.0
