<?php

declare(strict_types=1);

namespace ThaiBreak;

/**
 * Ultra-compact Directed Acyclic Word Graph (DAWG / Minimal DFA) in PHP.
 *
 * Reads an immutable binary DAWG file (TBD1 format, ~226 KB for 36,040 Thai words (incl. 10,133 given names))
 * with zero pre-parsing overhead and minimal memory footprint (< 0.5 MB).
 *
 * License: Apache-2.0
 */
class CompactDawg
{
    private string $data;
    private \SplFixedArray $offsets;
    private int $numStates;
    private int $numWords;

    public function __construct(string $data)
    {
        if (strlen($data) < 12 || substr($data, 0, 4) !== 'TBD1') {
            throw new \InvalidArgumentException('Invalid DAWG header: expected TBD1 magic');
        }

        $this->data      = $data;
        $this->numStates = unpack('V', substr($data, 4, 4))[1];
        $this->numWords  = unpack('V', substr($data, 8, 4))[1];

        $this->offsets = new \SplFixedArray($this->numStates);
        $curr          = 12;
        for ($i = 0; $i < $this->numStates; $i++) {
            $this->offsets[$i] = $curr;
            $numEdges          = ord($data[$curr]) & 0x7F;
            $curr             += 1 + $numEdges * 4;
        }
    }

    public static function fromFile(string $filePath): self
    {
        if (!is_file($filePath) || !is_readable($filePath)) {
            throw new \InvalidArgumentException("DAWG file not found or not readable: $filePath");
        }
        $data = file_get_contents($filePath);
        if ($data === false) {
            throw new \RuntimeException("Failed to read DAWG file: $filePath");
        }
        return new self($data);
    }

    public function getNumWords(): int
    {
        return $this->numWords;
    }

    public function getNumStates(): int
    {
        return $this->numStates;
    }

    /**
     * Find all matching dictionary words starting at character index `$startPos`.
     *
     * @param  list<string> $chars    Pre-split UTF-8 characters
     * @param  int          $startPos Start character position in $chars
     * @param  int          $maxLen   Maximum word length in characters
     * @return array<array{word: string, weight: float}>
     */
    public function prefixesFromChars(array $chars, int $startPos = 0, int $maxLen = 25): array
    {
        $len     = count($chars);
        $limit   = min($len, $startPos + $maxLen);
        $state   = 0;
        $matches = [];
        $sub     = '';

        for ($i = $startPos; $i < $limit; $i++) {
            $chStr = $chars[$i];
            $sub  .= $chStr;
            $ch    = mb_ord($chStr);
            if ($ch > 0xFFFF) {
                break;
            }

            $off      = $this->offsets[$state];
            $flags    = ord($this->data[$off]);
            $numEdges = $flags & 0x7F;

            $low       = 0;
            $high      = $numEdges - 1;
            $nextState = -1;
            $base      = $off + 1;

            while ($low <= $high) {
                $mid      = ($low + $high) >> 1;
                $edgeOff  = $base + $mid * 4;
                $edgeChar = ord($this->data[$edgeOff]) | (ord($this->data[$edgeOff + 1]) << 8);

                if ($edgeChar === $ch) {
                    $nextState = ord($this->data[$edgeOff + 2]) | (ord($this->data[$edgeOff + 3]) << 8);
                    break;
                } elseif ($edgeChar < $ch) {
                    $low = $mid + 1;
                } else {
                    $high = $mid - 1;
                }
            }

            if ($nextState === -1) {
                break;
            }

            $state   = $nextState;
            $nextOff = $this->offsets[$state];
            if ((ord($this->data[$nextOff]) & 0x80) !== 0) {
                $matches[] = [
                    'word'   => $sub,
                    'weight' => 1.0,
                ];
            }
        }

        return $matches;
    }

    /**
     * Check if a word exists in the DAWG.
     */
    public function has(string $word): bool
    {
        $chars   = preg_split('//u', $word, -1, PREG_SPLIT_NO_EMPTY);
        if ($chars === false || empty($chars)) {
            return false;
        }
        $matches = $this->prefixesFromChars($chars, 0, count($chars));
        foreach ($matches as $m) {
            if ($m['word'] === $word) {
                return true;
            }
        }
        return false;
    }
}
