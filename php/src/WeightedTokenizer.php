<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Weighted Thai Word Tokenizer — Viterbi Algorithm with Bigram Collocation and Pattern Acceptors.
 *
 * Architecture Overview
 * ---------------------
 * 1. Graph edge generation:
 *    - Thai dictionary words from weighted prefix map.
 *    - Thai abbreviation patterns (e.g. รพ., พญ., ด.ญ., มิ.ย., น., จ.).
 *    - Non-Thai formatted tokens (English words, formatted numbers, percentages, whitespace,
 *      and individual punctuation symbols like _ ( ) " -).
 *
 * 2. Single-pass Viterbi forward DP with Bigram Transitions:
 *    - Each word costs -log P(w) with P(w) = weight / total weight of the
 *      dictionary (a unigram model). With equal weights this prefers the
 *      segmentation with the fewest words.
 *    - Non-Thai tokens cost as much as the rarest word (weight 1), Thai
 *      abbreviation patterns 1.5 times that, and unknown-word fallbacks twice
 *      that, so a dictionary word followed by "." beats a pattern such as
 *      "ว." that would cut the word.
 *    - dp[j]   = minimum accumulated cost to reach character position j.
 *    - from[j] = best predecessor position.
 *    - word[j] = word on the winning edge.
 *    - isUnk[j]= whether position j was reached via unknown-word fallback.
 *    - When BigramModel is present, edge cost incorporates the collocation
 *      bonus: P(w | word[i]), disambiguating compound words based on context.
 *
 * 3. Syllable-based OOV chunking:
 *    - In traceback, consecutive unknown Thai TCC clusters are merged into
 *      coherent OOV token chunks (e.g. foreign names and transliterations),
 *      preventing broken single-character fragments.
 *
 * Complexity: O(n · k) linear time.
 * License: Apache-2.0
 */
class WeightedTokenizer
{
    /** Pattern matching non-Thai tokens: English words, numbers, whitespace, single symbols */
    private const PAT_NONTHAI = '/\G(?:[a-zA-Z]+(?:[-_\'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\x{0E00}-\x{0E7F}a-zA-Z0-9 \t\r\n])/u';

    /** Pattern matching Thai abbreviations with periods (e.g. รพ., พญ., ด.ญ., มิ.ย., น., จ.) */
    private const PAT_ABBR = '/\G(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+/u';

    /**
     * Names of the letters of the Latin alphabet in Thai. A name that starts another one comes first
     * (เอ็ม before เอ), so the longest name wins.
     */
    private const LETTER_NAMES = [
        'ดับเบิลยู', 'เอ็กซ์', 'แอล', 'เอ็ม', 'เอ็น', 'เอฟ', 'เอช', 'เอส', 'อาร์', 'คิว', 'แซด', 'ซี', 'ดี', 'จี', 'ไอ', 'เจ', 'เค',
        'โอ', 'พี', 'ยู', 'วี', 'วาย', 'บี', 'ที', 'อี', 'เอ',
    ];

    /** Cost of a run of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น), relative to the cost of the rarest word */
    private const LETTER_RUN_COST_FACTOR = 1.0;

    /** Cost of an abbreviation pattern, relative to the cost of the rarest word */
    private const ABBR_COST_FACTOR = 1.5;

    /**
     * Extra cost per letter of an abbreviation pattern, relative to the cost of
     * the rarest word, so "เขต|จ." beats "เข|ตจ." (a pattern taking the last
     * letter of the previous word).
     */
    private const ABBR_LETTER_COST_FACTOR = 0.01;

    /**
     * Costs closer than this are a tie. Edges into a position are visited from
     * the longest word to the shortest, so on a tie the later, shorter last word
     * wins and the earlier words stay longer ("ผิด|ราย" rather than "ผิ|ดราย").
     */
    private const TIE_EPSILON = 1e-9;

    /** Cost of an unknown-word fallback edge, relative to the cost of the rarest word */
    private const UNKNOWN_COST_FACTOR = 2.0;

    /**
     * An out-of-vocabulary word may span up to this many TCC clusters. Such an edge competes with
     * the dictionary words, so a long unknown word is kept whole instead of being cut into short words.
     */
    private const OOV_MAX_CLUSTERS = 6;

    /** Cost of an out-of-vocabulary edge of one cluster, relative to the cost of the rarest word. */
    private const OOV_COST_FACTOR = 3.0;

    /**
     * Extra cost per further cluster of an out-of-vocabulary edge, relative to the cost of the
     * rarest word. Below about 0.5 an unknown word beats real words and F1 drops sharply.
     */
    private const OOV_CLUSTER_COST_FACTOR = 0.8;

    /**
     * Frequent function words. An out-of-vocabulary edge may not start or end with one of them, nor
     * contain one at a cluster boundary, so an unknown word does not swallow its neighbours
     * (ฮิวจ์ส|ไม่|ได้, not ฮิวจ์สไม่ได้; ชวรัตน์|ที่|ลืม|ไว้, not ชวรัตน์ที่ลืมไว้): the same name
     * would otherwise be tokenized differently on its own and next to a function word, and a search
     * for it would miss the document.
     */
    private const OOV_BOUNDARY_WORDS = [
        'ที่', 'และ', 'ของ', 'ใน', 'ได้', 'ให้', 'ไม่', 'ว่า', 'เป็น',
        'มี', 'จะ', 'ไป', 'มา', 'ก็', 'กับ', 'แต่', 'หรือ', 'จาก',
        'โดย', 'เพื่อ', 'แล้ว', 'อยู่', 'นี้', 'นั้น', 'ซึ่ง', 'การ', 'ความ',
        'ต่อ', 'ถึง', 'ยัง', 'เมื่อ', 'ทั้ง', 'ตาม', 'ด้วย', 'อีก', 'คือ',
        'ไว้',
        // words that often come right before a name
        'นาย', 'นาง', 'พี่', 'น้อง', 'แพทย์', 'ตัว',
    ];

    /**
     * Function words that may still occur inside an out-of-vocabulary word: they are frequent
     * syllables of transliterated names (อาห์มาดิเนจัด, อะมีรุ้ลฮัจย์).
     */
    private const OOV_INNER_ALLOWED = ['มา', 'มี'];

    private ThaiTrie     $trie;
    private ?BigramModel $bigramModel;

    /** @var array<string, true>|null Characters of the Thai block U+0E00-U+0E7F */
    private static ?array $thaiChars = null;

    /** @var array<string, true>|null Characters an out-of-vocabulary edge may cover (Thai letters, vowels and marks, but not ๆ, ฯ, digits or other symbols, which are tokens of their own) */
    private static ?array $oovChars = null;

    /**
     * OOV_BOUNDARY_WORDS by first character, and their byte and character lengths, and the bit mask of
     * those that may not occur inside an out-of-vocabulary word.
     *
     * @var array{byFirst: array<string, list<array{0: int, 1: list<string>, 2: int}>>, bytes: list<int>, lens: list<int>, inner: int}|null
     */
    private static ?array $functionWords = null;

    /** @var array<string, list<string>>|null LETTER_NAMES by first character, in LETTER_NAMES order */
    private static ?array $letterNames = null;

    private static function initTables(): void
    {
        if (self::$functionWords !== null) {
            return;
        }
        $thai = [];
        for ($c = 0x0E00; $c <= 0x0E7F; $c++) {
            $thai[mb_chr($c, 'UTF-8')] = true;
        }
        $oov = [];
        foreach ([[0x0E01, 0x0E2E], [0x0E30, 0x0E3A], [0x0E40, 0x0E45], [0x0E47, 0x0E4E]] as [$lo, $hi]) {
            for ($c = $lo; $c <= $hi; $c++) {
                $oov[mb_chr($c, 'UTF-8')] = true;
            }
        }
        $byFirst = [];
        $bytes   = [];
        $lens    = [];
        $inner   = 0;
        foreach (self::OOV_BOUNDARY_WORDS as $k => $f) {
            $chars = mb_str_split($f, 1, 'UTF-8');
            $byFirst[$chars[0]][] = [$k, $chars, count($chars)];
            $bytes[$k] = strlen($f);
            $lens[$k]  = count($chars);
            if (!in_array($f, self::OOV_INNER_ALLOWED, true)) {
                $inner |= 1 << $k;
            }
        }
        $letters = [];
        foreach (self::LETTER_NAMES as $name) {
            $letters[mb_substr($name, 0, 1, 'UTF-8')][] = $name;
        }
        self::$thaiChars     = $thai;
        self::$oovChars      = $oov;
        self::$letterNames   = $letters;
        self::$functionWords = ['byFirst' => $byFirst, 'bytes' => $bytes, 'lens' => $lens, 'inner' => $inner];
    }

    public function __construct(ThaiTrie $trie, ?BigramModel $bigramModel = null)
    {
        $this->trie        = $trie;
        $this->bigramModel = $bigramModel;
    }

    public function setBigramModel(?BigramModel $bigramModel): self
    {
        $this->bigramModel = $bigramModel;
        return $this;
    }

    public function getBigramModel(): ?BigramModel
    {
        return $this->bigramModel;
    }

    // ──────────────────────────────────────────────────────────────────────
    // Public API
    // ──────────────────────────────────────────────────────────────────────

    /**
     * Tokenize UTF-8 Thai (and mixed) text into word tokens.
     *
     * @param  string $text           UTF-8 text to segment
     * @param  bool   $keepWhitespace Include whitespace tokens in output
     * @return string[]
     */
    public function tokenize(string $text, bool $keepWhitespace = false): array
    {
        if ($text === '') {
            return [];
        }

        $chars = $this->trie->splitChars($text);

        // Match against a normalized copy, but return the original characters
        [$normChars, $origPos] = self::normalizeForMatching($chars);
        $tokens = $this->segment($normChars);
        if ($normChars !== $chars) {
            $original = [];
            $pos      = 0;
            foreach ($tokens as $tok) {
                $end        = $pos + mb_strlen($tok, 'UTF-8');
                $original[] = implode('', array_slice($chars, $origPos[$pos], $origPos[$end] - $origPos[$pos]));
                $pos        = $end;
            }
            $tokens = $original;
        }

        if (!$keepWhitespace) {
            // Unicode whitespace (e.g. NBSP), as in the Go, Rust and TypeScript ports
            $tokens = array_values(array_filter($tokens, fn($t) => !preg_match('/^\s*$/u', $t)));
        }

        return $tokens;
    }

    // ──────────────────────────────────────────────────────────────────────
    // Core Viterbi Segmentation
    // ──────────────────────────────────────────────────────────────────────

    /**
     * Viterbi DP segmentation with Bigram Transitions and OOV chunking.
     *
     * @param  list<string> $chars  Unicode characters of the text
     * @return list<string>         Segmented tokens
     */
    private function segment(array $chars): array
    {
        $n        = count($chars);
        $text     = implode('', $chars);
        $validPos = ThaiTCC::tccPosArray($chars);

        // A match at offset 0 marks the string as valid UTF-8, so the matches below at other offsets
        // do not check the rest of the text again (that made each of them cost O(length))
        preg_match('//u', $text);

        self::initTables();
        $thaiChars = self::$thaiChars;
        $oovChars  = self::$oovChars;
        $fw        = self::$functionWords;
        $fwBytes   = $fw['bytes'];
        $fwLens    = $fw['lens'];
        $fwInner   = $fw['inner'];

        // Bit k of $fwStarts[p] is set when OOV_BOUNDARY_WORDS[k] begins at character p, and bit k of
        // $fwEnds[p] when it ends there (sparse arrays: only positions with a function word are set)
        $fwStarts = [];
        $fwEnds   = [];
        for ($p = 0; $p < $n; $p++) {
            if (!isset($fw['byFirst'][$chars[$p]])) {
                continue;
            }
            foreach ($fw['byFirst'][$chars[$p]] as [$k, $fc, $flen]) {
                if ($p + $flen > $n) {
                    continue;
                }
                for ($t = 1; $t < $flen; $t++) {
                    if ($chars[$p + $t] !== $fc[$t]) {
                        continue 2;
                    }
                }
                $bit = 1 << $k;
                $fwStarts[$p]         = ($fwStarts[$p] ?? 0) | $bit;
                $fwEnds[$p + $flen]   = ($fwEnds[$p + $flen] ?? 0) | $bit;
            }
        }

        // Precompute byte offsets of each character index for zero-copy regex matching
        $charByteOffsets = [0];
        $b = 0;
        foreach ($chars as $ch) {
            $b += strlen($ch);
            $charByteOffsets[] = $b;
        }

        // Unigram costs: -log(weight / total); a word of weight 1 costs $rareCost
        $normalizer = $this->trie->totalWeight() + 1.0;
        $rareCost   = log($normalizer);

        // ── Single-pass Viterbi DP ──────────────────────────────────────────
        // dp[j]    — minimum accumulated cost to position j
        // from[j]  — predecessor position
        // (every edge into j covers chars[from[j]..j), so no word is stored)
        // isUnk[j] — whether position j was reached via unknown fallback
        //
        // Positions are visited in order. When position i is reached, every
        // edge into it has been relaxed, so dp[i] is final: an unreachable i
        // gets an unknown-word edge, then the edges starting at i are relaxed
        // right away. Edges are never stored, so memory stays O(n) for any
        // text length.
        $dp    = array_fill(0, $n + 1, INF);
        $from  = array_fill(0, $n + 1, -1);
        $isUnk = array_fill(0, $n + 1, false);
        $dp[0] = 0.0;

        for ($i = 0; $i <= $n; $i++) {
            if (!$validPos[$i]) {
                continue;
            }

            // ── Unknown-word fallback: connect to nearest reachable predecessor
            if ($dp[$i] === INF) {
                for ($k = $i - 1; $k >= 0; $k--) {
                    if ($dp[$k] < INF && $validPos[$k]) {
                        $dp[$i]    = $dp[$k] + self::UNKNOWN_COST_FACTOR * $rareCost;
                        $from[$i]  = $k;
                        $isUnk[$i] = true;
                        break;
                    }
                }
            }

            if ($i === $n) {
                break;
            }

            // ── Edges starting at i: [end position, cost, out-of-vocabulary]
            $edges   = [];
            $bytePos = $charByteOffsets[$i];

            if (isset($thaiChars[$chars[$i]])) {
                // ── 1. Thai dictionary words starting at position $i
                foreach ($this->trie->prefixesFromChars($chars, $i) as $entry) {
                    $j = $i + mb_strlen($entry['word'], 'UTF-8');
                    if ($j <= $n && $validPos[$j]) {
                        $edges[] = [$j, log($normalizer / $entry['weight']), false];
                    }
                }

                // ── 2. Thai abbreviation patterns (e.g. รพ., พญ., ด.ญ., มิ.ย., น., จ.)
                // The pattern needs a full stop within four characters
                $dot = false;
                for ($t = $i + 1, $tEnd = min($i + 4, $n - 1); $t <= $tEnd; $t++) {
                    if ($chars[$t] === '.') {
                        $dot = true;
                        break;
                    }
                }
                if ($dot && preg_match(self::PAT_ABBR, $text, $mAbbr, 0, $bytePos) && $mAbbr[0] !== '') {
                    $abbrLen = mb_strlen($mAbbr[0], 'UTF-8');
                    $j       = $i + $abbrLen;
                    if ($j <= $n && $validPos[$j] && !$this->abbreviationIsWords($chars, $j, $mAbbr[0])) {
                        $letters  = $abbrLen - substr_count($mAbbr[0], '.');
                        $edges[]  = [$j, (self::ABBR_COST_FACTOR + self::ABBR_LETTER_COST_FACTOR * $letters) * $rareCost, false];
                    }
                }

                // ── 3. Runs of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น, ดีเอสไอ)
                for ($end = $i, $letters = 0; ;) {
                    $next = 0;
                    foreach (($end < $n ? (self::$letterNames[$chars[$end]] ?? []) : []) as $name) {
                        $len = mb_strlen($name, 'UTF-8');
                        if ($end + $len <= $n && $validPos[$end + $len] && implode('', array_slice($chars, $end, $len)) === $name) {
                            $next = $end + $len;
                            break;
                        }
                    }
                    if ($next === 0) {
                        break;
                    }
                    $end = $next;
                    if (++$letters >= 2) {
                        $edges[] = [$end, self::LETTER_RUN_COST_FACTOR * $rareCost, false];
                    }
                }

                // ── 4. Out-of-vocabulary words of 1..OOV_MAX_CLUSTERS TCC clusters
                $clusters = 0;
                $fwHere   = $fwStarts[$i] ?? 0;
                for ($j = $i + 1; $j <= $n && isset($oovChars[$chars[$j - 1]]); $j++) {
                    if (!$validPos[$j]) {
                        continue;
                    }
                    $clusters++;
                    if ($clusters > self::OOV_MAX_CLUSTERS) {
                        break;
                    }
                    $fwEnd = $fwEnds[$j] ?? 0;
                    $hits  = $fwHere | $fwEnd;
                    if ($hits !== 0) {
                        // The edge borders a function word that is shorter than the edge
                        $edgeBytes = $charByteOffsets[$j] - $charByteOffsets[$i];
                        $borders   = false;
                        for ($k = 0; $hits !== 0; $k++, $hits >>= 1) {
                            if (($hits & 1) === 1 && $edgeBytes > $fwBytes[$k]) {
                                $borders = true;
                                break;
                            }
                        }
                        if ($borders) {
                            // Every longer edge from $i would contain a function word that ends here
                            $inner = $fwEnd & $fwInner;
                            for ($k = 0; $inner !== 0; $k++, $inner >>= 1) {
                                if (($inner & 1) === 1 && $j - $i > $fwLens[$k] && !empty($validPos[$j - $fwLens[$k]])) {
                                    break 2;
                                }
                            }
                            continue;
                        }
                    }
                    $edges[] = [
                        $j,
                        (self::OOV_COST_FACTOR + self::OOV_CLUSTER_COST_FACTOR * ($clusters - 1)) * $rareCost,
                        true,
                    ];
                }
            } else {
                // ── 5. Non-Thai tokens (English words, numbers, single symbols, spaces)
                if (preg_match(self::PAT_NONTHAI, $text, $mNonThai, 0, $bytePos) && $mNonThai[0] !== '') {
                    $j = $i + mb_strlen($mNonThai[0], 'UTF-8');
                    if ($j <= $n && $validPos[$j]) {
                        $edges[] = [$j, $rareCost, false];
                    }
                }
            }

            // ── Relax the edges
            $prev = ($this->bigramModel !== null && $from[$i] >= 0)
                ? implode('', array_slice($chars, $from[$i], $i - $from[$i]))
                : '';
            foreach ($edges as [$j, $baseCost, $unknown]) {
                // Apply bigram collocation bonus if bigram model is present
                if ($prev !== '') {
                    $w        = implode('', array_slice($chars, $i, $j - $i));
                    $bonus    = $this->bigramModel->getBonus($prev, $w, $j - $i);
                    $edgeCost = max(0.01, $baseCost - $bonus);
                } else {
                    $edgeCost = $baseCost;
                }

                $newCost = $dp[$i] + $edgeCost;

                // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
                if ($newCost <= $dp[$j] + self::TIE_EPSILON) {
                    $dp[$j]    = $newCost;
                    $from[$j]  = $i;
                    $isUnk[$j] = $unknown;
                }
            }
        }

        // ── Traceback ────────────────────────────────────────────────────────
        if ($dp[$n] === INF) {
            return [implode('', $chars)];
        }

        $spans = [];
        $pos   = $n;
        while ($pos > 0) {
            $spans[] = [$from[$pos], $pos, $isUnk[$pos]];
            $pos     = $from[$pos];
            if ($pos < 0) {
                break;
            }
        }
        $spans = array_reverse($spans);

        // ── Syllable-based OOV Chunking ──────────────────────────────────────
        // Merge adjacent unknown Thai TCC clusters into coherent OOV chunks
        $tokens   = [];
        $curChunk = '';

        foreach ($spans as [$start, $end, $unk]) {
            $tok = implode('', array_slice($chars, $start, $end - $start));
            if ($unk && preg_match('/^[\x{0E00}-\x{0E7F}]+$/u', $tok)) {
                $curChunk .= $tok;
            } else {
                if ($curChunk !== '') {
                    $tokens[] = $curChunk;
                    $curChunk = '';
                }
                $tokens[] = $tok;
            }
        }
        if ($curChunk !== '') {
            $tokens[] = $curChunk;
        }

        return $tokens;
    }

    // ──────────────────────────────────────────────────────────────────────
    // Helpers
    // ──────────────────────────────────────────────────────────────────────

    /**
     * Normalize common Thai spelling variants for dictionary matching:
     *   เ + เ → แ,  ํ + า → ำ,  ํ + tone + า → tone + ำ (e.g. "นํ้า" → "น้ำ"),
     *   ำ + tone → tone + ำ (a tone mark typed after Sara Am, "นำ้" → "น้ำ")
     *
     * Returns the normalized characters and, for each of them, the index of
     * the original character it starts at (plus a final entry for the end).
     * Token boundaries never fall inside a replaced pair, so tokens map back
     * to exact slices of the original text.
     *
     * @param  list<string> $chars
     * @return array{0: list<string>, 1: list<int>}
     */
    private static function normalizeForMatching(array $chars): array
    {
        $norm = [];
        $orig = [];
        $n    = count($chars);
        for ($i = 0; $i < $n; $i++) {
            $next = $chars[$i + 1] ?? '';
            if ($chars[$i] === 'เ' && $next === 'เ') {
                array_push($norm, 'แ');
                array_push($orig, $i++);
            } elseif ($chars[$i] === 'ำ' && self::isToneMark($next)) {
                array_push($norm, $next, 'ำ');
                array_push($orig, $i, $i);
                $i += 1;
            } elseif ($chars[$i] === "\u{0E4D}" && $next === 'า' && self::isToneMark($chars[$i + 2] ?? '')) {
                array_push($norm, $chars[$i + 2], 'ำ');
                array_push($orig, $i, $i);
                $i += 2;
            } elseif ($chars[$i] === "\u{0E4D}" && $next === 'า') {
                array_push($norm, 'ำ');
                array_push($orig, $i++);
            } elseif ($chars[$i] === "\u{0E4D}" && self::isToneMark($next) && ($chars[$i + 2] ?? '') === 'า') {
                array_push($norm, $next, 'ำ');
                array_push($orig, $i, $i);
                $i += 2;
            } else {
                $norm[] = $chars[$i];
                $orig[] = $i;
            }
        }
        $orig[] = $n;

        return [$norm, $orig];
    }

    private static function isToneMark(string $char): bool
    {
        return $char !== '' && $char >= '่' && $char <= '๋';
    }

    /**
     * Whether what the abbreviation pattern matched is really words and a full stop. The pattern
     * accepts any one to four consonant letters before a full stop, and อ ว ย count as consonants,
     * so it matches "ไป." in "ต่อไป." and "ของส." in "ของส.ส.". It is not an abbreviation when it
     * starts with a function word followed by more letters ("ของส.ส." is ของ + ส.ส.), or when it is
     * one dictionary word with a vowel or tone mark and a final full stop that is not followed by a
     * letter ("ต่อไป."), unless the dictionary has the abbreviation itself. A string of consonants only
     * (มทส. นพ.) stays an abbreviation even when the dictionary has it without the full stop.
     *
     * @param list<string> $chars
     * @param int          $end   Index of the character after the match
     */
    private function abbreviationIsWords(array $chars, int $end, string $abbr): bool
    {
        $first = explode('.', $abbr, 2)[0];
        foreach (self::OOV_BOUNDARY_WORDS as $f) {
            if (strlen($first) > strlen($f) && str_starts_with($first, $f)) {
                return true;
            }
        }
        $next       = $chars[$end] ?? '';
        $nextIsThai = $next !== '' && $next >= "\u{0e01}" && $next <= "\u{0e5b}";

        return substr_count($abbr, '.') === 1
            && str_ends_with($abbr, '.')
            && mb_strlen($first, 'UTF-8') >= 2
            && preg_match('/[^ก-ฮ]/u', $first) === 1
            && !$nextIsThai
            && $this->trie->has($first)
            && !$this->trie->has($abbr);
    }
}
