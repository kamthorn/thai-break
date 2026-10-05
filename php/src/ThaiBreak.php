<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * ThaiBreak — Primary entry point for Thai word tokenization and typographic line breaking.
 *
 * Fast, high-accuracy Thai text processing for PHP and Laravel.
 *
 * Usage:
 *   use ThaiBreak\ThaiBreak;
 *
 *   // 1. Word Tokenization
 *   $words = ThaiBreak::words('ฉันรักภาษาไทย');
 *   // → ['ฉัน', 'รัก', 'ภาษา', 'ไทย']
 *
 *   // 2. Insert Line Breaks (ZWSP) for dompdf, mPDF, or Web
 *   $pdfSafeHtml = ThaiBreak::lines('<p>บริษัทได้จัดสรรงบประมาณ ฿50,000,000</p>', isHtml: true);
 *
 *   // 3. Soft Wrapping for fixed-width text
 *   $wrapped = ThaiBreak::wrap($longText, width: 45);
 *
 * License: Apache-2.0
 */
class ThaiBreak
{
    /**
     * Tokenize Thai text into words.
     *
     * @param  string $text           UTF-8 text
     * @param  bool   $keepWhitespace Whether to include whitespace tokens
     * @return string[]
     */
    public static function words(string $text, bool $keepWhitespace = false): array
    {
        return ThaiTokenizer::getDefault()->tokenize($text, $keepWhitespace);
    }

    /**
     * Alias of words() — tokenize Thai text into words.
     *
     * @param  string $text           UTF-8 text
     * @param  bool   $keepWhitespace Whether to include whitespace tokens
     * @return string[]
     */
    public static function tokenize(string $text, bool $keepWhitespace = false): array
    {
        return self::words($text, $keepWhitespace);
    }

    /**
     * Word boundaries of the text, in the style of an ICU BreakIterator: n segments give n + 1
     * offsets from 0 to the text length, and whitespace and punctuation are segments of their own.
     *
     * Offsets are bytes by default, for substr(); pass $inCharacters for code point offsets, for
     * mb_substr().
     *
     * @param  string $text         UTF-8 text
     * @param  bool   $inCharacters Count code points instead of bytes
     * @return int[]
     */
    public static function boundaries(string $text, bool $inCharacters = false): array
    {
        $offsets = [0];
        $pos = 0;
        foreach (ThaiTokenizer::getDefault()->tokenize($text, true) as $token) {
            $pos += $inCharacters ? mb_strlen($token, 'UTF-8') : strlen($token);
            $offsets[] = $pos;
        }
        return $offsets;
    }

    /**
     * Tokenize Thai text and join tokens with a delimiter.
     *
     * @param  string $text      UTF-8 text
     * @param  string $delimiter Delimiter string (default '|')
     * @return string
     */
    public static function join(string $text, string $delimiter = '|'): string
    {
        return ThaiTokenizer::getDefault()->tokenizeToString($text, $delimiter);
    }

    /**
     * Insert line break opportunities (Zero-Width Space U+200B by default)
     * adhering to Thai typographic rules (no mid-word cuts, no orphan punctuation).
     *
     * @param  string $text   Plain text or HTML markup
     * @param  string $marker Marker to insert (default: "\u{200B}")
     * @param  bool   $isHtml Whether to preserve HTML tags and entities
     * @return string
     */
    public static function lines(string $text, string $marker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false): string
    {
        return ThaiTokenizer::getDefault()->insertLineBreaks($text, $marker, $isHtml);
    }

    /**
     * Alias of lines() — insert line break opportunities.
     */
    public static function breakLines(string $text, string $marker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false): string
    {
        return self::lines($text, $marker, $isHtml);
    }

    /**
     * Soft wrap Thai text to a specified visual column width.
     * Accurately accounts for Thai combining vowels and tone marks (0 width).
     *
     * @param  string $text         UTF-8 plain text
     * @param  int    $width        Maximum column width per line
     * @param  string $break        Line break separator (default "\n")
     * @param  bool   $cutLongWords Whether to force break words wider than $width
     * @return string
     */
    public static function wrap(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false): string
    {
        return ThaiTokenizer::getDefault()->wrap($text, $width, $break, $cutLongWords);
    }

    /**
     * Calculate visual display column width for Thai text (ignoring combining marks).
     */
    public static function displayWidth(string $text): int
    {
        return ThaiLineBreaker::thaiDisplayWidth($text);
    }

    /**
     * Add custom words with optional weights to the shared tokenizer instance.
     *
     * @param array<string, float>|string[] $words
     */
    public static function addCustomWords(array $words): void
    {
        ThaiTokenizer::getDefault()->addCustomWords($words);
    }

    /**
     * Add a single custom word to the shared tokenizer instance.
     */
    public static function addWord(string $word, float $weight = 1.0): void
    {
        ThaiTokenizer::getDefault()->addWord($word, $weight);
    }

    /**
     * Get the underlying ThaiTokenizer singleton.
     */
    public static function getTokenizer(): ThaiTokenizer
    {
        return ThaiTokenizer::getDefault();
    }

    /**
     * Get the underlying ThaiLineBreaker bound to the default tokenizer.
     */
    public static function getLineBreaker(): ThaiLineBreaker
    {
        return ThaiTokenizer::getDefault()->getLineBreaker();
    }
}
