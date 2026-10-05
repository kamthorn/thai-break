# ThaiBreak Web API Service (Rust)

บริการ Web API ประสิทธิภาพสูงสำหรับการตัดคำภาษาไทย (Thai Word Segmentation) และการจัดแบ่งบรรทัด (Typographic Line Breaking) พัฒนาด้วยภาษา **Rust** บน **Axum / Tokio** ขับเคลื่อนโดย Core Engine [ThaiBreak](https://github.com/kamthorn/thai-break)

> [!NOTE]
> นี่คือ **ตัวอย่าง** การนำ thai-break ไปทำเป็น HTTP service (อยู่ใน `examples/service/` ของ monorepo [thai-break](https://github.com/kamthorn/thai-break) ใช้ engine ใน `rust/` ของ repo เดียวกันผ่าน path dependency) ยังไม่ได้ปล่อย Docker image หรือรองรับการใช้งานจริง ถ้าต้องการตัดคำในแอปพลิเคชัน แนะนำให้ใช้ library ตรง ๆ (Rust, Go, TypeScript, PHP, Python)

ออกแบบมาเฉพาะสำหรับใช้งานเป็น **Internal Microservice** เบื้องหลัง API Gateway (Kong, Traefik, Nginx, AWS API Gateway, Cloudflare) โดยทำงานแบบ Stateless, Zero-Authen Overhead, ประหยัดหน่วยความจำระดับขีดสุด และตอบสนองระดับ Sub-millisecond

---

## ⚡ คุณสมบัติเด่น (Features)

- 🦀 **พัฒนาด้วย Rust แท้ 100%:** ไม่มี Garbage Collector (GC) ป้องกัน Latency Spikes (P99 / P99.9 latency นิ่งเสถียรที่สุด)
- 💾 **ประหยัดทรัพยากรระดับ Ultra-Low Memory:** ใช้ RAM ประมาณ **7.5 MiB** หลังรับ 38,454 request (วัดด้วย `docker stats` กับ container ที่ไม่ได้ใช้ Bigram Model) ด้วยโครงสร้าง Finite State Transducer (FST) แบบ Zero-Copy Binary
- ⏱️ **ความเร็วสูงระดับไมโครวินาที & Zero Cold-Start:** Cold start < 1 ms และประมวลผลคำเฉลี่ย **~10–30 µs ต่อประโยค** (Total HTTP roundtrip ~1 ms)
- 🛡️ **HTML / XML Preservation:** รักษาโครงสร้างแท็ก HTML/XML (`<p>`, `<b>`, `<span>`), สคริปต์/สไตล์ และ HTML entities (`&amp;`) ให้ปลอดภัย 100% โดยแทรกตัวตัดคำเฉพาะข้อความจริงเท่านั้น
- 📦 **Zero-Config Deployment:** ฝังพจนานุกรม FST แบบ Binary ไว้ในตัว executable ทันที พร้อมรันโดยไม่ต้องพึ่งพาไฟล์ภายนอก
- 🚀 **Dual-Mode Engine (Words vs Lines):** แยกโหมดการตัดคำเชิงความหมาย (Semantic Tokenization) และการตัดคำเพื่อจัดหน้าสิ่งพิมพ์/เว็บ (Typographic Line Breaking) อย่างแม่นยำ
- 🐳 **Docker Multi-Stage Build บน Alpine 3.24:** ขนาด Image **~18 MB** (ใช้ Base `rust:1.98.1-alpine3.24` และ Runtime `alpine:3.24`) รันด้วย User ที่ไม่มีสิทธิ์ Root (Security Hardened) และมี Healthcheck ในตัว

---

## 🚀 เริ่มต้นใช้งานอย่างรวดเร็ว (Quick Start)

### วิธีที่ 1: รันด้วย Docker Compose (แนะนำ)

```bash
cd examples/service
docker compose up -d
```

### วิธีที่ 2: รันด้วย Docker เดี่ยวๆ

Build จากรากของ repository (ต้องใช้ `rust/` เป็น build context):

```bash
docker build -f examples/service/Dockerfile -t thai-break-service .
```

```bash
docker run -d \
  --name thai-break-service \
  -p 8080:8080 \
  thai-break-service:latest
```

ทดสอบการทำงานผ่าน Healthcheck:
```bash
curl http://localhost:8080/health
# ผลลัพธ์: {"status":"ok","timestamp":1790178243}
```

---

## 📖 สเปกของ API (API Reference)

### 1. ตัดคำเป็น Array (`POST /api/v1/words`)
ตัดข้อความภาษาไทยออกเป็นคำๆ ในรูปแบบ Array
(มี alias `POST /api/v1/tokenize` ที่ทำงานเหมือนกันทุกประการ)

#### ตัวอย่าง: ข้อความเดี่ยว (Single text)
```bash
curl -X POST http://localhost:8080/api/v1/words \
  -H "Content-Type: application/json" \
  -d '{
    "text": "ฉันรักภาษาไทย"
  }'
```
**Response:**
```json
{
  "success": true,
  "data": ["ฉัน", "รัก", "ภาษา", "ไทย"]
}
```

#### ตัวอย่าง: ประมวลผลเป็นชุด (Batch texts)
```bash
curl -X POST http://localhost:8080/api/v1/words \
  -H "Content-Type: application/json" \
  -d '{
    "texts": ["สวัสดีครับ", "ยินดีต้อนรับสู่บริการ"]
  }'
```
**Response:**
```json
{
  "success": true,
  "count": 2,
  "data": [
    ["สวัสดี", "ครับ"],
    ["ยินดี", "ต้อนรับ", "สู่", "บริการ"]
  ]
}
```

---

### 2. แทรกจุดตัดบรรทัดสำหรับ HTML / PDF / Web (`POST /api/v1/lines`)
แทรกจุดตัดคำ (Default: Zero-Width Space `\u200B` หรือกำหนดเป็น `<wbr>`, `|`, `&shy;`) ตามมาตรฐาน **W3C Thai Text Layout** และ **Unicode UAX #14** โดยไม่ทำลายโครงสร้าง HTML

```bash
curl -X POST http://localhost:8080/api/v1/lines \
  -H "Content-Type: application/json" \
  -d '{
    "text": "<p>สวัสดี <b>ประเทศไทย</b> &amp; กรุงเทพมหานคร</p>",
    "marker": "|",
    "is_html": true
  }'
```

**Response:**
```json
{
  "success": true,
  "data": "<p>สวัสดี <b>ประเทศ|ไทย</b> &amp; กรุงเทพ|มหา|นคร</p>"
}
```
> **หมายเหตุ:** สังเกตว่าแท็ก `<p>`, `<b>`, `</b>`, `&amp;`, `</p>` จะไม่ถูกแตะต้องหรือถูกแทรกจุดตัดเข้าไปภายในแท็ก

---

### 3. ตัดบรรทัดตามความกว้างหน้าจอ (`POST /api/v1/wrap`)
ตัดขึ้นบรรทัดใหม่ (`\n`) โดยคำนวณตามความกว้างตัวอักษรจริง (Display Width: สระบน-ล่าง/วรรณยุกต์ ไม่นับความกว้าง)

```bash
curl -X POST http://localhost:8080/api/v1/wrap \
  -H "Content-Type: application/json" \
  -d '{
    "text": "ฉันรักภาษาไทยมากที่สุดในโลก",
    "width": 12
  }'
```

**Response:**
```json
{
  "success": true,
  "data": "ฉันรักภาษาไทย\nมากที่สุดในโลก"
}
```

---

### 4. รวมศูนย์คำสั่งเดียว (`POST /api/v1/break`)
Endpoint สารพัดประโยชน์ที่สามารถเลือกโหมดการทำงานได้ผ่านฟิลด์ `mode` (`words`, `lines`, `wrap`, `join`)

#### โหมด `join` (ต่อคำด้วย Delimiter ที่ต้องการ):
```bash
curl -X POST http://localhost:8080/api/v1/break \
  -H "Content-Type: application/json" \
  -d '{
    "mode": "join",
    "text": "กระทรวงดิจิทัลเพื่อเศรษฐกิจและสังคม",
    "delimiter": "|"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": "กระทรวง|ดิจิทัล|เพื่อ|เศรษฐกิจ|และ|สังคม"
}
```

#### พารามิเตอร์ทั้งหมดของ `POST /api/v1/break`:
| พารามิเตอร์ | ชนิดข้อมูล | ค่าเริ่มต้น | คำอธิบาย |
|---|---|---|---|
| `mode` | string | `"words"` | โหมดการทำงาน: `"words"`, `"lines"`, `"wrap"`, หรือ `"join"` |
| `text` | string | `null` | ข้อความเดี่ยวที่ต้องการประมวลผล |
| `texts` | array of strings | `null` | อาเรย์ข้อความสำหรับประมวลผลแบบ Batch |
| `is_html` | boolean | `false` | เปิดใช้การตัดคำแบบรักษาแท็ก HTML/XML และ Entity |
| `marker` | string | `"\u200B"` | สัญลักษณ์ตัดบรรทัด (ใช้ในโหมด `lines`) |
| `delimiter` | string | `"|"` | สัญลักษณ์คั่นคำ (ใช้ในโหมด `join`) |
| `width` | integer | `80` | ความกว้างคอลัมน์สูงสุด (ใช้ในโหมด `wrap`) |
| `keep_whitespace` | boolean | `false` | เก็บช่องว่างไว้เป็นคำในผลลัพธ์หรือไม่ (โหมด `words`) |
| `normalize` | boolean | `false` | ทำความสะอาดข้อความ (สระวิปริต, เสียงลาก, อักขระล่องหน) ก่อนประมวลผล |

---

### 5. ทำความสะอาดข้อความ (`POST /api/v1/normalize`)
จัดระเบียบข้อความภาษาไทยให้ถูกต้องก่อนตัดคำ: รวมสระแอที่พิมพ์ซ้ำ (`เเ` → `แ`), รวมนิคหิต+สระอา (`ํา` → `ำ`), จัดลำดับสระ/วรรณยุกต์, ยุบเสียงลาก (`มากกก` → `มาก`, ตรวจกับพจนานุกรมก่อนยุบกรณีซ้ำ 2 ตัว), ลบ zero-width และช่องว่างหน้าเครื่องหมาย

```bash
curl -X POST http://localhost:8080/api/v1/normalize \
  -H "Content-Type: application/json" \
  -d '{"texts": ["เเปลกมากกก", "นํ้าดื่ม"]}'
```
**Response:**
```json
{
  "success": true,
  "count": 2,
  "data": ["แปลกมาก", "น้ำดื่ม"]
}
```

---

### 6. เปรียบเทียบโหมดตัดคำ (`POST /api/v1/compare`)
ดูผลลัพธ์ Words Mode (ความหมายระดับคำ) เทียบกับ Lines Mode (จัดบรรทัด) พร้อมคำอธิบายในครั้งเดียว

```bash
curl -X POST http://localhost:8080/api/v1/compare \
  -H "Content-Type: application/json" \
  -d '{"text": "เดินทางไปกรมการกงสุลที่เกิดเหตุ", "normalize": false}'
```
**Response:**
```json
{
  "success": true,
  "data": {
    "text": "เดินทางไปกรมการกงสุลที่เกิดเหตุ",
    "words": ["เดินทาง", "ไป", "กรมการกงสุล", "ที่เกิดเหตุ"],
    "lines_break": "เดินทาง|ไป|กรมการ|กงสุล|ที่|เกิด|เหตุ",
    "explanation": "Words mode retains proper names, compounds, and institutions intact for semantic completeness. Lines mode splits them into constituent units for flexible line wrapping."
  }
}
```

---

### 7. แปลงจำนวนเงินเป็นคำอ่าน (`GET/POST /api/v1/bahttext`)
แปลงตัวเลขเป็นคำอ่านบาทไทยตามแบบ Excel BAHTTEXT (รองรับ `amount` ตัวเลข หรือ `text` เช่น `"1,234.50"`)

```bash
curl "http://localhost:8080/api/v1/bahttext?amount=1234.50"
# {"success":true,"data":"หนึ่งพันสองร้อยสามสิบสี่บาทห้าสิบสตางค์"}
```

---

### 8. ตรวจสอบเลขบัตรประชาชน (`GET/POST /api/v1/validate-id`)
ตรวจสอบเลขประจำตัวประชาชน 13 หลักด้วย DOPA checksum (รับได้ทั้งแบบมีขีดและไม่มีขีด)

```bash
curl -X POST http://localhost:8080/api/v1/validate-id \
  -H "Content-Type: application/json" \
  -d '{"id": "1-1007-01234-56-1"}'
```
**Response:**
```json
{
  "success": true,
  "data": {"is_valid": true, "id": "1100701234561", "formatted": "1-1007-01234-56-1", "error": null}
}
```

---

### 9. ข้อมูลและสถานะระบบ (Info & Health)

- `GET /health` หรือ `GET /livez` หรือ `GET /readyz`:
  ```json
  {"status":"ok","timestamp":1790178243}
  ```
- `GET /api/v1/info` หรือ `GET /`:
  ```json
  {
    "name": "thai-break-service",
    "version": "0.1.0",
    "engine": "thaibreak-rust",
    "base_dictionary_words": 41272,
    "words_dictionary_words": 41272,
    "lines_dictionary_words": 33145,
    "status": "ready"
  }
  ```

---

## 📚 พจนานุกรมและโมเดล (Dictionary & Bigram Model)

Service นี้ใช้พจนานุกรม 2 ชั้นรวมกัน แล้วคอมไพล์เป็น FST ไบนารี:

| องค์ประกอบ | ที่มา | ลิขสิทธิ์ | จำนวน |
|---|---|---|---|
| **Base Dictionary** | `data/words.txt` ของ thai-break (ราชบัณฑิตยสถาน) | Public Domain | 25,402 คำ |
| **Extra Dictionary** | [thai-break-dict-extra](https://github.com/kamthorn/thai-break-dict-extra) `dist/words-extra.tsv` | CC0-1.0 | 18,109 คำ |
| **Lines Dictionary** | `dist/words-extra-lines.tsv` (ตัดชื่อเฉพาะ/โรงเรียน/คำประสมข่าวออกเพื่อให้พับบรรทัดได้ยืดหยุ่น) | CC0-1.0 | 9,218 คำ |
| **Bigram Model** (ไม่บังคับ ไม่อยู่ใน repo) | `thai-break-dict-extra/data/bigrams-prachathai.tsv` สกัดจาก Prachathai-67k (54,380 บทความ) | Apache-2.0 | 955,967 คู่คำ |

**ผลลัพธ์หลังรวม:** `data/words.fst` = **41,272 คำ** (918 KB), `data/lines.fst` = **33,145 คำ** (700 KB)

### การรีบิลด์พจนานุกรม
```bash
# 1. รีบิลด์พจนานุกรมเสริม (ถ้ามีการแก้คำใน thai-break-dict-extra ซึ่งต้อง clone ไว้ข้าง ๆ repo นี้)
cd ../thai-break-dict-extra
python3 scripts/build.py --compile-fst --compile-dawg

# 2. รวม Base + Extra แล้วคอมไพล์เป็น FST สำหรับ Service (รันจาก examples/service)
cd ../thai-break/examples/service
python3 scripts/compile_dicts.py

# 3. รีบิลด์ไบนารี (FST ถูกฝังใน executable)
cargo build --release
```

### Bigram Model และค่า Alpha
Bigram Model **ไม่อยู่ใน repo** (ไฟล์ 30 MB และให้ผลแย่ลงเล็กน้อยในการวัดล่าสุด ดูตารางด้านล่าง) ถ้าต้องการใช้ ให้คัดลอก `thai-break-dict-extra/data/bigrams-prachathai.tsv` มาเป็น `examples/service/data/bigrams.tsv` (ไฟล์นี้ถูก gitignore) หรือส่ง path ผ่าน `THAIBREAK_BIGRAM_PATH`

เมื่อมีไฟล์ Service โหลด Bigram Model ด้วยค่า **`alpha = 2.0`** (ไม่ใช่ค่า default 0.15 ของไลบรารี) พร้อมสูตร `BigramFormula::Count`:

$$\text{bonus} = \frac{\alpha \cdot \ln(1 + \text{count})}{\max(1, \text{word\_len})}$$

- ถ้าไม่ระบุ `--bigram-path` Service จะค้นหา `data/bigrams.tsv` ให้อัตโนมัติ
- Bigram จาก **LST20** (NECTEC Data Agreement) เป็นไฟล์สำหรับวัดผลเท่านั้น เก็บไว้ที่ `local/bigrams-lst20.tsv` (gitignored) และ **ห้าม commit** — ต้องส่ง path ชัดเจนถ้าต้องการใช้
- Bigram จาก **Wisesight** (CC0-1.0, คนตัดคำเอง 1,153 ประโยค) อยู่ใน `thai-break-dict-extra/data/bigrams-wisesight.tsv` ใช้เป็นทางเลือกที่ปลอดภัยทางลิขสิทธิ์

### ความแม่นยำ (วัดเมื่อ 2026-10-05)

LST20 test split (38,454 ประโยค, 174,074 คำ) ผ่าน `POST /api/v1/words` ด้วย `tools/benchmark.py --segmenter-cmd` ของ thai-break 1.3.0:

| การตั้งค่า | Word F1 | Boundary F1 |
|---|---|---|
| พจนานุกรมรวม ไม่มี Bigram (ค่าเริ่มต้นของตัวอย่างนี้) | 83.7 | 92.0 |
| พจนานุกรมรวม + Bigram Prachathai (alpha 2.0) | 83.3 | 91.7 |
| (เทียบ) พจนานุกรมฐานอย่างเดียว ไม่มี Bigram | 86.1 | 93.3 |

- Bigram ลดคะแนนเล็กน้อยบน LST20 กับเอนจินรุ่นนี้ (ค่า alpha 2.0 ปรับไว้กับเอนจินรุ่นก่อน) จึงไม่เปิดเป็นค่าเริ่มต้น
- พจนานุกรมรวมได้คะแนนระดับคำต่ำกว่าฐานอย่างเดียว เพราะ dict-extra มีคำประสมและชื่อเฉพาะที่ gold ของ LST20 แยกเป็นคำย่อย นี่เป็นการออกแบบที่ตั้งใจให้ค้นหาคำประสมได้ตรงตัว ดู `ROADMAP.md` ของ repo

> [!NOTE]
> การเพิ่มคำในพจนานุกรมเป็นปัจจัยที่เพิ่มความแม่นยำมากที่สุด (คำที่ตัดผิดส่วนใหญ่ไม่มีในพจนานุกรม) ส่วนน้ำหนักคำ (Tier Weight) มีผลรองลงมา

---

## ⚙️ การตั้งค่าระบบ (Configuration)

สามารถตั้งค่าผ่าน Environment Variables หรือ CLI arguments ได้ดังนี้:

| Environment Variable | ค่าเริ่มต้น | คำอธิบาย |
|---|---|---|
| `HOST` | `0.0.0.0` | IP Address ที่เปิดรับ Connection |
| `PORT` | `8080` | Port ที่เปิดให้บริการ |
| `THAIBREAK_WORDS_FST_PATH` | `/app/data/words.fst` | ที่อยู่ไฟล์ FST สำหรับโหมดคำ (words) |
| `THAIBREAK_LINES_FST_PATH` | `/app/data/lines.fst` | ที่อยู่ไฟล์ FST สำหรับโหมดบรรทัด (lines) |
| `THAIBREAK_BIGRAM_PATH` | `data/bigrams.tsv` (auto) | ที่อยู่ไฟล์ Bigram Model `bigrams.tsv` (ถ้าไม่ระบุจะค้นหา `data/bigrams.tsv` ให้อัตโนมัติ) |
| `MAX_BODY_MB` | `4` | ขนาด Request Body สูงสุด (MB) |
| `CONCURRENCY_LIMIT` | `200` | จำนวน Request ที่ประมวลผลพร้อมกันสูงสุด |
| `REQUEST_TIMEOUT_SECS` | `30` | Timeout ต่อ Request (วินาที) |
| `RUST_LOG` | `info` | ระดับการบันทึก Log (`trace`, `debug`, `info`, `warn`, `error`) |

#### การใช้พจนานุกรม FST กำหนดเอง (Custom FST Dictionary):
คุณสามารถ Mount ไฟล์ `.fst` เข้ามาแทนที่ได้ทันที:
```bash
docker run -d \
  -v /path/to/my-words.fst:/custom/words.fst:ro \
  -e THAIBREAK_WORDS_FST_PATH=/custom/words.fst \
  -p 8080:8080 \
  thai-break-service:latest
```

---

## 🌐 การเชื่อมต่อกับ API Gateway (API Gateway Integration)

เนื่องจาก Service นี้ถูกออกแบบมาให้เป็น **Stateless Core Processor** จึงไม่ต้องเสีย CPU ในการคำนวณ Token Authen ภายในตัว แต่ให้ API Gateway ทำหน้าที่แทน:

### ตัวอย่าง: Nginx Reverse Proxy
```nginx
location /thai-break/ {
    proxy_pass http://thai-break-service:8080/;
    proxy_set_header Host $host;
    proxy_set_header X-Real-IP $remote_addr;
    proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    
    # เปิด Keepalive และ Buffer สำหรับความเร็วสูงสุด
    proxy_http_version 1.1;
    proxy_set_header Connection "";
}
```

### ตัวอย่าง: Traefik (Docker Compose Labels)
```yaml
labels:
  - "traefik.enable=true"
  - "traefik.http.routers.thaibreak.rule=PathPrefix(`/api/v1/break`, `/api/v1/words`, `/api/v1/lines`)"
  - "traefik.http.routers.thaibreak.entrypoints=web"
  - "traefik.http.services.thaibreak.loadbalancer.server.port=8080"
```

---

## 🛠️ การคอมไพล์และทดสอบโค้ดในเครื่อง (Development)

```bash
# ติดตั้ง Rust (ถ้ายังไม่มี): https://rustup.rs
# รันจาก examples/service
# 1. รัน Unit & Integration Tests ทั้งหมด
#    (ใช้ `cargo test` แบบ debug; `cargo test --release` บน target ที่ยังไม่เคย build จะ compile ไม่ผ่าน
#     เพราะ crate thaibreak มีหลาย crate-type ให้รัน `cargo build --release` ก่อน)
cargo test

# 2. รัน Service ในโหมดพัฒนา
cargo run

# 3. คอมไพล์แบบ Optimize ขั้นสูงสุด
cargo build --release
```

---

## 📄 ลิขสิทธิ์ (License)

Apache-2.0 License — คลังพจนานุกรมและซอร์สโค้ดสะอาด 100% ใช้งานเชิงพาณิชย์ได้อย่างเสรี
