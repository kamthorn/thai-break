<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Bigram Collocation Model for Thai Word Tokenization.
 *
 * Models transition probabilities between adjacent words: P(w_2 | w_1).
 * Used by WeightedTokenizer during Viterbi forward DP to reward frequent
 * word collocations, resolving compound word ambiguities (e.g. ตากลม, ผู้นำ,
 * คนละคน, ไม่ได้, ทำการ, วันที่).
 *
 * License: Apache-2.0
 */
class BigramModel
{
    /** @var array<string, float> pairKey ("prev\tnext") => count */
    private array $bigrams = [];

    /** @var float Scaling factor for bigram transition bonus */
    private float $alpha;

    public function __construct(float $alpha = 0.15)
    {
        $this->alpha = $alpha;
    }

    /**
     * Set the scaling weight of bigram bonuses.
     */
    public function setAlpha(float $alpha): self
    {
        $this->alpha = $alpha;
        return $this;
    }

    public function getAlpha(): float
    {
        return $this->alpha;
    }

    /**
     * Add a bigram transition with frequency count.
     */
    public function add(string $prevWord, string $nextWord, float $count = 1.0): void
    {
        $this->bigrams[$prevWord . "\t" . $nextWord] = $count;
    }

    /**
     * Check if a bigram transition exists.
     */
    public function has(string $prevWord, string $nextWord): bool
    {
        return isset($this->bigrams[$prevWord . "\t" . $nextWord]);
    }

    /**
     * Calculate transition bonus for a word pair.
     * Higher frequency pair yields a higher cost deduction (bonus).
     *
     * @param string $prevWord Preceding word token
     * @param string $nextWord Current word candidate
     * @param int    $wordLen  Length of nextWord in characters
     * @return float           Cost deduction bonus
     */
    public function getBonus(string $prevWord, string $nextWord, int $wordLen = 1): float
    {
        if ($this->alpha <= 0.0 || $prevWord === '' || $nextWord === '') {
            return 0.0;
        }

        $key = $prevWord . "\t" . $nextWord;
        if (!isset($this->bigrams[$key])) {
            return 0.0;
        }

        $count = $this->bigrams[$key];
        return ($this->alpha * log(1.0 + $count)) / max(1, $wordLen);
    }

    /**
     * Count of registered bigrams.
     */
    public function count(): int
    {
        return count($this->bigrams);
    }

    /**
     * Load bigrams from a TSV file (format: prev<TAB>next<TAB>count).
     */
    public static function fromTsvFile(string $filePath, float $alpha = 0.15): self
    {
        if (!is_readable($filePath)) {
            throw new \RuntimeException("Cannot read bigram file: $filePath");
        }

        $model  = new self($alpha);
        $handle = fopen($filePath, 'r');
        if ($handle === false) {
            throw new \RuntimeException("Cannot open file: $filePath");
        }

        while (($line = fgets($handle)) !== false) {
            $line = trim($line);
            if ($line === '' || $line[0] === '#') {
                continue;
            }
            $parts = explode("\t", $line);
            if (count($parts) >= 3) {
                $model->add($parts[0], $parts[1], (float) $parts[2]);
            }
        }
        fclose($handle);

        return $model;
    }

    /**
     * Load from array: ['prev\tnext' => count, ...].
     *
     * @param array<string, float|int> $bigrams
     */
    public static function fromArray(array $bigrams, float $alpha = 0.15): self
    {
        $model = new self($alpha);
        foreach ($bigrams as $key => $count) {
            $model->bigrams[$key] = (float) $count;
        }
        return $model;
    }
}
