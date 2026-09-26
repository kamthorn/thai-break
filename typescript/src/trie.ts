/**
 * Ultra-fast Word Lookup Trie supporting both Compact DAWG (FST / Minimal DFA)
 * and Flat Prefix Hash Map backends with runtime dynamic overlay.
 */

export interface PrefixMatch {
  word: string;
  end: number;
  weight: number;
}

/**
 * Ultra-compact Directed Acyclic Word Graph (DAWG) / Minimal DFA.
 * Uses zero-copy memory traversal over a byte buffer.
 */
export class CompactDawg {
  readonly data: Uint8Array;
  readonly view: DataView;
  readonly numStates: number;
  readonly numWords: number;
  readonly offsets: Uint32Array;

  constructor(buffer: ArrayBuffer | Uint8Array) {
    if (buffer instanceof Uint8Array) {
      this.data = buffer;
      this.view = new DataView(buffer.buffer, buffer.byteOffset, buffer.byteLength);
    } else {
      this.data = new Uint8Array(buffer);
      this.view = new DataView(buffer);
    }

    if (this.data.length < 12) {
      throw new Error('Invalid DAWG buffer: too short');
    }
    const magic = String.fromCharCode(this.data[0], this.data[1], this.data[2], this.data[3]);
    if (magic !== 'TBD1') {
      throw new Error(`Invalid DAWG magic: expected TBD1, got ${magic}`);
    }

    this.numStates = this.view.getUint32(4, true);
    this.numWords = this.view.getUint32(8, true);

    this.offsets = new Uint32Array(this.numStates);
    let curr = 12;
    for (let i = 0; i < this.numStates; i++) {
      this.offsets[i] = curr;
      const numEdges = this.data[curr] & 0x7F;
      curr += 1 + numEdges * 4;
    }
  }

  prefixesFromChars(chars: string[], start: number, maxLen: number = 25): PrefixMatch[] {
    const limit = Math.min(chars.length, start + maxLen);
    let state = 0;
    const matches: PrefixMatch[] = [];

    for (let idx = start; idx < limit; idx++) {
      const ch = chars[idx].charCodeAt(0);
      const off = this.offsets[state];
      const flags = this.data[off];
      const numEdges = flags & 0x7F;

      let low = 0;
      let high = numEdges - 1;
      let nextState = -1;
      const base = off + 1;

      while (low <= high) {
        const mid = (low + high) >> 1;
        const edgeOff = base + mid * 4;
        const edgeChar = this.view.getUint16(edgeOff, true);
        if (edgeChar === ch) {
          nextState = this.view.getUint16(edgeOff + 2, true);
          break;
        } else if (edgeChar < ch) {
          low = mid + 1;
        } else {
          high = mid - 1;
        }
      }

      if (nextState === -1) break;

      state = nextState;
      const nextOff = this.offsets[state];
      if ((this.data[nextOff] & 0x80) !== 0) {
        matches.push({
          word: chars.slice(start, idx + 1).join(''),
          end: idx + 1,
          weight: 1.0,
        });
      }
    }

    return matches;
  }
}

export class ThaiTrie {
  private dawg?: CompactDawg;
  private overlay: Map<string, number> = new Map();
  private prefixes: Map<string, number> = new Map();
  private _maxWeight: number = 1.0;
  private _totalWeight: number = 0.0;

  get maxWeight(): number {
    return this._maxWeight;
  }

  /** Sum of the weights of all words; word probabilities are weight / totalWeight. */
  get totalWeight(): number {
    return this._totalWeight;
  }

  get size(): number {
    if (this.dawg) {
      let extra = 0;
      for (const w of this.overlay.values()) {
        if (w > 0) extra++;
      }
      return this.dawg.numWords + extra;
    }
    let cnt = 0;
    for (const w of this.prefixes.values()) {
      if (w > 0) cnt++;
    }
    return cnt;
  }

  /**
   * Add a word with its frequency weight. Works on both DAWG and flat map backends.
   */
  add(word: string, weight: number): void {
    if (!word) return;
    if (weight > this._maxWeight) {
      this._maxWeight = weight;
    }

    const chars = Array.from(word);
    const n = chars.length;
    const targetMap = this.dawg ? this.overlay : this.prefixes;

    // Register all proper prefixes with weight 0 (continuation nodes)
    let p = '';
    for (let i = 0; i < n - 1; i++) {
      p += chars[i];
      if (!targetMap.has(p)) {
        targetMap.set(p, 0.0);
      }
    }

    // Register full word
    const existing = targetMap.get(word) ?? 0.0;
    if (weight > existing) {
      targetMap.set(word, weight);
      this._totalWeight += weight - existing;
    } else if (!targetMap.has(word)) {
      targetMap.set(word, weight);
      this._totalWeight += weight;
    }
  }

  /**
   * Find all matching dictionary words starting at character index `start`.
   */
  prefixesFromChars(chars: string[], start: number, maxLen: number = 25): PrefixMatch[] {
    if (this.dawg) {
      const matches = this.dawg.prefixesFromChars(chars, start, maxLen);
      if (this.overlay.size > 0) {
        const limit = Math.min(chars.length, start + maxLen);
        let cur = '';
        for (let i = start; i < limit; i++) {
          cur += chars[i];
          const w = this.overlay.get(cur);
          if (w === undefined) break;
          if (w > 0) {
            const existing = matches.find((m) => m.end === i + 1);
            if (existing) {
              existing.weight = w;
            } else {
              matches.push({ word: cur, end: i + 1, weight: w });
            }
          }
        }
      }
      return matches;
    }

    // Flat Map fallback
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
   * Load dictionary from a pre-compiled binary DAWG buffer.
   */
  static fromBinary(buffer: ArrayBuffer | Uint8Array): ThaiTrie {
    const trie = new ThaiTrie();
    trie.dawg = new CompactDawg(buffer);
    trie._maxWeight = 1.0;
    trie._totalWeight = trie.dawg.numWords;
    return trie;
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
