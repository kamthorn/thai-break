#!/usr/bin/env php
<?php
/**
 * tools/build_php_dict.php
 * Compile TSV/Text wordlist into an OPcache preloading-ready PHP array file.
 *
 * Usage:
 *   php tools/build_php_dict.php [input.txt] [output.php]
 * Default:
 *   data/words.txt -> data/words.php
 */

$inputFile  = $argv[1] ?? __DIR__ . '/../data/words.txt';
$outputFile = $argv[2] ?? __DIR__ . '/../data/words.php';

if (!file_exists($inputFile)) {
    fwrite(STDERR, "Error: Input file '$inputFile' not found.\n");
    exit(1);
}

echo "Building OPcache dictionary from '$inputFile'...\n";
$t0 = microtime(true);

$prefixes = [];
$totalWeight = 0.0;
$wordCount = 0;

$handle = fopen($inputFile, 'r');
if (!$handle) {
    fwrite(STDERR, "Error: Cannot open '$inputFile'.\n");
    exit(1);
}

while (($line = fgets($handle)) !== false) {
    $line = trim($line);
    if ($line === '' || $line[0] === '#') {
        continue;
    }
    $parts = explode("\t", $line);
    $word = $parts[0];
    $weight = isset($parts[1]) ? (float)$parts[1] : 1.0;
    if ($weight <= 0) $weight = 1.0;

    $chars = preg_split('//u', $word, -1, PREG_SPLIT_NO_EMPTY);
    $last = count($chars) - 1;
    $sub = '';

    foreach ($chars as $idx => $ch) {
        $sub .= $ch;
        if ($idx === $last) {
            $old = $prefixes[$sub] ?? 0.0;
            $prefixes[$sub] = max($old, $weight);
            $totalWeight += $prefixes[$sub] - $old;
        } elseif (!isset($prefixes[$sub])) {
            $prefixes[$sub] = 0.0;
        }
    }
    $wordCount++;
}
fclose($handle);

// Write to PHP file with var_export for fastest opcache parsing
$header = "<?php\n/**\n * Auto-generated OPcache Preload Dictionary.\n * Words: $wordCount, Prefix Entries: " . count($prefixes) . "\n * Generated on " . date('Y-m-d H:i:s') . "\n */\ndeclare(strict_types=1);\n\nreturn [\n    'totalWeight' => $totalWeight,\n    'wordCount' => $wordCount,\n    'prefixes' => [\n";

$fp = fopen($outputFile, 'w');
fwrite($fp, $header);

foreach ($prefixes as $key => $weight) {
    $escaped = addcslashes($key, "'\\");
    fwrite($fp, "        '$escaped' => $weight,\n");
}

fwrite($fp, "    ],\n];\n");
fclose($fp);

$elapsed = round((microtime(true) - $t0) * 1000, 2);
$outSize = round(filesize($outputFile) / 1024, 2);

echo "Done in {$elapsed} ms!\n";
echo "  Words: $wordCount\n";
echo "  Prefix Entries: " . count($prefixes) . "\n";
echo "  Output File: $outputFile ($outSize KB)\n";
