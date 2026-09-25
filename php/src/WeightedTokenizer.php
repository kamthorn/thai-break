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

    private ThaiTrie     $trie;
    private ?BigramModel $bigramModel;

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

        $chars  = $this->trie->splitChars($text);
        $tokens = $this->segment($chars);

        if (!$keepWhitespace) {
            $tokens = array_values(array_filter($tokens, fn($t) => trim($t) !== ''));
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
        // word[j]  — word token on winning edge into j
        // isUnk[j] — whether position j was reached via unknown fallback
        //
        // Positions are visited in order. When position i is reached, every
        // edge into it has been relaxed, so dp[i] is final: an unreachable i
        // gets an unknown-word edge, then the edges starting at i are relaxed
        // right away. Edges are never stored, so memory stays O(n) for any
        // text length.
        $dp    = array_fill(0, $n + 1, INF);
        $from  = array_fill(0, $n + 1, -1);
        $word  = array_fill(0, $n + 1, '');
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
                        $word[$i]  = implode('', array_slice($chars, $k, $i - $k));
                        $isUnk[$i] = true;
                        break;
                    }
                }
            }

            if ($i === $n) {
                break;
            }

            // ── Edges starting at i: [end position, word, cost]
            $edges   = [];
            $bytePos = $charByteOffsets[$i];

            if ($this->isThai($chars[$i])) {
                // ── 1. Thai dictionary words starting at position $i
                foreach ($this->trie->prefixesFromChars($chars, $i) as $entry) {
                    $j = $i + mb_strlen($entry['word'], 'UTF-8');
                    if ($j <= $n && $validPos[$j]) {
                        $edges[] = [$j, $entry['word'], log($normalizer / $entry['weight'])];
                    }
                }

                // ── 2. Thai abbreviation patterns (e.g. รพ., พญ., ด.ญ., มิ.ย., น., จ.)
                if (preg_match(self::PAT_ABBR, $text, $mAbbr, 0, $bytePos) && $mAbbr[0] !== '') {
                    $abbrLen = mb_strlen($mAbbr[0], 'UTF-8');
                    $j       = $i + $abbrLen;
                    if ($j <= $n && $validPos[$j]) {
                        $letters  = $abbrLen - substr_count($mAbbr[0], '.');
                        $edges[]  = [$j, $mAbbr[0], (self::ABBR_COST_FACTOR + self::ABBR_LETTER_COST_FACTOR * $letters) * $rareCost];
                    }
                }
            } else {
                // ── 3. Non-Thai tokens (English words, numbers, single symbols, spaces)
                if (preg_match(self::PAT_NONTHAI, $text, $mNonThai, 0, $bytePos) && $mNonThai[0] !== '') {
                    $j = $i + mb_strlen($mNonThai[0], 'UTF-8');
                    if ($j <= $n && $validPos[$j]) {
                        $edges[] = [$j, $mNonThai[0], $rareCost];
                    }
                }
            }

            // ── Relax the edges
            foreach ($edges as [$j, $w, $baseCost]) {
                // Apply bigram collocation bonus if bigram model is present
                if ($this->bigramModel !== null && $word[$i] !== '') {
                    $bonus    = $this->bigramModel->getBonus($word[$i], $w, $j - $i);
                    $edgeCost = max(0.01, $baseCost - $bonus);
                } else {
                    $edgeCost = $baseCost;
                }

                $newCost = $dp[$i] + $edgeCost;

                // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
                if ($newCost <= $dp[$j] + self::TIE_EPSILON) {
                    $dp[$j]    = $newCost;
                    $from[$j]  = $i;
                    $word[$j]  = $w;
                    $isUnk[$j] = false;
                }
            }
        }

        // ── Traceback ────────────────────────────────────────────────────────
        if ($dp[$n] === INF) {
            return [implode('', $chars)];
        }

        $rawTokens   = [];
        $rawIsUnk    = [];
        $pos         = $n;
        while ($pos > 0) {
            $rawTokens[] = $word[$pos];
            $rawIsUnk[]  = $isUnk[$pos];
            $pos         = $from[$pos];
            if ($pos < 0) {
                break;
            }
        }

        $rawTokens = array_reverse($rawTokens);
        $rawIsUnk  = array_reverse($rawIsUnk);

        // ── Syllable-based OOV Chunking ──────────────────────────────────────
        // Merge adjacent unknown Thai TCC clusters into coherent OOV chunks
        $tokens   = [];
        $curChunk = '';

        foreach ($rawTokens as $idx => $tok) {
            if ($rawIsUnk[$idx] && preg_match('/^[\x{0E00}-\x{0E7F}]+$/u', $tok)) {
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

    /** Return true if the character is in the Thai Unicode block U+0E00-U+0E7F */
    private function isThai(string $char): bool
    {
        return (bool) preg_match('/[\x{0E00}-\x{0E7F}]/u', $char);
    }
}
