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
            'ties keep the earlier word whole' => ['บอกว่าอึดอัด', 'บอก|ว่า|อึดอัด'],
            'ties keep the earlier word whole (2)' => ['ลาออกจากรองประธาน', 'ลาออก|จาก|รอง|ประธาน'],
        ];
    }

    public function testLongTextIsSegmentedToTheEnd(): void
    {
        // Longer than the 50,000-edge limit that used to leave the rest of the text as one token
        $sentence = 'การประชุมสามัญผู้ถือหุ้นประจำปีจัดขึ้นที่โรงแรมในกรุงเทพมหานคร';
        $words    = ThaiTokenizer::getDefault()->tokenize(str_repeat($sentence, 2000));
        $this->assertCount(2000 * count(ThaiTokenizer::getDefault()->tokenize($sentence)), $words);
        $this->assertLessThan(20, max(array_map('mb_strlen', $words)));
    }

    #[DataProvider('segmentationCases')]
    public function testTokenize(string $input, string $expected): void
    {
        $this->assertSame($expected, implode('|', ThaiTokenizer::getDefault()->tokenize($input)));
    }
}
