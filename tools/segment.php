<?php

/**
 * Segment text line by line for benchmarks and quick checks.
 *
 * Usage:
 *   php tools/segment.php [DICTIONARY] < input.txt
 *
 * Reads one text per line from stdin and prints its words joined by "|".
 * DICTIONARY is a word list (.txt) or a word<TAB>weight file (.tsv);
 * the default is data/words.txt.
 *
 * License: Apache-2.0
 */

declare(strict_types=1);

require dirname(__DIR__) . '/vendor/autoload.php';

use ThaiBreak\ThaiTokenizer;

$dict = $argv[1] ?? dirname(__DIR__) . '/data/words.txt';
$tokenizer = str_ends_with($dict, '.tsv') ? ThaiTokenizer::fromTsvFile($dict) : ThaiTokenizer::fromTextFile($dict);

while (($line = fgets(STDIN)) !== false) {
    echo implode('|', $tokenizer->tokenize(rtrim($line, "\r\n"))), "\n";
}
