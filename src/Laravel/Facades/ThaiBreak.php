<?php

declare(strict_types=1);

namespace ThaiBreak\Laravel\Facades;

use Illuminate\Support\Facades\Facade;
use ThaiBreak\ThaiTokenizer;
use ThaiBreak\ThaiLineBreaker;
use ThaiBreak\ThaiTrie;
use ThaiBreak\WeightedTokenizer;

/**
 * ThaiBreak Facade for Laravel.
 *
 * @method static array words(string $text, bool $keepWhitespace = false)
 * @method static array tokenize(string $text, bool $keepWhitespace = false)
 * @method static string join(string $text, string $delimiter = '|')
 * @method static string tokenizeToString(string $text, string $delimiter = '|')
 * @method static string lines(string $text, string $marker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false)
 * @method static string breakLines(string $text, string $marker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false)
 * @method static string insertLineBreaks(string $text, string $marker = ThaiLineBreaker::DEFAULT_MARKER, bool $isHtml = false)
 * @method static string wrap(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false)
 * @method static string wrapText(string $text, int $width = 60, string $break = "\n", bool $cutLongWords = false)
 * @method static int displayWidth(string $text)
 * @method static void addWord(string $word, float $weight = 1.0)
 * @method static void addCustomWords(array $words)
 * @method static ThaiLineBreaker getLineBreaker()
 * @method static ThaiTrie getTrie()
 * @method static WeightedTokenizer getEngine()
 *
 * @see \ThaiBreak\ThaiBreak
 * @see \ThaiBreak\ThaiTokenizer
 */
class ThaiBreak extends Facade
{
    /**
     * Get the registered name of the component.
     */
    protected static function getFacadeAccessor(): string
    {
        return 'thaibreak';
    }
}
