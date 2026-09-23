<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * ThaiTokenizer — High-level facade for weighted Thai word segmentation.
 *
 * Usage:
 *   $tokenizer = ThaiTokenizer::withDefaultDict();
 *   $words = $tokenizer->tokenize('ฉันรักภาษาไทย');
 *   // → ['ฉัน', 'รัก', 'ภาษา', 'ไทย']
 *
 *   // Custom domain dictionary (e.g., medical terms with higher weight)
 *   $tokenizer->addCustomWords(['อัลตราซาวด์' => 5000.0, 'ผ่าตัด' => 3000.0]);
 *   $words = $tokenizer->tokenize('ผู้ป่วยต้องอัลตราซาวด์');
 *
 * License: Apache-2.0
 */
class ThaiTokenizer
{
    private static ?self      $defaultInstance = null;
    private ThaiTrie          $trie;
    private ?BigramModel      $bigrams;
    private WeightedTokenizer $engine;
    private ?ThaiLineBreaker  $lineBreaker = null;

    public function __construct(ThaiTrie $trie, ?BigramModel $bigrams = null)
    {
        $this->trie    = $trie;
        $this->bigrams = $bigrams;
        $this->engine  = new WeightedTokenizer($trie, $bigrams);
    }

    // ------------------------------------------------------------------
    // Factory methods
    // ------------------------------------------------------------------

    /**
     * Get or create a shared singleton instance using the default dictionary and bigram model.
     */
    public static function getDefault(): self
    {
        return self::$defaultInstance ??= self::withDefaultDict();
    }

    /**
     * Create a tokenizer with the built-in Thai word dictionary.
     * Dictionary is loaded from data/words.txt.
     */
    public static function withDefaultDict(?string $dictPath = null, ?string $bigramPath = null): self
    {
        $path = $dictPath;
        if ($path === null) {
            $candidates = [
                __DIR__ . '/../../data/words.txt',
                __DIR__ . '/../data/words.txt',
                'data/words.txt',
            ];
            foreach ($candidates as $c) {
                if (file_exists($c)) {
                    $path = $c;
                    break;
                }
            }
        }

        if ($path !== null && file_exists($path)) {
            $trie = str_ends_with($path, '.tsv')
                ? DictionaryLoader::fromTsvFile($path)
                : DictionaryLoader::fromTextFile($path);
        } else {
            $trie = new ThaiTrie();
        }

        $bigramModel = null;
        if ($bigramPath !== null && file_exists($bigramPath)) {
            $bigramModel = BigramModel::fromTsvFile($bigramPath);
        }

        return new self($trie, $bigramModel);
    }

    /**
     * Create a tokenizer from a TSV dictionary file.
     *
     * @param string $tsvPath Path to TSV file (word<TAB>weight)
     */
    public static function fromTsvFile(string $tsvPath): self
    {
        return new self(DictionaryLoader::fromTsvFile($tsvPath));
    }

    /**
     * Create a tokenizer from a plain text word list.
     *
     * @param string $txtPath Path to plain text file (one word per line)
     */
    public static function fromTextFile(string $txtPath): self
    {
        return new self(DictionaryLoader::fromTextFile($txtPath));
    }

    /**
     * Create a tokenizer from a PHP array.
     *
     * @param array<int|string, string|float> $words
     */
    public static function fromArray(array $words): self
    {
        return new self(DictionaryLoader::fromArray($words));
    }

    // ------------------------------------------------------------------
    // Dictionary & Bigram management
    // ------------------------------------------------------------------

    /**
     * Add custom words with optional weights at runtime.
     *
     * @param array<string, float>|string[] $words
     */
    public function addCustomWords(array $words): void
    {
        $this->trie->addMany($words);
        $this->engine      = new WeightedTokenizer($this->trie, $this->bigrams);
        $this->lineBreaker = null;
    }

    /**
     * Add a single word with weight.
     *
     * @param float $weight Higher weight = more preferred
     */
    public function addWord(string $word, float $weight = 1.0): void
    {
        $this->trie->add($word, $weight);
        $this->engine      = new WeightedTokenizer($this->trie, $this->bigrams);
        $this->lineBreaker = null;
    }

