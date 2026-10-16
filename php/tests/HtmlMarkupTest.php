<?php

declare(strict_types=1);

namespace ThaiBreak\Tests;

use PHPUnit\Framework\TestCase;
use ThaiBreak\ThaiLineBreaker;

class HtmlMarkupTest extends TestCase
{
    /** The pattern that ThaiLineBreaker::splitMarkup() replaces */
    private const PAT_HTML_TAGS = '/(<!--.*?-->|<script\b[^>]*>.*?<\/script>|<style\b[^>]*>.*?<\/style>|<[a-zA-Z\/!?][^>]*>|&[a-zA-Z0-9#]+;)/usi';

    public function testSplitMarkupMatchesThePattern(): void
    {
        $pieces = [
            '<', '>', '!', '-', '/', '?', '<!--', '-->', '<script', '</script>', '<SCRIPT', '</Script>',
            '<style', '</STYLE>', '<scripts', '<script_', '<scriptก', ' ', "\n", 'a', 'Z', '&', '#', ';', '&amp;',
            'ก', 'ไทย', '<b>', '</b>', '<ab',
        ];
        mt_srand(42);
        for ($n = 0; $n < 20000; $n++) {
            $text = '';
            for ($k = mt_rand(0, 11); $k > 0; $k--) {
                $text .= $pieces[mt_rand(0, count($pieces) - 1)];
            }
            $want = preg_split(self::PAT_HTML_TAGS, $text, -1, PREG_SPLIT_DELIM_CAPTURE);
            $this->assertSame($want, ThaiLineBreaker::splitMarkup($text), json_encode($text, JSON_UNESCAPED_UNICODE));
        }
    }

    public function testInvalidUtf8IsOnePart(): void
    {
        $this->assertSame(["<b>\xff</b>"], ThaiLineBreaker::splitMarkup("<b>\xff</b>"));
    }

    public function testUnterminatedMarkupTakesLinearTime(): void
    {
        foreach (['<script>', '<style x>', '<!-->', '<a', '<!--<script>'] as $piece) {
            $text = str_repeat($piece, 200000);
            $start = microtime(true);
            ThaiLineBreaker::splitMarkup($text);
            $this->assertLessThan(2.0, microtime(true) - $start, $piece);
        }
    }
}
