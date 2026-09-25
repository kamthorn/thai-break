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
     * Includes open brackets, quotes, and hashtag/at symbols.
     */
    private const PAT_NO_BREAK_AFTER = '/^[([{\"“‘<«#@（【《]$/u';

    /**
     * Tokens that must NOT start a line (never break BEFORE these).
     * Includes quotes and Thai postfixes (ๆ, ฯ, ฯลฯ).
     */
    private const PAT_NO_BREAK_BEFORE = '/^(?:[\"”’>»ๆฯ๏]|ฯลฯ)$/u';

    /** Pattern matching HTML raw blocks (comments, scripts, styles), tags, and entities */
    private const PAT_HTML_TAGS = '/(<!--.*?-->|<script\b[^>]*>.*?<\/script>|<style\b[^>]*>.*?<\/style>|<[^>]+>|&[a-zA-Z0-9#]+;)/usi';

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
     * take 0 columns) and breaks only at the same break opportunities that
     * insertLineBreaks() marks, or after spaces. Trailing spaces are trimmed.
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
            $lines    = [];
            $curLine  = '';
            $curWidth = 0;

            foreach ($this->breakSegments($para) as $seg) {
                // Trailing spaces may hang past the margin, so only the visible part must fit
                $visible      = rtrim($seg);
                $visibleWidth = self::thaiDisplayWidth($visible);

                if ($curLine !== '' && $curWidth + $visibleWidth > $width) {
                    // Indentation that does not fit is dropped rather than left as an empty line
                    if (rtrim($curLine) !== '') {
                        $lines[] = rtrim($curLine);
                    }
                    $curLine  = '';
                    $curWidth = 0;
                }

                // A single segment wider than the line
                if ($curLine === '' && $cutLongWords && $visibleWidth > $width) {
                    $pieces = self::cutToWidth($visible, $width);
                    $seg    = array_pop($pieces) . substr($seg, strlen($visible));
                    array_push($lines, ...$pieces);
                }

                $curLine  .= $seg;
                $curWidth += self::thaiDisplayWidth($seg);
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
     * The tokens are treated as separate dictionary words, so a Thai|Thai
     * junction is a word boundary.
     *
     * @param string $left  The token preceding the potential break point
     * @param string $right The token following the potential break point
     */
    public static function canBreakBetween(string $left, string $right): bool
    {
        if ($left === '' || $right === '') {
            return false;
        }

        $cps  = self::codePoints($left . $right);
        $at   = count(self::codePoints($left));
        $dict = array_fill(0, count($cps) + 1, false);
        $dict[$at] = true;

        // Whitespace safety: never insert break adjacent to spaces
        if (preg_match('/\s$/u', $left) || preg_match('/^\s/u', $right)) {
            return false;
        }

        return Uax14::breakOpportunities($cps, $dict)[$at] === Uax14::ALLOWED
            && self::passesTypographicRules($left, $right);
    }

    /**
     * Legacy token-level rules that are not yet expressed as UAX #14 rules.
     */
    private static function passesTypographicRules(string $left, string $right): bool
    {
        // 2. Left token must not end a line (open brackets, quotes, tags)
        if (preg_match(self::PAT_NO_BREAK_AFTER, $left)) {
            return false;
        }

        // 3. Right token must not start a line (quotes, ๆ, ฯ, ฯลฯ)
        if (preg_match(self::PAT_NO_BREAK_BEFORE, $right)) {
            return false;
        }

        // 4. Latin letters and digits (UAX #14 LB23): WP01, ISO29110, 3rd
        if ((preg_match('/[A-Za-z]$/', $left) && preg_match('/^[0-9]/', $right)) ||
            (preg_match('/[0-9]$/', $left) && preg_match('/^[A-Za-z]/', $right))) {
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
     * Process plain text by inserting break markers between break segments.
     */
    private function processPlainText(string $text, string $breakMarker): string
    {
        $out  = '';
        $prev = '';
        foreach ($this->breakSegments($text) as $seg) {
            // Breaks after spaces and ZWSP are already implicit: never put a marker next to them
            if ($prev !== '' && !preg_match('/[\s\x{200B}]$/u', $prev)) {
                $out .= $breakMarker;
            }
            $out .= $seg;
            $prev = $seg;
        }

        return $out;
    }

    /**
     * Split plain text into segments that must not be broken internally.
     * A line may break between any two segments; spaces stay at the end of
     * the segment they follow.
     *
     * @return list<string>
     */
    private function breakSegments(string $text): array
    {
        $tokens = $this->tokenizer->tokenize($text, true);
        $n      = count($tokens);

        if ($n <= 1) {
            return $text === '' ? [] : [$text];
        }

        // Token ends are the dictionary word boundaries inside Thai runs
        $cps  = [];
        $ends = [];
        foreach ($tokens as $tok) {
            array_push($cps, ...self::codePoints($tok));
            $ends[] = count($cps);
        }
        $dict = array_fill(0, count($cps) + 1, false);
        foreach ($ends as $end) {
            $dict[$end] = true;
        }
        $actions = Uax14::breakOpportunities($cps, $dict);

        $segments = [];
        $cur      = '';
        for ($i = 0; $i < $n; $i++) {
            $cur .= $tokens[$i];
            if ($i + 1 < $n && $actions[$ends[$i]] === Uax14::ALLOWED
                && self::passesTypographicRules($tokens[$i], $tokens[$i + 1])) {
                $segments[] = $cur;
                $cur        = '';
            }
        }
        $segments[] = $cur;

        return $segments;
    }

    /**
     * Force-break text that is wider than a line into pieces of at most $width columns.
     *
     * @return non-empty-list<string>
     */
    private static function cutToWidth(string $text, int $width): array
    {
        $pieces = [];
        $part   = '';
        $partW  = 0;
        foreach (preg_split('//u', $text, -1, PREG_SPLIT_NO_EMPTY) ?: [$text] as $ch) {
            $cw = self::thaiDisplayWidth($ch);
            if ($partW + $cw > $width && $part !== '') {
                $pieces[] = $part;
                $part     = $ch;
                $partW    = $cw;
            } else {
                $part  .= $ch;
                $partW += $cw;
            }
        }
        $pieces[] = $part;

        return $pieces;
    }

    /**
     * Split a UTF-8 string into Unicode code points.
     *
     * @return list<int>
     */
    private static function codePoints(string $text): array
    {
        if ($text === '') {
            return [];
        }
        return array_values(unpack('N*', mb_convert_encoding($text, 'UTF-32BE', 'UTF-8')) ?: []);
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
