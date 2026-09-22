/**
 * Flat Prefix Map Trie for ultra-fast word lookups in TypeScript / JavaScript.
 */

export interface PrefixMatch {
  word: string;
  end: number;
  weight: number;
}

export class ThaiTrie {
  private prefixes: Map<string, number> = new Map();
  private _maxWeight: number = 1.0;

  get maxWeight(): number {
    return this._maxWeight;
  }

  get size(): number {
    return this.prefixes.size;
  }

  /**
   * Add a word with its frequency weight.
   */
  add(word: string, weight: number): void {
    if (!word) return;
    if (weight > this._maxWeight) {
      this._maxWeight = weight;
    }

    const chars = Array.from(word);
    const n = chars.length;

    // Register all proper prefixes with weight 0 (continuation nodes)
    let p = '';
    for (let i = 0; i < n - 1; i++) {
      p += chars[i];
      if (!this.prefixes.has(p)) {
        this.prefixes.set(p, 0.0);
      }
    }

    // Register full word
    const existing = this.prefixes.get(word);
    if (existing === undefined || weight > existing) {
      this.prefixes.set(word, weight);
    }
  }

  /**
   * Find all matching dictionary words starting at character index `start`.
   */
  prefixesFromChars(chars: string[], start: number, maxLen: number = 25): PrefixMatch[] {
    const limit = Math.min(chars.length, start + maxLen);
    const matches: PrefixMatch[] = [];
    let cur = '';

    for (let i = start; i < limit; i++) {
      cur += chars[i];
      const w = this.prefixes.get(cur);
      if (w === undefined) {
        break; // No words share this prefix
      }
      if (w > 0) {
        matches.push({
          word: cur,
          end: i + 1,
          weight: w,
        });
      }
    }

    return matches;
  }

  /**
   * Load dictionary from a TSV string (format: word<TAB>weight).
   */
  static fromTsv(content: string): ThaiTrie {
    const trie = new ThaiTrie();
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;
      const parts = trimmed.split('\t');
      const word = parts[0];
      let weight = 1.0;
      if (parts.length >= 2) {
        const parsed = parseFloat(parts[1]);
        if (!isNaN(parsed) && parsed > 0) {
          weight = parsed;
        }
      }
      trie.add(word, weight);
    }
    return trie;
  }
}
