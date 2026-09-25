<?php
declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use ThaiBreak\ThaiTokenizer;

class SegmentationTest extends TestCase
{
    /** @return array<string, array{string, string}> Input and expected words joined by '|' */
    public static function segmentationCases(): array
    {
        return [
            'a word before a full stop is not cut into an abbreviation' => ['เขากินข้าว.', 'เขา|กิน|ข้าว|.'],
            'a word before a full stop, then more text' => ['ฉันรักเธอ.ไปเที่ยวกัน', 'ฉัน|รัก|เธอ|.|ไป|เที่ยว|กัน'],
            'abbreviations are still recognized' => ['เมื่อ 5 มิ.ย. ที่ จ.พิษณุโลก', 'เมื่อ|5|มิ.ย.|ที่|จ.|พิษณุโลก'],
            'an abbreviation after a word keeps the word whole' => ['ในเขตจ.พิจิตร', 'ใน|เขต|จ.|พิจิตร'],
            'an abbreviation after a word that ends like one' => ['ในเดือนพ.ย.', 'ใน|เดือน|พ.ย.'],
        ];
    }

    #[DataProvider('segmentationCases')]
    public function testTokenize(string $input, string $expected): void
    {
        $this->assertSame($expected, implode('|', ThaiTokenizer::getDefault()->tokenize($input)));
    }
}
