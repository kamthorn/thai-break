<?php

declare(strict_types=1);

/**
 * ThaiBreak (thai-break) — Fast Thai Word Segmenter & Typographic Line Breaker
 * Demonstration Script
 *
 * แสดงการทำงานของ ThaiBreak ในสถานการณ์ต่าง ๆ
 */

require_once __DIR__ . '/../src/ThaiTrie.php';
require_once __DIR__ . '/../src/ThaiTCC.php';
require_once __DIR__ . '/../src/BigramModel.php';
require_once __DIR__ . '/../src/WeightedTokenizer.php';
require_once __DIR__ . '/../src/DictionaryLoader.php';
require_once __DIR__ . '/../src/ThaiTokenizer.php';
require_once __DIR__ . '/../src/ThaiLineBreaker.php';
require_once __DIR__ . '/../src/ThaiBreak.php';

use ThaiBreak\ThaiBreak;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\DictionaryLoader;
use ThaiBreak\ThaiLineBreaker;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

function section(string $title): void
{
    echo "\n\033[1;36m━━━ $title ━━━\033[0m\n\n";
}

function demo(string $text, array $tokens, ?float $ms = null): void
{
    $display = "\033[33m" . implode(' | ', $tokens) . "\033[0m";
    $timing  = $ms !== null ? sprintf("  \033[90m(%.1f ms)\033[0m", $ms) : '';
    echo "  Input : \033[37m$text\033[0m\n";
    echo "  Output: $display$timing\n\n";
}

// ─────────────────────────────────────────────────────────────────────────────
echo "\033[1;37m╔══════════════════════════════════════════════════════════════╗\033[0m\n";
echo "\033[1;37m║      ThaiBreak (thai-break) — Demonstration Script           ║\033[0m\n";
echo "\033[1;37m╚══════════════════════════════════════════════════════════════╝\033[0m\n";

// ─────────────────────────────────────────────────────────────────────────────
section('1. Basic Tokenization (Default Dictionary)');
// ─────────────────────────────────────────────────────────────────────────────

$tokenizer = ThaiTokenizer::withDefaultDict();

$sentences = [
    'ฉันรักภาษาไทย',
    'สวัสดีครับ',
    'ประเทศไทยมีวัฒนธรรมที่สวยงาม',
    'วันนี้อากาศดีมาก',
    'ฉันกินข้าวที่ร้านอาหาร',
];

foreach ($sentences as $s) {
    $t0 = microtime(true);
    $tokens = $tokenizer->tokenize($s);
    $ms = (microtime(true) - $t0) * 1000;
    demo($s, $tokens, $ms);
}

// ─────────────────────────────────────────────────────────────────────────────
section('2. Weight Preference — สาธิตผลของ Weight');
// ─────────────────────────────────────────────────────────────────────────────

echo "  \033[90m[กรณีที่ 1] คำเดียวที่มี weight สูงชนะคู่คำที่มี weight ต่ำ\033[0m\n\n";

// ตากลม: ['ตา','กลม'] vs ['ตากลม']
$trieA = DictionaryLoader::fromArray(['ตา' => 3000.0, 'กลม' => 3000.0, 'ตากลม' => 100.0]);
$tokA  = (new ThaiTokenizer($trieA))->tokenize('ตากลม');
echo "  Dict: ตา(3000) + กลม(3000) vs ตากลม(100)\n";
demo('ตากลม', $tokA);

$trieB = DictionaryLoader::fromArray(['ตา' => 3000.0, 'กลม' => 3000.0, 'ตากลม' => 9999.0]);
$tokB  = (new ThaiTokenizer($trieB))->tokenize('ตากลม');
echo "  Dict: ตา(3000) + กลม(3000) vs ตากลม(9999)\n";
demo('ตากลม', $tokB);

echo "  \033[90m[กรณีที่ 2] Custom Domain Word — เพิ่ม weight สูงแบบ runtime\033[0m\n\n";

$tokenizer2 = ThaiTokenizer::withDefaultDict();
$tok1 = $tokenizer2->tokenize('อัลตราซาวด์');
echo "  Before injection:\n";
demo('อัลตราซาวด์', $tok1);

$tokenizer2->addWord('อัลตราซาวด์', 99000.0);
$tok2 = $tokenizer2->tokenize('อัลตราซาวด์');
echo "  After addWord('อัลตราซาวด์', 99000.0):\n";
demo('อัลตราซาวด์', $tok2);

// ─────────────────────────────────────────────────────────────────────────────
section('3. Mixed Thai-English-Number Text');
// ─────────────────────────────────────────────────────────────────────────────

$mixed = [
    'ใช้ PHP 8.3 พัฒนาเว็บแอปพลิเคชัน',
    'มีสมาชิก 1,234 คนในกลุ่ม',
    'download ไฟล์ขนาด 100MB',
    'AI และ machine learning กำลังเติบโต',
];

foreach ($mixed as $s) {
    demo($s, $tokenizer->tokenize($s));
}

