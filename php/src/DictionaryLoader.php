<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * DictionaryLoader — loads Thai word dictionaries with weights.
 *
 * Supports multiple formats:
 *   1. Plain text file     : one word per line (weight = 1.0)
 *   2. TSV file            : word<TAB>weight per line
 *   3. PHP array           : ['word' => weight, ...]  or  ['word1', 'word2', ...]
 *   4. Inline word list    : array passed directly
 *
 * Built-in dictionary path: data/wordlist.tsv
 *
 * The TSV format is preferred because it encodes word frequency, allowing
 * the WeightedTokenizer to prefer more common words when multiple valid
 * segmentations exist.
 *
 * License: Apache-2.0
 */
class DictionaryLoader
{
    /**
     * Load a plain text word list (one word per line).
     * All words get weight = 1.0.
     *
     * @param  string $filePath Path to .txt file
     * @param  float  $weight   Default weight for all words
     */
    public static function fromTextFile(string $filePath, float $weight = 1.0): ThaiTrie
    {
        self::assertFileReadable($filePath);
        $trie   = new ThaiTrie();
        $handle = fopen($filePath, 'r');
        if ($handle === false) {
            throw new \RuntimeException("Cannot read file: $filePath");
        }
        while (($line = fgets($handle)) !== false) {
            $word = trim($line);
            if ($word !== '' && $word[0] !== '#') {
                $trie->add($word, $weight);
            }
        }
        fclose($handle);
        return $trie;
    }

    /**
     * Load a TSV file with format:  word<TAB>weight
     * Lines starting with '#' are treated as comments.
     *
     * @param  string $filePath Path to .tsv file
     */
    public static function fromTsvFile(string $filePath): ThaiTrie
    {
        self::assertFileReadable($filePath);
        $trie   = new ThaiTrie();
        $handle = fopen($filePath, 'r');
        if ($handle === false) {
            throw new \RuntimeException("Cannot read file: $filePath");
        }
        while (($line = fgets($handle)) !== false) {
            $line = trim($line);
            if ($line === '' || $line[0] === '#') {
                continue;
            }
            $tab = strpos($line, "\t");
            if ($tab !== false) {
                $word   = substr($line, 0, $tab);
                $weight = (float) substr($line, $tab + 1);
            } else {
                $word   = $line;
                $weight = 1.0;
            }
            if ($word !== '') {
                $trie->add($word, $weight);
            }
        }
        fclose($handle);
        return $trie;
    }

    /**
     * Load from a PHP array.
     * Accepts:
     *   - Associative: ['คน' => 15000.0, 'รัก' => 8000.0]
     *   - Sequential:  ['คน', 'รัก', 'ไทย']  (weight = 1.0)
     *
     * @param  array<int|string, string|float> $words
     */
    public static function fromArray(array $words): ThaiTrie
    {
        $trie = new ThaiTrie();
        $trie->addMany($words);
        return $trie;
    }

    /**
     * Merge multiple tries into one (useful for combining base + domain dict).
     * Words in later tries override weights from earlier ones if higher.
     *
     * @param  ThaiTrie[] $tries
     */
    public static function merge(array $tries): ThaiTrie
    {
        // We cannot directly access nodes cross-instance, so re-export words
        // using a shared flat list. Users can call addMany multiple times instead.
        throw new \LogicException(
            'Use ThaiTrie::addMany() to merge dictionaries manually. '
            . 'Load the base dictionary first, then call addMany() with domain words.'
        );
    }

    // ------------------------------------------------------------------

    private static function assertFileReadable(string $path): void
    {
        if (!is_file($path) || !is_readable($path)) {
            throw new \InvalidArgumentException("File not found or not readable: $path");
        }
    }
}