    public function getBigramModel(): ?BigramModel
    {
        return $this->bigrams;
    }

    public function withBigramModel(?BigramModel $model): self
    {
        $this->bigrams     = $model;
        $this->engine      = new WeightedTokenizer($this->trie, $model);
        $this->lineBreaker = null;
        return $this;
    }

    // ------------------------------------------------------------------
    // Tokenization
    // ------------------------------------------------------------------

    /**
     * Tokenize Thai text into word tokens.
     *
     * @param  string $text           UTF-8 Thai (or mixed) text
     * @param  bool   $keepWhitespace Include whitespace tokens in result
     * @return string[]
     */
    public function tokenize(string $text, bool $keepWhitespace = false): array
    {
        return $this->engine->tokenize($text, $keepWhitespace);
    }

    /**
     * Alias of tokenize() — tokenize Thai text into words.
     *
     * @param  string $text           UTF-8 text
     * @param  bool   $keepWhitespace Include whitespace tokens
     * @return string[]
     */
    public function words(string $text, bool $keepWhitespace = false): array
    {
        return $this->tokenize($text, $keepWhitespace);
    }

    /**
     * Tokenize and return tokens joined by a delimiter (useful for display).
     *
     * @param  string $text      Input text
     * @param  string $delimiter Separator between tokens (default '|')
     * @return string
     */
    public function tokenizeToString(string $text, string $delimiter = '|'): string
    {
        return implode($delimiter, $this->tokenize($text));
    }

    /**
     * Alias of tokenizeToString() — tokenize and join tokens with a delimiter.
     */
    public function join(string $text, string $delimiter = '|'): string
    {
        return $this->tokenizeToString($text, $delimiter);
    }

    /**
     * Expose the underlying trie (for inspection or serialization).
     */
    public function getTrie(): ThaiTrie
    {
        return $this->trie;
    }

    /**
     * Expose the underlying segmentation engine.
     */
    public function getEngine(): WeightedTokenizer
    {
        return $this->engine;
    }

    // ------------------------------------------------------------------
    // Line Breaking & Wrapping (PDF & Typographic layout)
    // ------------------------------------------------------------------

    /**
     * Get or create a ThaiLineBreaker instance bound to this tokenizer.
     */
    public function getLineBreaker(): ThaiLineBreaker
    {
        return $this->lineBreaker ??= new ThaiLineBreaker($this);
    }

    /**
     * Insert line break markers (default: Zero-Width Space U+200B) for Thai text.
     * Adheres to Thai typographical rules (preventing mid-word breaks and orphan punctuation).
     *
     * @param string $text        UTF-8 text or HTML fragment
     * @param string $breakMarker Marker string (default: "\u{200B}")
     * @param bool   $isHtml      Whether to treat input as HTML (protecting tags & entities)
     */
    public function insertLineBreaks(string $text, string $breakMarker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false): string
    {
        return $this->getLineBreaker()->insertLineBreaks($text, $breakMarker, $isHtml);
    }

    /**
     * Alias of insertLineBreaks().
     */
    public function lines(string $text, string $breakMarker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false): string
    {
        return $this->insertLineBreaks($text, $breakMarker, $isHtml);
    }

    /**
     * Wrap Thai text into lines of at most $width display columns.
     *
     * @param string $text         UTF-8 plain text
     * @param int    $width        Maximum display width per line in columns
     * @param string $break        Line break separator (default "\n")
     * @param bool   $cutLongWords Whether to force break words wider than $width
     */
    public function wrap(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false): string
    {
        return $this->getLineBreaker()->wrap($text, $width, $break, $cutLongWords);
    }

    /**
     * Static convenience helper: insert line break markers into Thai text using the default dictionary.
     */
    public static function breakLines(string $text, string $breakMarker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false): string
    {
        return self::getDefault()->insertLineBreaks($text, $breakMarker, $isHtml);
    }

    /**
     * Static convenience helper: wrap Thai text into lines using the default dictionary.
     */
    public static function wrapText(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false): string
    {
        return self::getDefault()->wrap($text, $width, $break, $cutLongWords);
    }
}
