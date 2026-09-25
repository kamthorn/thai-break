<?php
declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\Attributes\DataProvider;
use PHPUnit\Framework\TestCase;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\Uax14;

class LineBreakerTest extends TestCase
{
    /** @return array<string, array{string, string}> Input and expected output with '|' as the break marker */
    public static function lineBreakCases(): array
    {
        return [
            'LB9 combining mark stays with its base' => ["สวัสดี\u{0301}ครับ", "สวัสดี\u{0301}|ครับ"],
            'LB21 no break before a hyphen' => ['สี-ขาว', 'สี-|ขาว'],
            'LB21 no break before an en dash (BA)' => ['ข้อความ–ข้อความ', 'ข้อความ–|ข้อความ'],
        ];
    }

    #[DataProvider('lineBreakCases')]
    public function testInsertLineBreaks(string $input, string $expected): void
    {
        $this->assertSame($expected, ThaiTokenizer::getDefault()->insertLineBreaks($input, '|'));
    }

    public function testMandatoryBreaks(): void
    {
        $cps = array_map('mb_ord', mb_str_split("a\r\nb"));
        $this->assertSame(
            [Uax14::NO_BREAK, Uax14::NO_BREAK, Uax14::NO_BREAK, Uax14::MANDATORY, Uax14::MANDATORY],
            Uax14::breakOpportunities($cps)
        );
    }
}
