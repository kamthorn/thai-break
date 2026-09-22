/**
 * Bigram Collocation Model for Thai Word Tokenization in TypeScript.
 */

export class BigramModel {
  private bigrams: Map<string, number> = new Map();
  public alpha: number;

  constructor(alpha: number = 0.15) {
    this.alpha = alpha;
  }

  get size(): number {
    return this.bigrams.size;
  }

  /**
   * Add a bigram transition with frequency count.
   */
  add(prevWord: string, nextWord: string, count: number = 1.0): void {
    this.bigrams.set(`${prevWord}\t${nextWord}`, count);
  }

  /**
   * Calculate transition bonus for a word pair.
   * Higher frequency yields higher cost deduction.
   */
  getBonus(prevWord: string, nextWord: string, wordLen: number = 1): number {
    if (this.alpha <= 0.0 || !prevWord || !nextWord) {
      return 0.0;
    }

    const count = this.bigrams.get(`${prevWord}\t${nextWord}`);
    if (count === undefined) {
      return 0.0;
    }

    const denom = Math.max(1, wordLen);
    return (this.alpha * Math.log(1.0 + count)) / denom;
  }

  /**
   * Load bigram transitions from a TSV string (prev<TAB>next<TAB>count).
   */
  static fromTsv(content: string, alpha: number = 0.15): BigramModel {
    const model = new BigramModel(alpha);
    const lines = content.split(/\r?\n/);
    for (const line of lines) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith('#')) continue;
      const parts = trimmed.split('\t');
      if (parts.length < 3) continue;
      const [w1, w2, countStr] = parts;
      const count = parseFloat(countStr);
      if (!isNaN(count) && count > 0) {
        model.add(w1, w2, count);
      }
    }
    return model;
  }
}
