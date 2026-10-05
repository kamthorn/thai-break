<?php
declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\TestCase;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\ThaiTrie;
use ThaiBreak\DictionaryLoader;
use ThaiBreak\ThaiBreak;

class PHPUnitTokenizerTest extends TestCase
{
    private ThaiTokenizer $tok;
    private ThaiTokenizer $defaultTok;

    protected function setUp(): void
    {
        $this->tok = ThaiTokenizer::fromArray(['ฉัน' => 9000.0, 'รัก' => 8000.0, 'ภาษา' => 7000.0, 'ไทย' => 9000.0, 'สวัสดี' => 9000.0, 'ครับ' => 8500.0]);
        $this->defaultTok = ThaiTokenizer::withDefaultDict();
    }

    public function testSimpleSentence(): void
    {
        $this->assertSame(['ฉัน','รัก','ภาษา','ไทย'], $this->tok->tokenize('ฉันรักภาษาไทย'));
    }

    public function testGreeting(): void
    {
        $this->assertSame(['สวัสดี','ครับ'], $this->tok->tokenize('สวัสดีครับ'));
    }

    public function testHighWeightWordWins(): void
    {
        $tok = ThaiTokenizer::fromArray(['ตา' => 3000.0, 'กลม' => 3000.0, 'ตากลม' => 9999.0]);
        $this->assertSame(['ตากลม'], $tok->tokenize('ตากลม'));
    }

    public function testTwoHighWeightWordsBeatLowCompound(): void
    {
        $tok = ThaiTokenizer::fromArray(['ของ' => 9000.0, 'ดี' => 9000.0, 'ของดี' => 100.0]);
        $this->assertSame(['ของ','ดี'], $tok->tokenize('ของดี'));
    }

    public function testDefaultDictTokenizes(): void
    {
        $result = $this->defaultTok->tokenize('ฉันรักภาษาไทย');
        $this->assertNotEmpty($result);
        $this->assertSame('ฉันรักภาษาไทย', implode('', $result));
    }

    public function testMixedThaiEnglish(): void
    {
        $result = $this->defaultTok->tokenize('ใช้ PHP 8.3');
        $this->assertNotEmpty($result);
    }

    public function testEmptyString(): void
    {
        $this->assertSame([], $this->tok->tokenize(''));
    }

    public function testTokenizeToString(): void
    {
        $result = $this->tok->tokenizeToString('ฉันรักภาษาไทย', '|');
        $this->assertSame('ฉัน|รัก|ภาษา|ไทย', $result);
    }

    public function testInsertLineBreaks(): void
    {
        $result = $this->defaultTok->insertLineBreaks('ฉันรักภาษาไทย');
        $this->assertStringContainsString("\u{200B}", $result);
        $this->assertSame('ฉันรักภาษาไทย', str_replace("\u{200B}", '', $result));
    }

    public function testLineBreaksDoNotAttachToSpaces(): void
    {
        $result = $this->defaultTok->insertLineBreaks('ฉัน รัก ภาษา ไทย');
        $this->assertStringNotContainsString(" \u{200B}", $result);
        $this->assertStringNotContainsString("\u{200B} ", $result);
    }

    public function testLineBreaksKeepLatinLettersAndDigitsTogether(): void
    {
        // UAX #14 LB23: no break between letters and digits.
        $result = $this->defaultTok->insertLineBreaks('ตาม WP01 และมาตรฐาน ISO29110', '|');
        $this->assertStringContainsString('WP01', $result);
        $this->assertStringContainsString('ISO29110', $result);
        $this->assertStringContainsString('และ|มาตรฐาน', $result);
    }

    public function testLineBreaksNeverBreakBeforeASolidus(): void
    {
        // UAX #14 LB13: "ISO/IEC" must not become "ISO" + "/IEC" across lines.
        $result = $this->defaultTok->insertLineBreaks('ISO/IEC 29110 กำหนดให้มี', '|');
        $this->assertStringNotContainsString('|/', $result);
    }

    public function testLineBreaksHtmlProtection(): void
    {
        $html = '<p class="lead">สวัสดีครับ</p>';
        $result = $this->defaultTok->insertLineBreaks($html, "\u{200B}", true);
        $this->assertStringContainsString('<p class="lead">', $result);
        $this->assertStringContainsString('</p>', $result);
    }

    public function testLineBreaksHtmlRawProtection(): void
    {
        $html = '<p>สวัสดีครับ</p><script>var x = "สวัสดีครับ";</script><!-- หมายเหตุ -->';
        $result = $this->defaultTok->insertLineBreaks($html, '|', true);
        $this->assertStringContainsString('<script>var x = "สวัสดีครับ";</script>', $result);
        $this->assertStringContainsString('<!-- หมายเหตุ -->', $result);
    }


    public function testWrapText(): void
    {
        $wrapped = $this->defaultTok->wrap('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย', 20);
        $this->assertStringContainsString("\n", $wrapped);
        $this->assertSame('ประเทศไทยมีวัฒนธรรมที่สวยงามและหลากหลาย', str_replace("\n", '', $wrapped));
    }

    public function testBoundariesCoverTheText(): void
    {
        // bytes for substr(), code points for mb_substr(); the emoji is 4 bytes
        $text = 'ฉันรักภาษาไทย 😀 ครับ';
        $bytes = ThaiBreak::boundaries($text);
        $chars = ThaiBreak::boundaries($text, true);
        $this->assertSame(0, $bytes[0]);
        $this->assertSame(strlen($text), end($bytes));
        $this->assertSame(mb_strlen($text, 'UTF-8'), end($chars));
        $segments = [];
        for ($i = 0; $i + 1 < count($bytes); $i++) {
            $segments[] = substr($text, $bytes[$i], $bytes[$i + 1] - $bytes[$i]);
            $this->assertSame(
                end($segments),
                mb_substr($text, $chars[$i], $chars[$i + 1] - $chars[$i], 'UTF-8')
            );
        }
        $this->assertSame($text, implode('', $segments));
        $this->assertSame(
            ThaiBreak::words($text),
            array_values(array_filter($segments, fn ($s) => trim($s) !== ''))
        );
        $this->assertSame([0], ThaiBreak::boundaries(''));
        $legacy = 'นํ้าตาล';
        $legacyBounds = ThaiBreak::boundaries($legacy);
        $this->assertSame(strlen($legacy), end($legacyBounds));
    }
}