// ─────────────────────────────────────────────────────────────────────────────
section('4. Domain-Specific Dictionary — พจนานุกรมเฉพาะสาขา');
// ─────────────────────────────────────────────────────────────────────────────

echo "  \033[90mสร้าง tokenizer สำหรับสาขาการแพทย์\033[0m\n\n";

$medTokenizer = ThaiTokenizer::withDefaultDict();
$medTokenizer->addCustomWords([
    'ยาปฏิชีวนะ'  => 50000.0,
    'การผ่าตัด'   => 48000.0,
    'โรคมะเร็ง'   => 46000.0,
    'เลือดออก'    => 44000.0,
    'ปวดหัว'      => 42000.0,
    'ความดันโลหิต' => 40000.0,
    'คลื่นไส้'    => 38000.0,
    'อาการ'        => 36000.0,
    'ผู้ป่วย'     => 34000.0,
    'แพทย์'       => 32000.0,
]);

$medTexts = [
    'ผู้ป่วยมีอาการปวดหัวและคลื่นไส้',
    'แพทย์สั่งยาปฏิชีวนะให้ผู้ป่วย',
    'การผ่าตัดโรคมะเร็งใช้เวลานาน',
];

foreach ($medTexts as $s) {
    demo($s, $medTokenizer->tokenize($s));
}

// ─────────────────────────────────────────────────────────────────────────────
section('5. Whitespace Mode Comparison');
// ─────────────────────────────────────────────────────────────────────────────

$wsText = 'ดี มาก ครับ';
$tok_no_ws   = $tokenizer->tokenize($wsText, false);
$tok_with_ws = $tokenizer->tokenize($wsText, true);

echo "  Input: '$wsText'\n\n";
echo "  keepWhitespace=false: [" . implode(', ', array_map(fn($t) => "\"$t\"", $tok_no_ws)) . "]\n";
echo "  keepWhitespace=true:  [" . implode(', ', array_map(fn($t) => "\"$t\"", $tok_with_ws)) . "]\n";

// ─────────────────────────────────────────────────────────────────────────────
section('6. Performance Test');
// ─────────────────────────────────────────────────────────────────────────────

$sizes = [100, 500, 1000, 2000];
printf("  %-10s %-10s %-12s\n", 'Chars', 'Tokens', 'Time (ms)');
echo "  " . str_repeat('─', 35) . "\n";

foreach ($sizes as $size) {
    $text  = mb_substr(str_repeat('ฉันรักภาษาไทยและวัฒนธรรมไทย', (int)ceil($size / 27)), 0, $size);
    $t0    = microtime(true);
    $toks  = $tokenizer->tokenize($text);
    $ms    = (microtime(true) - $t0) * 1000;
    printf("  %-10d %-10d %.2f\n", mb_strlen($text), count($toks), $ms);
}

// ─────────────────────────────────────────────────────────────────────────────
section('7. Line Breaking & Text Wrapping (PDF & Typographic Layout)');
// ─────────────────────────────────────────────────────────────────────────────

$reportText = 'บริษัท ผลิตภัณฑ์คอนกรีต จำกัด (มหาชน) ได้รายงานผลการดำเนินงานประจำปี 2567 ต่อที่ประชุมสามัญผู้ถือหุ้น โดยมียอดขายรวม ฿1,250,000,000 (เพิ่มขึ้น 8.5%) และมีกำไรสุทธิเพิ่มขึ้นอย่างมีนัยสำคัญ';

echo "  \033[90m[7.1] Insert Zero-Width Space (U+200B) for PDF engines (dompdf, mPDF, TCPDF)\033[0m\n";
$zwsp = ThaiBreak::lines($reportText);
echo "  ZWSP Preview (marked with |) :\n  \033[33m" . str_replace("\u{200B}", "|", $zwsp) . "\033[0m\n\n";

echo "  \033[90m[7.2] HTML Safety (preserving tags <...> and entities &...;)\033[0m\n";
$htmlDoc = '<div class="card"><p><b>ข่าวประชาสัมพันธ์:</b> ยอดขายเติบโต &amp; ขยายสาขาใหม่ๆ ทั่วประเทศ</p></div>';
$zwspHtml = ThaiBreak::lines($htmlDoc, "|", true);
echo "  HTML Output (marked with |)  :\n  \033[32m" . $zwspHtml . "\033[0m\n\n";

echo "  \033[90m[7.3] Soft Wrapping (45 display columns, no mid-word cuts, no orphan punctuation)\033[0m\n";
$wrapped = ThaiBreak::wrap($reportText, 45);
$lines   = explode("\n", $wrapped);
foreach ($lines as $i => $line) {
    $w = ThaiBreak::displayWidth($line);
    printf("  Line %d [%2d cols]: \033[36m%s\033[0m\n", $i + 1, $w, $line);
}

echo "\n";
echo "\033[1;32m✅ Demo completed successfully!\033[0m\n\n";
