<?php
declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\TestCase;
use ThaiBreak\Uax14;

class Uax14Test extends TestCase
{
    public function testLineBreakClasses(): void
    {
        $expected = [
            'a' => Uax14::AL, '1' => Uax14::NU, ' ' => Uax14::SP, '-' => Uax14::HY, '–' => Uax14::BA,
            '(' => Uax14::OP, ')' => Uax14::CP, '}' => Uax14::CL, '、' => Uax14::CL, '/' => Uax14::SY,
            '.' => Uax14::IS, '!' => Uax14::EX, '"' => Uax14::QU, '“' => Uax14::QI, '”' => Uax14::QF,
            '$' => Uax14::PR, '฿' => Uax14::PR, '\\' => Uax14::PR, '%' => Uax14::PO, '#' => Uax14::AL,
            'ก' => Uax14::SA, 'ๆ' => Uax14::SA, 'ั' => Uax14::SM, '่' => Uax14::SM, '๐' => Uax14::NU,
            '๚' => Uax14::BA, '中' => Uax14::ID, '가' => Uax14::H2, 'א' => Uax14::HL,
            "\u{200B}" => Uax14::ZW, "\u{00A0}" => Uax14::GL, "\u{200D}" => Uax14::ZWJ, "\u{0301}" => Uax14::CM,
            '😀' => Uax14::ID, "\u{1F1F9}" => Uax14::RI, "\u{10FFFF}" => Uax14::AL,
        ];
        foreach ($expected as $char => $class) {
            $this->assertSame($class, Uax14::lineBreakClass(mb_ord((string) $char)), "class of U+" . dechex(mb_ord((string) $char)));
        }
    }

    /** Official conformance test (SA resolved to AL, no dictionary) */
    public function testLineBreakTestConformance(): void
    {
        $path = __DIR__ . '/../../testdata/LineBreakTest-16.0.0.txt';
        if (!is_file($path)) {
            $this->markTestSkipped('LineBreakTest data not found');
        }

        $failures = [];
        foreach (file($path, FILE_IGNORE_NEW_LINES) ?: [] as $line) {
            if ($line === '' || $line[0] === '#') {
                continue;
            }
            $cps      = [];
            $expected = [];
            foreach (preg_split('/\s+/', trim($line)) ?: [] as $field) {
                match ($field) {
                    '×'     => $expected[] = false,
                    '÷'     => $expected[] = true,
                    default => $cps[] = (int) hexdec($field),
                };
            }
            $actual = array_map(fn(int $a) => $a !== Uax14::NO_BREAK, Uax14::breakOpportunities($cps));
            if ($actual !== $expected) {
                $failures[] = $line;
            }
        }

        $this->assertSame([], array_slice($failures, 0, 10), count($failures) . ' LineBreakTest cases failed');
    }

    public function testFlags(): void
    {
        $this->assertTrue(Uax14::isEastAsian(mb_ord('（')));
        $this->assertFalse(Uax14::isEastAsian(mb_ord('(')));
        $this->assertTrue(Uax14::isExtPictUnassigned(0x1FFFD));
        $this->assertFalse(Uax14::isExtPictUnassigned(mb_ord('😀')));
    }
}
