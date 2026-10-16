<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * ThaiLineBreaker — Line breaking and text wrapping for Thai text.
 *
 * Designed for PDF rendering engines (dompdf, mPDF, TCPDF), web browsers,
 * and console/receipt printer text layout.
 *
 * Break opportunities follow the Unicode Line Breaking Algorithm (UAX #14,
 * Unicode 16.0, all rules LB1–LB31; see Uax14):
 *   1. Dictionary inside Thai runs: SA (Complex_Context) text is segmented
 *      with the tokenizer; next to other characters Thai letters resolve to
 *      AL, so "ราคา100บาท" and "ไทย(สยาม)" are not broken.
 *   2. Thai tailoring (W3C tlreq): ๆ and ฯ (including ฯลฯ) never start a
 *      line, even after a space.
 *   3. Whitespace Safety: Natural spaces are already line break opportunities;
 *      never insert zero-width break markers directly adjacent to spaces,
 *      preventing orphan spaces at the start of new lines.
 *   4. HTML Awareness: Text is treated as HTML only when it contains a real
 *      tag; tags (<...>) and entities (&...;) are preserved untouched and
 *      only text nodes receive break opportunities.
 *
 * License: Apache-2.0
 */
class ThaiLineBreaker
{
    /** Default break opportunity marker: Zero-Width Space (U+200B) */
    public const DEFAULT_MARKER = "\u{200B}";

    /** Pattern detecting HTML markup: a tag starts with a letter, "/", "!" or "?" right after "<" */
    private const PAT_HTML_DETECT = '/<[a-zA-Z\/!?][^>]*>/u';

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
        if ($isHtml || preg_match(self::PAT_HTML_DETECT, $text)) {
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

        return Uax14::breakOpportunities($cps, $dict)[$at] === Uax14::ALLOWED;
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
     * UAX #14 is evaluated at every position; the tokenizer only supplies
     * the word boundaries inside Thai runs.
     *
     * @return list<string>
     */
    private function breakSegments(string $text): array
    {
        if ($text === '') {
            return [];
        }

        $chars = mb_str_split($text);
        $cps   = array_map(static fn(string $ch): int => mb_ord($ch) ?: 0xFFFD, $chars);
        $n     = count($chars);

        $dict = array_fill(0, $n + 1, false);
        $pos  = 0;
        foreach ($this->tokenizer->tokenize($text, true) as $tok) {
            $pos += mb_strlen($tok);
            $dict[min($pos, $n)] = true;
        }
        $actions = Uax14::breakOpportunities($cps, $dict);

        $segments = [];
        $start    = 0;
        for ($i = 1; $i < $n; $i++) {
            if ($actions[$i] === Uax14::ALLOWED) {
                $segments[] = implode('', array_slice($chars, $start, $i - $start));
                $start      = $i;
            }
        }
        $segments[] = implode('', array_slice($chars, $start));

        return $segments;
    }

    /**
     * Force-break text that is wider than a line into pieces of at most $width
     * columns. Pieces end only between Thai character clusters (TCC) and never
     * separate a character from its combining marks or a zero-width joiner.
     *
     * @return non-empty-list<string>
     */
    private static function cutToWidth(string $text, int $width): array
    {
        $chars    = mb_str_split($text);
        $tccValid = ThaiTCC::tccPosArray($chars);
        $clusters = [];
        $prevZwj  = false;
        foreach ($chars as $i => $ch) {
            $cls = Uax14::lineBreakClass(mb_ord($ch) ?: 0xFFFD);
            if ($i > 0 && (!$tccValid[$i] || $prevZwj || in_array($cls, [Uax14::CM, Uax14::SM, Uax14::ZWJ], true))) {
                $clusters[count($clusters) - 1] .= $ch;
            } else {
                $clusters[] = $ch;
            }
            $prevZwj = $cls === Uax14::ZWJ;
        }

        $pieces = [];
        $part   = '';
        $partW  = 0;
        foreach ($clusters as $cluster) {
            $cw = self::thaiDisplayWidth($cluster);
            if ($partW + $cw > $width && $part !== '') {
                $pieces[] = $part;
                $part     = $cluster;
                $partW    = $cw;
            } else {
                $part  .= $cluster;
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
        $parts = self::splitMarkup($html);
        if (count($parts) <= 1) {
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

    /**
     * Split HTML into text chunks and the comments, script and style blocks,
     * tags and entities between them, like preg_split() with
     * PREG_SPLIT_DELIM_CAPTURE and the pattern
     * `(<!--.*?-->|<script\b[^>]*>.*?<\/script>|<style\b[^>]*>.*?<\/style>|<[a-zA-Z\/!?][^>]*>|&[a-zA-Z0-9#]+;)`
     * (flags `usi`), but in linear time: with that pattern, every `<script>`,
     * `<!--` or `<a` whose terminator is missing scans to the end of the text again.
     *
     * @return list<string>
     */
    public static function splitMarkup(string $html): array
    {
        if (!mb_check_encoding($html, 'UTF-8')) {
            return [$html]; // the /u pattern matches nothing in invalid UTF-8
        }

        // Next position of each terminator from the last lookup (false: none). Lookups
        // come from increasing offsets, so a cached hit at or after the new offset is
        // still the first one and a cached miss stays a miss.
        $cache = [];
        $find = static function (string $needle, int $from) use ($html, &$cache) {
            if (array_key_exists($needle, $cache) && ($cache[$needle] === false || $cache[$needle] >= $from)) {
                return $cache[$needle];
            }
            return $cache[$needle] = $from > strlen($html) ? false : stripos($html, $needle, $from);
        };

        $parts = [];
        $last  = 0;
        $n     = strlen($html);
        for ($i = 0; $i < $n;) {
            $end = null;
            if ($html[$i] === '<') {
                if (substr_compare($html, '<!--', $i, 4) === 0) {
                    $j = $find('-->', $i + 4);
                    $end = $j === false ? null : $j + 3;
                }
                foreach (['script', 'style'] as $raw) {
                    $after = $i + strlen($raw) + 1;
                    if ($end === null && substr_compare($html, "<$raw", $i, strlen($raw) + 1, true) === 0
                        && preg_match('/^\w/u', mb_substr(substr($html, $after, 4), 0, 1, 'UTF-8')) !== 1) {
                        $k = $find('>', $after);
                        $j = $k === false ? false : $find("</$raw>", $k + 1);
                        $end = $j === false ? null : $j + strlen($raw) + 3;
                    }
                }
                $c = $html[$i + 1] ?? '';
                if ($end === null && (($c >= 'a' && $c <= 'z') || ($c >= 'A' && $c <= 'Z') || $c === '/' || $c === '!' || $c === '?')) {
                    $k = $find('>', $i + 1);
                    $end = $k === false ? null : $k + 1;
                }
            } elseif ($html[$i] === '&') {
                $run = strspn($html, 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789#', $i + 1);
                if ($run > 0 && ($html[$i + 1 + $run] ?? '') === ';') {
                    $end = $i + $run + 2;
                }
            }
            if ($end === null) {
                $i++;
                continue;
            }
            $parts[] = substr($html, $last, $i - $last);
            $parts[] = substr($html, $i, $end - $i);
            $i = $last = $end;
        }
        $parts[] = substr($html, $last);

        return $parts;
    }
}
