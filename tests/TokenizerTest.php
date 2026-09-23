<?php

declare(strict_types=1);

/**
 * ThaiBreak (thai-break) - Fast Thai Word Segmenter & Typographic Line Breaker
 * Test Suite
 *
 * Covers:
 *   1. Basic Thai tokenization
 *   2. Weight preference (high-weight word wins over concatenation)
 *   3. Unknown word handling
 *   4. Mixed Thai-English-number text
 *   5. Whitespace handling
 *   6. Custom dictionary injection
 *   7. TCC boundary safety (no mid-syllable splits)
 *   8. Long text performance
 */

require_once __DIR__ . '/../src/ThaiTrie.php';
require_once __DIR__ . '/../src/ThaiTCC.php';
require_once __DIR__ . '/../src/BigramModel.php';
require_once __DIR__ . '/../src/WeightedTokenizer.php';
require_once __DIR__ . '/../src/DictionaryLoader.php';
require_once __DIR__ . '/../src/ThaiTokenizer.php';
require_once __DIR__ . '/../src/ThaiLineBreaker.php';
require_once __DIR__ . '/../src/ThaiBreak.php';

use ThaiBreak\ThaiTokenizer;
use ThaiBreak\ThaiTrie;
use ThaiBreak\DictionaryLoader;
use ThaiBreak\ThaiLineBreaker;
use ThaiBreak\ThaiBreak;

// -----------------------------------------------------------------------
// Simple test harness
// -----------------------------------------------------------------------

$passed = 0;
$failed = 0;
$tests  = [];

function test(string $name, callable $fn): void
{
    global $passed, $failed, $tests;
    try {
        $result = $fn();
        if ($result === true) {
            $passed++;
            $tests[] = ['PASS', $name, ''];
        } else {
            $failed++;
            $tests[] = ['FAIL', $name, $result];
        }
    } catch (\Throwable $e) {
        $failed++;
        $tests[] = ['ERROR', $name, $e->getMessage()];
    }
}

function assertEqual(array $expected, array $actual, string $context = ''): bool|string
{
    if ($expected === $actual) {
        return true;
    }
    return sprintf(
        "%sExpected: [%s]\n        Got:      [%s]",
        $context ? "$context\n        " : '',
        implode(', ', $expected),
        implode(', ', $actual)
    );
}

// -----------------------------------------------------------------------
// TEST 1: ThaiTrie — basic operations
// -----------------------------------------------------------------------

test('Trie: add and has word', function () {
    $trie = new ThaiTrie();
    $trie->add('สวัสดี', 100.0);
    return $trie->has('สวัสดี')
        ? true
        : 'Word not found in trie';
});

test('Trie: weight stored correctly', function () {
    $trie = new ThaiTrie();
    $trie->add('ครับ', 5000.0);
    $w = $trie->getWeight('ครับ');
    return abs($w - 5000.0) < 0.001
        ? true
        : "Expected weight 5000.0, got $w";
});

test('Trie: keep higher weight on duplicate', function () {
    $trie = new ThaiTrie();
    $trie->add('คน', 100.0);
    $trie->add('คน', 9000.0);
    $w = $trie->getWeight('คน');
    return abs($w - 9000.0) < 0.001
        ? true
        : "Expected weight 9000.0, got $w";
});

test('Trie: prefixes finds all matches', function () {
    $trie = new ThaiTrie();
    $trie->add('ฉัน', 50.0);
    $trie->add('ฉันรัก', 30.0);
    $trie->add('รัก', 80.0);
    $prefixes = $trie->prefixes('ฉันรักคุณ');
    $words = array_column($prefixes, 'word');
    sort($words);
    return in_array('ฉัน', $words, true) && in_array('ฉันรัก', $words, true)
        ? true
        : 'Expected ฉัน and ฉันรัก in prefixes, got: ' . implode(', ', $words);
});

test('Trie: addMany with associative array', function () {
    $trie = new ThaiTrie();
    $trie->addMany(['ดี' => 200.0, 'มาก' => 150.0]);
    return $trie->has('ดี') && $trie->has('มาก')
        ? true
        : 'Words not found after addMany';
});

// -----------------------------------------------------------------------
// TEST 2: Basic tokenization with known words
// -----------------------------------------------------------------------

