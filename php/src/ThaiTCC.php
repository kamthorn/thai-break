<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Thai Character Cluster (TCC) boundary detector.
 *
 * Implements the Thai Character Cluster grammar rules from:
 *   "Thai Character Cluster-Based Word Segmentation"
 *   (Theeramunkong et al., 2000; Jakkrit TeCho, Wittawat Jitkrittum / jtcc; Korakot Chaovavanich / PyThaiNLP)
 *
 * A TCC is the smallest inseparable unit in Thai script. Characters within a TCC
 * (such as a base consonant and its associated above/below vowels and tone marks,
 * or leading vowels) must never be split. Word boundaries may ONLY occur at
 * TCC boundaries.
 *
 * License: Apache-2.0
 */
class ThaiTCC
{
    /**
     * Regex matching one Thai Character Cluster anchored with \G at the current byte offset.
     */
    private static ?string $tccPattern = null;

    /**
     * Initialize and return the compiled Theeramunkong TCC regular expression.
     */
    public static function getPattern(): string
    {
        if (self::$tccPattern !== null) {
            return self::$tccPattern;
        }

        $c = '[ก-ฮ]';
        $t = '[่-๋]?';
        $d = '[ุู]';
        // 'k' matches an optional karan (thanthakat / silenced consonant) cluster:
        // e.g., 'ต์', 'ทร์', 'ธิ์', 'ทธ์', 'สิทธิ์'
        $k = '([ก-ฮ][ก-ฮ]?[ุูิ]?์)?';

        $rawRules = [
            'c[ั]([่-๋]c)?',
            'c[ั]([่-๋]c)?k',
            'เc็ck',
            'เcctาะk',
            'เccีtยะk',
            'เccีtย(?=[เ-ไก-ฮ]|$)k',
            'เc[ิีุู]tย(?=[เ-ไก-ฮ]|$)k',
            'เcc็ck',
            'เcิc์ck',
            'เcิtck',
            'เcีtยะ?k',
            'เcืtอะk',
            'เcื',
            'เctา?ะ?k',
            'c[ึื]tck',
            'c[ะ-ู]tk',
            'c[ิุู]์',
            'cรรc์',
            'c็',
            'ct[ะาำ]?k',
            'แc็ck',
            'แcc์k',
            'แctะk',
            'แcc็ck',
            'แccc์k',
            'โctะk',
            '[เ-ไ]ctk',
            'ก็',
            'อึ',
            'หึ',
        ];

        $patterns = [];
        foreach ($rawRules as $rule) {
            $p = str_replace('k', $k, $rule);
            $p = str_replace('c', $c, $p);
            $p = str_replace('t', $t, $p);
            $p = str_replace('d', $d, $p);
            $patterns[] = $p;
        }

        self::$tccPattern = '/\G(?:' . implode('|', $patterns) . ')/u';
        return self::$tccPattern;
    }

    /**
     * Compute a boolean array of valid TCC break positions for the given characters.
     *
     * $result[$i] === true  ⟹ position $i is a valid word boundary
     * $result[$i] === false ⟹ position $i is inside a cluster (never break here)
     *
     * Positions 0 and count($chars) are always valid.
     *
     * @param  list<string> $chars  Unicode characters of the text
     * @return bool[]               Valid break flags, indexed 0 … count($chars)
     */
    public static function tccPosArray(array $chars): array
    {
        $len = count($chars);

        $valid       = array_fill(0, $len + 1, false);
        $valid[0]    = true;
        $valid[$len] = true;

        if ($len === 0) {
            return $valid;
        }

        $text = implode('', $chars);

        // ------------------------------------------------------------------
        // Build character index ↔ byte offset lookup tables
        // ------------------------------------------------------------------
        $charByteOffsets = [0];
        $b = 0;
        foreach ($chars as $ch) {
            $b += strlen($ch);
            $charByteOffsets[] = $b;
        }
        $byteToChar = array_flip($charByteOffsets);
        $totalBytes = $b;

        $pattern = self::getPattern();

        // ------------------------------------------------------------------
        // Scan clusters using anchored \G pattern
        // ------------------------------------------------------------------
        $bytePos = 0;
        while ($bytePos < $totalBytes) {
            if (preg_match($pattern, $text, $match, 0, $bytePos) && $match[0] !== '') {
                $bytePos += strlen($match[0]);
                if (isset($byteToChar[$bytePos])) {
                    $valid[$byteToChar[$bytePos]] = true;
                }
            } else {
                // Fallback: advance by 1 Unicode character
                $chIdx = $byteToChar[$bytePos] ?? 0;
                $bytePos += strlen($chars[$chIdx] ?? ' ');
                if (isset($byteToChar[$bytePos])) {
                    $valid[$byteToChar[$bytePos]] = true;
                }
            }
        }

        // ------------------------------------------------------------------
        // Non-Thai characters: always valid break positions on both sides
        // ------------------------------------------------------------------
        foreach ($chars as $i => $ch) {
            if (!preg_match('/[\x{0E00}-\x{0E7F}]/u', $ch)) {
                $valid[$i]     = true;
                $valid[$i + 1] = true;
            }
        }

        return $valid;
    }
}
