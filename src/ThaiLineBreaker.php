<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * ThaiLineBreaker — Line breaking and text wrapping for Thai text.
 *
 * Designed for PDF rendering engines (dompdf, mPDF, TCPDF), web browsers,
 * and console/receipt printer text layout.
 *
 * Implements line breaking rules aligned with:
 *   - W3C Requirements for Thai Text Layout (clreq/tlreq)
 *   - Unicode Standard Annex #14 (UAX #14: Line Breaking Algorithm)
 *   - Royal Society of Thailand (ราชบัณฑิตยสภา) typographic standards
 *
 * Key Rules:
 *   1. No Line-Start: Closing punctuation (), ], }, ”, ’, », etc.), sentence
 *      terminators (,, ., :, ;, !, ?), and Thai postfixes (ๆ ไม้ยมก, ฯ ไปยาลน้อย,
 *      ฯลฯ ไปยาลใหญ่) MUST NEVER appear at the beginning of a line.
 *   2. No Line-End: Opening punctuation ((, [, {, “, ‘, «, etc.) and prefix
 *      symbols (฿, $, #, @) MUST NEVER appear at the end of a line.
 *   3. Whitespace Safety: Natural spaces are already line break opportunities;
 *      never insert zero-width break markers directly adjacent to spaces,
 *      preventing orphan spaces at the start of new lines.
 *   4. Numbers & Abbreviations: Never break inside numbers (10,000, 3.14, 40%)
 *      or Thai dotted abbreviations (พ.ศ., รพ., น., จ.).
 *   5. HTML Awareness: When processing HTML, tags (<...>) and entities (&...;)
 *      are preserved untouched — only text nodes receive break opportunities.
 *
 * License: Apache-2.0
 */
class ThaiLineBreaker
{
    /** Default break opportunity marker: Zero-Width Space (U+200B) */
    public const DEFAULT_MARKER = "\u{200B}";

    /**
     * Tokens that must NOT end a line (never break AFTER these).
     * Includes open brackets, quotes, and prefix currency/hashtag symbols.
     */
    private const PAT_NO_BREAK_AFTER = '/^[([{\"“‘<«฿$€¥£#@（【《]$/u';

    /**
     * Tokens that must NOT start a line (never break BEFORE these).
     * Includes close brackets, quotes, sentence punctuation, and Thai postfixes (ๆ, ฯ, ฯลฯ).
     */
    private const PAT_NO_BREAK_BEFORE = '/^(?:[)\\]}\"”’>»,.:;!?ๆฯ๏๚๛）】》]|ฯลฯ)$/u';

    /** Pattern matching HTML tags and HTML entities */
    private const PAT_HTML_TAGS = '/(<[^>]+>|&[a-zA-Z0-9#]+;)/u';

    private ThaiTokenizer $tokenizer;

    public function __construct(?ThaiTokenizer $tokenizer = null)
    {
        $this->tokenizer = $tokenizer ?? ThaiTokenizer::getDefault();
    }

    /**
     * Insert line break opportunity markers (default: Zero-Width Space U+200B)
     * at typographically permissible word boundaries.
     *
     * Suitable for passing to dompdf, mPDF, or web templates to enable clean
     * Thai line wrapping without breaking mid-word or hanging punctuation.
     *
     * @param  string $text        UTF-8 text or HTML fragment
     * @param  string $breakMarker Marker string to insert (default: "\u{200B}")
     * @param  bool   $isHtml      Whether to treat input as HTML (protecting tags & entities)
     * @return string
     */
    public function insertLineBreaks(string $text, string $breakMarker = self::DEFAULT_MARKER, bool $isHtml = false): string
    {
        if ($text === '') {
            return '';
        }

        // Auto-detect HTML if not explicitly specified
        if ($isHtml || str_contains($text, '<') && str_contains($text, '>')) {
            return $this->processHtmlText($text, $breakMarker);
        }

        return $this->processPlainText($text, $breakMarker);
    }

    /**
     * Wrap Thai text into lines of at most `$width` display columns.
     *
     * Uses visual display width (where Thai combining vowels and tone marks
     * take 0 columns), breaking only at valid word boundaries and adhering to
     * line-start/line-end typographic constraints.
     *
     * @param  string $text          UTF-8 plain text
     * @param  int    $width         Maximum display width per line in columns
     * @param  string $break         Line break separator (default: "\n")
     * @param  bool   $cutLongWords  Whether to force-break words longer than $width
     * @return string
     */
    public function wrap(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false): string
    {
        if ($text === '' || $width <= 0) {
            return $text;
        }

        // Normalize newlines and process paragraph by paragraph
        $paragraphs = preg_split('/\r?\n/u', $text) ?: [$text];
        $wrappedParagraphs = [];

        foreach ($paragraphs as $para) {
            if ($para === '') {
                $wrappedParagraphs[] = '';
                continue;
            }

            $tokens = $this->tokenizer->tokenize($para, true);
            $lines = [];
            $curLine = '';
            $curWidth = 0;

            foreach ($tokens as $tok) {
                if ($tok === '') {
                    continue;
                }

                $w = self::thaiDisplayWidth($tok);

                // If leading whitespace on a fresh line, skip it
                if ($curLine === '' && trim($tok) === '') {
                    continue;
                }

                if ($curWidth + $w <= $width) {
                    $curLine  .= $tok;
                    $curWidth += $w;
                } else {
                    // Current line is full
                    if ($curLine !== '') {
                        // Check hanging punctuation rule: if $tok cannot start a line (e.g. ๆ or ))
                        // and current line is not empty, allow hanging onto the current line
                        if (preg_match(self::PAT_NO_BREAK_BEFORE, $tok) && $curWidth + $w <= $width + 3) {
                            $curLine  .= $tok;
                            $curWidth += $w;
                            continue;
                        }

                        $lines[] = rtrim($curLine);

                        // If the next token starting the new line is whitespace, skip it
                        if (trim($tok) === '') {
                            $curLine  = '';
                            $curWidth = 0;
                            continue;
                        }

                        $curLine  = $tok;
                        $curWidth = $w;
                    } else {
                        // Single token exceeds line width
                        if ($cutLongWords && $w > $width) {
                            // Break long token by TCC/chars
                            $chars = preg_split('//u', $tok, -1, PREG_SPLIT_NO_EMPTY) ?: [$tok];
                            $part = '';
                            $partW = 0;
                            foreach ($chars as $ch) {
                                $cw = self::thaiDisplayWidth($ch);
                                if ($partW + $cw > $width && $part !== '') {
                                    $lines[] = $part;
                                    $part = $ch;
                                    $partW = $cw;
                                } else {
                                    $part .= $ch;
                                    $partW += $cw;
                                }
                            }
                            $curLine  = $part;
                            $curWidth = $partW;
                        } else {
                            $lines[]  = $tok;
                            $curLine  = '';
                            $curWidth = 0;
                        }
                    }
                }
            }

            if ($curLine !== '') {
                $lines[] = rtrim($curLine);
            }

            $wrappedParagraphs[] = implode($break, $lines);
        }

        return implode($break, $wrappedParagraphs);
    }

    /**
     * Determine whether a line break is permissible between two adjacent tokens.
     *
     * @param string $left  The token preceding the potential break point
     * @param string $right The token following the potential break point
     */
    public static function canBreakBetween(string $left, string $right): bool
    {
        // 1. Whitespace safety: never insert break adjacent to spaces
        if (preg_match('/\s$/u', $left) || preg_match('/^\s/u', $right)) {
            return false;
        }

        // 2. Left token must not end a line (open brackets, prefix currency/tags)
        if (preg_match(self::PAT_NO_BREAK_AFTER, $left)) {
            return false;
        }

        // 3. Right token must not start a line (close brackets, commas, periods, ๆ, ฯ, ฯลฯ)
        if (preg_match(self::PAT_NO_BREAK_BEFORE, $right)) {
            return false;
        }

        // 4. Numbers connected by hyphen, slash, colon, or period (e.g. 10-20, 1/2)
        if (is_numeric($left) && in_array($right, ['-', '/', ':', '.', '%'], true)) {
            return false;
        }
        if (in_array($left, ['-', '/', ':'], true) && is_numeric($right)) {
            return false;
        }

        return true;
    }

    /**
     * Calculate visual display column width for Thai text.
     *
     * Combining above/below vowels, tone marks, and zero-width spaces are counted
     * as 0 display width, while base consonants and Latin/digits occupy 1 column,
     * and full-width characters occupy 2 columns.
     */
    public static function thaiDisplayWidth(string $text): int
    {
        // Strip zero-width Thai combining marks (vowels, tone marks, thanthakat) and ZWSP
        $stripped = preg_replace('/[\x{0E31}\x{0E34}-\x{0E3A}\x{0E47}-\x{0E4E}\x{200B}]/u', '', $text);
        return mb_strwidth($stripped ?? '', 'UTF-8');
    }

    // ──────────────────────────────────────────────────────────────────────
    // Internal Processing
    // ──────────────────────────────────────────────────────────────────────

    /**
     * Process plain text by tokenizing and inserting break markers where permitted.
     */
    private function processPlainText(string $text, string $breakMarker): string
    {
        $tokens = $this->tokenizer->tokenize($text, true);
        $n      = count($tokens);

        if ($n <= 1) {
            return $text;
        }

        $out = '';
        for ($i = 0; $i < $n; $i++) {
            $out .= $tokens[$i];
            if ($i + 1 < $n && self::canBreakBetween($tokens[$i], $tokens[$i + 1])) {
                $out .= $breakMarker;
            }
        }

        return $out;
    }

    /**
     * Process HTML by isolating tags/entities and only adding break markers to text nodes.
     */
    private function processHtmlText(string $html, string $breakMarker): string
    {
        $parts = preg_split(self::PAT_HTML_TAGS, $html, -1, PREG_SPLIT_DELIM_CAPTURE);
        if ($parts === false || count($parts) <= 1) {
            return $this->processPlainText($html, $breakMarker);
        }

        $out = '';
        foreach ($parts as $part) {
            if ($part === '') {
                continue;
            }
            // If part is an HTML tag (<...>) or HTML entity (&...;), leave it intact
            if (($part[0] === '<' && str_ends_with($part, '>')) ||
                ($part[0] === '&' && str_ends_with($part, ';'))) {
                $out .= $part;
            } else {
                $out .= $this->processPlainText($part, $breakMarker);
            }
        }

        return $out;
    }
}