test('Tokenize: simple sentence', function () {
    $trie = DictionaryLoader::fromArray([
        'ฉัน'  => 5000.0,
        'รัก'  => 4000.0,
        'ภาษา' => 3000.0,
        'ไทย'  => 3500.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('ฉันรักภาษาไทย');
    return assertEqual(['ฉัน', 'รัก', 'ภาษา', 'ไทย'], $result);
});

test('Tokenize: greeting', function () {
    $trie = DictionaryLoader::fromArray([
        'สวัสดี' => 5000.0,
        'ครับ'   => 4000.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('สวัสดีครับ');
    return assertEqual(['สวัสดี', 'ครับ'], $result);
});

test('Tokenize: tokenizeToString with pipe delimiter', function () {
    $trie = DictionaryLoader::fromArray([
        'ฉัน' => 5000.0,
        'ดี'  => 4000.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenizeToString('ฉันดี');
    return $result === 'ฉัน|ดี'
        ? true
        : "Expected 'ฉัน|ดี', got '$result'";
});

// -----------------------------------------------------------------------
// TEST 3: Weight preference
// -----------------------------------------------------------------------

test('Weight: high-weight long word wins over short-word combination', function () {
    // 'ตากลม' could be split as ['ตา','กลม'] or kept as ['ตากลม']
    // We give 'ตากลม' a high weight → should prefer it as one word
    $trie = DictionaryLoader::fromArray([
        'ตา'    => 3000.0,
        'กลม'   => 3000.0,
        'ตากลม' => 9000.0,  // higher weight → should win
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('ตากลม');
    return assertEqual(['ตากลม'], $result,
        "High-weight single word should be preferred");
});

test('Weight: two short high-weight words beat one low-weight compound', function () {
    // 'ของดี' could be one word (low weight) or ['ของ','ดี'] (both high weight)
    $trie = DictionaryLoader::fromArray([
        'ของ'   => 9000.0,
        'ดี'    => 9000.0,
        'ของดี' => 100.0,   // low weight → should NOT win
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('ของดี');
    return assertEqual(['ของ', 'ดี'], $result,
        "Two high-weight words should beat one low-weight compound");
});

test('Weight: custom domain word injection overrides base dict', function () {
    $trie = DictionaryLoader::fromArray([
        'คน'   => 5000.0,
        'สาร'  => 3000.0,
        'คนสาร' => 100.0, // low weight compound
    ]);
    $tokenizer = new ThaiTokenizer($trie);

    // Without custom word: 'คนสาร' splits as ['คน','สาร']
    $r1 = $tokenizer->tokenize('คนสาร');

    // Add high-weight custom word
    $tokenizer->addWord('คนสาร', 99000.0);
    $r2 = $tokenizer->tokenize('คนสาร');

    if ($r2 === ['คนสาร']) {
        return true;
    }
    return "After custom inject, expected ['คนสาร'], got: " . implode(', ', $r2);
});

// -----------------------------------------------------------------------
// TEST 4: Unknown word handling
// -----------------------------------------------------------------------

test('Unknown: single unknown character returned as token', function () {
    $trie = DictionaryLoader::fromArray(['ดี' => 1000.0]);
    $tokenizer = new ThaiTokenizer($trie);
    // 'ก' is unknown, 'ดี' is known
    $result = $tokenizer->tokenize('กดี');
    // Should not crash; result should be non-empty
    return count($result) > 0
        ? true
        : 'Expected non-empty result for unknown word';
});

test('Unknown: all unknown text returns some tokens', function () {
    $trie = new ThaiTrie(); // empty trie
    $tokenizer = new ThaiTokenizer($trie);
    $result = $tokenizer->tokenize('กขคงจ');
    return count($result) > 0
        ? true
        : 'Expected non-empty result';
});

// -----------------------------------------------------------------------
// TEST 5: Mixed Thai-English-number text
// -----------------------------------------------------------------------

test('Mixed: Thai + English word', function () {
    $trie = DictionaryLoader::fromArray([
        'คอม'       => 2000.0,
        'พิวเตอร์' => 2000.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('PHP ภาษาโปรแกรม');
    // PHP should be one token (non-Thai)
    $phpToken  = $result[0] ?? '';
    return $phpToken === 'PHP'
        ? true
        : "Expected first token 'PHP', got '$phpToken'";
});

test('Mixed: number embedded in Thai text', function () {
    $trie = DictionaryLoader::fromArray([
        'มี' => 5000.0,
        'คน' => 4500.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('มี10คน');
    // '10' should be its own token
    return in_array('10', $result, true)
        ? true
        : "Expected '10' as token, got: " . implode(', ', $result);
});

// -----------------------------------------------------------------------
// TEST 6: Whitespace handling
// -----------------------------------------------------------------------

test('Whitespace: strip whitespace tokens by default', function () {
    $trie = DictionaryLoader::fromArray([
        'ดี' => 1000.0,
        'มาก' => 800.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('ดี มาก', false);
    // Should not have pure-space token in result
    $hasSpace = in_array(' ', $result, true) || in_array('  ', $result, true);
    return !$hasSpace
        ? true
        : 'Unexpected whitespace token in result: ' . implode('|', $result);
});

test('Whitespace: keep whitespace when flag set', function () {
    $trie = DictionaryLoader::fromArray([
        'ดี'  => 1000.0,
        'มาก' => 800.0,
    ]);
    $tokenizer = new ThaiTokenizer($trie);
    $result    = $tokenizer->tokenize('ดี มาก', true);
    $hasSpace  = false;
    foreach ($result as $tok) {
        if (trim($tok) === '') {
            $hasSpace = true;
            break;
        }
    }
    return $hasSpace
        ? true
        : 'Expected whitespace token when keepWhitespace=true, got: ' . implode('|', $result);
});

// -----------------------------------------------------------------------
// TEST 7: DictionaryLoader
// -----------------------------------------------------------------------

test('DictionaryLoader: from text file', function () {
    $path = __DIR__ . '/../data/words.txt';
    if (!file_exists($path)) {
        return 'words.txt not found — skip';
    }
    $trie = DictionaryLoader::fromTextFile($path);
    return $trie->has('ของ') && $trie->has('คน')
        ? true
        : 'Expected common words in default dictionary';
});

test('DictionaryLoader: from text file words count', function () {
    $path = __DIR__ . '/../data/words.txt';
    if (!file_exists($path)) {
        return 'words.txt not found — skip';
    }
    $trie = DictionaryLoader::fromTextFile($path);
    return $trie->count() >= 20000
        ? true
        : "Expected at least 20000 words, got {$trie->count()}";
});

test('DictionaryLoader: fromArray sequential', function () {
    $trie = DictionaryLoader::fromArray(['ฉัน', 'รัก', 'ไทย']);
    return $trie->has('ฉัน') && $trie->has('รัก') && $trie->has('ไทย')
        ? true
        : 'Expected all words in trie';
});

// -----------------------------------------------------------------------
// TEST 8: Full pipeline with default dict
// -----------------------------------------------------------------------

test('Pipeline: withDefaultDict tokenizes common sentence', function () {
    $tokenizer = ThaiTokenizer::withDefaultDict();
    $result    = $tokenizer->tokenize('ฉันรักภาษาไทยมาก');
    return count($result) >= 3
        ? true
        : 'Expected at least 3 tokens, got: ' . implode('|', $result);
});

test('Pipeline: complex sentence with default dict', function () {
    $tokenizer = ThaiTokenizer::withDefaultDict();
    $result    = $tokenizer->tokenize('ประเทศไทยมีวัฒนธรรมที่สวยงาม');
    return count($result) >= 2
        ? true
        : 'Expected at least 2 tokens, got: ' . implode('|', $result);
});

// -----------------------------------------------------------------------
// TEST 9: ThaiLineBreaker & Wrapping (PDF & Typographic Layout)
// -----------------------------------------------------------------------

test('LineBreaker: canBreakBetween typographic rules', function () {
    // Whitespace safety
    if (ThaiLineBreaker::canBreakBetween('ฉัน', ' ') !== false) {
        return 'Should not break before space';
    }
    if (ThaiLineBreaker::canBreakBetween(' ', 'รัก') !== false) {
        return 'Should not break after space';
    }
    // No-break after opening brackets / prefix symbols
    if (ThaiLineBreaker::canBreakBetween('(', 'คำ') !== false) {
        return 'Should not break after opening bracket';
    }
    if (ThaiLineBreaker::canBreakBetween('฿', '100') !== false) {
        return 'Should not break after currency symbol';
    }
    // No-break before closing brackets / punctuation / postfixes
    if (ThaiLineBreaker::canBreakBetween('คำ', ')') !== false) {
        return 'Should not break before closing bracket';
    }
    if (ThaiLineBreaker::canBreakBetween('เร็ว', 'ๆ') !== false) {
        return 'Should not break before Mai Yamok (ๆ)';
    }
    if (ThaiLineBreaker::canBreakBetween('กรุงเทพ', 'ฯ') !== false) {
        return 'Should not break before Paiyannoi (ฯ)';
    }
    if (ThaiLineBreaker::canBreakBetween('ฯลฯ', 'ฯลฯ') !== false) {
        return 'Should not break before Paiyanyai';
    }
    // Valid break between regular Thai words
    if (ThaiLineBreaker::canBreakBetween('ฉัน', 'รัก') !== true) {
        return 'Should allow break between regular words';
    }
    return true;
});

test('LineBreaker: insertLineBreaks with zero-width space', function () {
    $text = 'ฉันรักภาษาไทย';
    $result = ThaiTokenizer::breakLines($text);
    if (!str_contains($result, ThaiLineBreaker::DEFAULT_MARKER)) {
        return 'Expected zero-width spaces in result';
    }
    if (str_replace(ThaiLineBreaker::DEFAULT_MARKER, '', $result) !== $text) {
        return 'Text content was altered unexpectedly';
    }
    return true;
});

test('LineBreaker: never insert break adjacent to spaces', function () {
    $text = 'ฉัน รัก ภาษา ไทย';
    $result = ThaiTokenizer::breakLines($text);
    if (str_contains($result, " " . ThaiLineBreaker::DEFAULT_MARKER) ||
        str_contains($result, ThaiLineBreaker::DEFAULT_MARKER . " ")) {
        return 'Break marker placed next to space: ' . bin2hex($result);
    }
    return true;
});

test('LineBreaker: brackets and postfixes adhere to typographic rules', function () {
    $text = 'เดินเล่นๆ (ที่นี่ๆ) กรุงเทพฯ';
    $result = ThaiTokenizer::breakLines($text);
    $marker = ThaiLineBreaker::DEFAULT_MARKER;
    if (str_contains($result, "($marker")) {
        return 'Break marker placed after opening parenthesis';
    }
    if (str_contains($result, "{$marker})")) {
        return 'Break marker placed before closing parenthesis';
    }
    if (str_contains($result, "{$marker}ๆ")) {
        return 'Break marker placed before Mai Yamok (ๆ)';
    }
    if (str_contains($result, "{$marker}ฯ")) {
        return 'Break marker placed before Paiyannoi (ฯ)';
    }
    return true;
});

test('LineBreaker: HTML tags and entities protected', function () {
    $html = '<div class="content">สวัสดี <b>ชาวโลก</b> &amp; ประเทศไทย</div>';
    $result = ThaiTokenizer::breakLines($html, ThaiLineBreaker::DEFAULT_MARKER, true);
    if (!str_contains($result, '<div class="content">') ||
        !str_contains($result, '<b>') ||
        !str_contains($result, '</b>') ||
        !str_contains($result, '&amp;') ||
        !str_contains($result, '</div>')) {
        return 'HTML tags or entities were damaged';
    }
    if (str_replace(ThaiLineBreaker::DEFAULT_MARKER, '', $result) !== $html) {
        return 'HTML content changed after removing markers';
    }
    return true;
});

test('LineBreaker: HTML raw scripts, styles, and comments preserved untouched', function () {
    $html = '<p>สวัสดีประเทศไทย</p><script>var x = "สวัสดีประเทศไทย";</script><!-- หมายเหตุ -->';
    $result = ThaiTokenizer::breakLines($html, '|', true);
    if (!str_contains($result, '<script>var x = "สวัสดีประเทศไทย";</script>')) {
        return 'Script tag content was modified';
    }
    if (!str_contains($result, '<!-- หมายเหตุ -->')) {
        return 'Comment was modified';
    }
    return true;
});


test('LineBreaker: thaiDisplayWidth correctly ignores combining marks', function () {
    if (ThaiLineBreaker::thaiDisplayWidth('ก') !== 1) {
        return 'Base consonant should be 1 column';
    }
    // ที่ = ท (1) + ี (0) + ่ (0) = 1
    if (ThaiLineBreaker::thaiDisplayWidth('ที่') !== 1) {
        return 'Consonant with above vowel and tone mark should be 1 column, got: ' . ThaiLineBreaker::thaiDisplayWidth('ที่');
    }
    // น้ำ = น (1) + ้ (0) + ำ (1) = 2
    if (ThaiLineBreaker::thaiDisplayWidth('น้ำ') !== 2) {
        return 'น้ำ should be 2 columns, got: ' . ThaiLineBreaker::thaiDisplayWidth('น้ำ');
    }
    return true;
});

test('LineBreaker: wrap text into specified column width', function () {
    $text = 'ประเทศไทยมีวัฒนธรรมที่สวยงามและมีความหลากหลายทางธรรมชาติมากมาย';
    $wrapped = ThaiTokenizer::wrapText($text, 20);
    $lines = explode("\n", $wrapped);
    if (count($lines) < 2) {
        return 'Expected multiple lines, got ' . count($lines);
    }
    foreach ($lines as $i => $line) {
        $w = ThaiLineBreaker::thaiDisplayWidth($line);
        if ($w > 25) { // reasonable bound allowing single word overshoot
            return sprintf('Line %d width %d exceeds margin: "%s"', $i + 1, $w, $line);
        }
    }
    // Content check
    if (str_replace("\n", '', $wrapped) !== $text) {
        return 'Wrapped text content does not match original';
    }
    return true;
});

// -----------------------------------------------------------------------
// TEST 10: Performance (smoke test)
// -----------------------------------------------------------------------

test('Performance: 1000-char Thai text under 2 seconds', function () {
    $tokenizer = ThaiTokenizer::withDefaultDict();
    $text      = str_repeat('ฉันรักภาษาไทยและวัฒนธรรมไทย', 33); // ~990 chars
    $start     = microtime(true);
    $result    = $tokenizer->tokenize($text);
    $elapsed   = microtime(true) - $start;
    if ($elapsed > 2.0) {
        return sprintf('Too slow: %.3f seconds for %d chars', $elapsed, mb_strlen($text));
    }
    return count($result) > 0
        ? true
        : 'Expected non-empty result';
});

test('ThaiBreak: convenience class methods', function () {
    $words = ThaiBreak::words('ฉันรักภาษาไทย');
    if ($words !== ['ฉัน', 'รัก', 'ภาษา', 'ไทย']) {
        return 'ThaiBreak::words failed: ' . implode('|', $words);
    }
    $joined = ThaiBreak::join('สวัสดีครับ', '-');
    if ($joined !== 'สวัสดี-ครับ') {
        return 'ThaiBreak::join failed: ' . $joined;
    }
    $lines = ThaiBreak::lines('ฉันรักภาษาไทย');
    if (!str_contains($lines, ThaiLineBreaker::DEFAULT_MARKER)) {
        return 'ThaiBreak::lines failed';
    }
    $wrapped = ThaiBreak::wrap('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย', 20);
    if (!str_contains($wrapped, "\n")) {
        return 'ThaiBreak::wrap failed';
    }
    return true;
});

// -----------------------------------------------------------------------
// Results
// -----------------------------------------------------------------------

echo "\n";
echo "╔══════════════════════════════════════════════════════════╗\n";
echo "║             ThaiBreak (thai-break) — Test Results        ║\n";
echo "╚══════════════════════════════════════════════════════════╝\n\n";

foreach ($tests as [$status, $name, $detail]) {
    $icon  = match ($status) { 'PASS' => '✅', 'FAIL' => '❌', default => '⚠️ ' };
    $color = match ($status) { 'PASS' => "\033[32m", 'FAIL' => "\033[31m", default => "\033[33m" };
    echo sprintf("%s %s%-6s\033[0m %s\n", $icon, $color, $status, $name);
    if ($detail && $detail !== '') {
        echo "        └─ $detail\n";
    }
}

echo "\n";
echo sprintf(
    "Total: %d tests | \033[32m%d passed\033[0m | \033[31m%d failed\033[0m\n\n",
    $passed + $failed,
    $passed,
    $failed
);

exit($failed > 0 ? 1 : 0);
