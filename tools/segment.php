<?php

/**
 * Segment text line by line for benchmarks and quick checks.
 *
 * Usage:
 *   php tools/segment.php [DICTIONARY] < input.txt
 *
 * Reads one text per line from stdin and prints its words joined by "|".
 * DICTIONARY is a word list (.txt) or a word<TAB>weight file (.tsv). Without it the
 * library's default tokenizer is used (the bundled word list with its word weights), so
 * the output is that of ThaiBreak::words(). A .txt list has no weights.
 *
 * License: Apache-2.0
 */

declare(strict_types=1);

require dirname(__DIR__) . '/vendor/autoload.php';

use ThaiBreak\ThaiTokenizer;

$dict = $argv[1] ?? null;
$tokenizer = $dict === null
    ? ThaiTokenizer::getDefault()
    : (str_ends_with($dict, '.tsv') ? ThaiTokenizer::fromTsvFile($dict) : ThaiTokenizer::fromTextFile($dict));

while (($line = fgets(STDIN)) !== false) {
    echo implode('|', $tokenizer->tokenize(rtrim($line, "\r\n"))), "\n";
}
