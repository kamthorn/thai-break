import { tccPosArray } from './tcc.js';
import { ThaiTrie } from './trie.js';
import { BigramModel } from './bigram.js';

/** Cost of an abbreviation pattern, relative to the cost of the rarest word. */
const ABBR_COST_FACTOR = 1.5;
/**
 * Extra cost per letter of an abbreviation pattern, so "เขต|จ." beats "เข|ตจ."
 * (a pattern taking the last letter of the previous word).
 */
const ABBR_LETTER_COST_FACTOR = 0.01;
/** Cost of an unknown-word fallback edge, relative to the cost of the rarest word. */
const UNKNOWN_COST_FACTOR = 2.0;
/**
 * Costs closer than this are a tie. Edges into a position are visited from
 * the longest word to the shortest, so on a tie the later, shorter last word
 * wins and the earlier words stay longer ("ผิด|ราย" rather than "ผิ|ดราย").
 */
const TIE_EPSILON = 1e-9;

const PAT_NONTHAI = /^(?:[a-zA-Z]+(?:[-_'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\u0e00-\u0e7fa-zA-Z0-9 \t\r\n])/u;
const PAT_ABBR = /^(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+/u;

/** An edge from the current position to `to`. */
interface DagEdge {
  to: number;
  word: string;
  cost: number;
}

function isThaiRune(code: number): boolean {
  return code >= 0x0e00 && code <= 0x0e7f;
}

function isThaiString(str: string): boolean {
  if (!str) return false;
  for (let i = 0; i < str.length; i++) {
    const code = str.codePointAt(i) ?? 0;
    if (!isThaiRune(code)) {
      return false;
    }
  }
  return true;
}

export class Tokenizer {
  private trie: ThaiTrie;
  private bigramModel?: BigramModel;

  constructor(trie: ThaiTrie, bigramModel?: BigramModel) {
    this.trie = trie;
    this.bigramModel = bigramModel;
  }

  setBigramModel(model?: BigramModel): void {
    this.bigramModel = model;
  }

  tokenize(text: string, keepWhitespace: boolean = false): string[] {
    if (!text) {
      return [];
    }

    const chars = Array.from(text);
    const n = chars.length;
    if (n === 0) {
      return [];
    }

    const validPos = tccPosArray(chars);

    // Precompute character index to code unit offset mapping
    const charOffsets: number[] = [0];
    let offset = 0;
    for (const ch of chars) {
      offset += ch.length;
      charOffsets.push(offset);
    }

    // Unigram costs: -log(weight / total); a word of weight 1 costs rareCost.
    // Non-Thai tokens cost rareCost, abbreviation patterns and unknown-word
    // fallbacks more, so a dictionary word followed by "." beats a pattern such
    // as "ว." that would cut the word.
    const normalizer = this.trie.totalWeight + 1.0;
    const rareCost = Math.log(normalizer);

    // Single-pass Viterbi DP. Positions are visited in order. When position i
    // is reached, every edge into it has been relaxed, so dp[i] is final: an
    // unreachable i gets an unknown-word edge, then the edges starting at i are
    // relaxed right away. Edges are never stored, so memory stays O(n) for any
    // text length.
    const dp: number[] = new Array(n + 1).fill(Infinity);
    const from: number[] = new Array(n + 1).fill(-1);
    const word: string[] = new Array(n + 1).fill('');
    const isUnk: boolean[] = new Array(n + 1).fill(false);
    dp[0] = 0.0;

    for (let i = 0; i <= n; i++) {
      if (!validPos[i]) {
        continue;
      }

      // Unknown-word fallback: connect to nearest reachable predecessor
      if (!isFinite(dp[i])) {
        for (let k = i - 1; k >= 0; k--) {
          if (isFinite(dp[k]) && validPos[k]) {
            dp[i] = dp[k] + UNKNOWN_COST_FACTOR * rareCost;
            from[i] = k;
            word[i] = chars.slice(k, i).join('');
            isUnk[i] = true;
            break;
          }
        }
      }

      if (i === n) {
        break;
      }

      // Edges starting at i
      const edges: DagEdge[] = [];
      const code = chars[i].codePointAt(0) ?? 0;
      const subText = text.slice(charOffsets[i]);

      if (isThaiRune(code)) {
        // 1. Thai dictionary words starting at i
        for (const m of this.trie.prefixesFromChars(chars, i, 25)) {
          if (m.end <= n && validPos[m.end]) {
            edges.push({ to: m.end, word: m.word, cost: Math.log(normalizer / m.weight) });
          }
        }

        // 2. Thai abbreviation patterns
        const mAbbr = PAT_ABBR.exec(subText);
        if (mAbbr && mAbbr[0].length > 0) {
          const mStr = mAbbr[0];
          const abbrLen = Array.from(mStr).length;
          const j = i + abbrLen;
          if (j <= n && validPos[j]) {
            const letters = abbrLen - mStr.split('.').length + 1;
            edges.push({ to: j, word: mStr, cost: (ABBR_COST_FACTOR + ABBR_LETTER_COST_FACTOR * letters) * rareCost });
          }
        }
      } else {
        // 3. Non-Thai tokens
        const mNonThai = PAT_NONTHAI.exec(subText);
        if (mNonThai && mNonThai[0].length > 0) {
          const j = i + Array.from(mNonThai[0]).length;
          if (j <= n && validPos[j]) {
            edges.push({ to: j, word: mNonThai[0], cost: rareCost });
          }
        }
      }

      // Relax the edges
      for (const edge of edges) {
        const j = edge.to;
        let edgeCost = edge.cost;
        if (this.bigramModel && word[i] !== '') {
          const bonus = this.bigramModel.getBonus(word[i], edge.word, j - i);
          edgeCost = Math.max(0.01, edge.cost - bonus);
        }

        // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
        const newCost = dp[i] + edgeCost;
        if (newCost <= dp[j] + TIE_EPSILON) {
          dp[j] = newCost;
          from[j] = i;
          word[j] = edge.word;
          isUnk[j] = false;
        }
      }
    }

    // Traceback
    if (!isFinite(dp[n])) {
      return [text];
    }

    const rawTokens: string[] = [];
    const rawIsUnk: boolean[] = [];
    let pos = n;
    while (pos > 0) {
      rawTokens.push(word[pos]);
      rawIsUnk.push(isUnk[pos]);
      pos = from[pos];
      if (pos < 0) {
        break;
      }
    }

    rawTokens.reverse();
    rawIsUnk.reverse();

    // Syllable-based OOV chunking: merge consecutive unknown Thai clusters
    const tokens: string[] = [];
    let curChunk = '';

    for (let idx = 0; idx < rawTokens.length; idx++) {
      const tokStr = rawTokens[idx];
      if (rawIsUnk[idx] && isThaiString(tokStr)) {
        curChunk += tokStr;
      } else {
        if (curChunk.length > 0) {
          tokens.push(curChunk);
          curChunk = '';
        }
        tokens.push(tokStr);
      }
    }
    if (curChunk.length > 0) {
      tokens.push(curChunk);
    }

    if (!keepWhitespace) {
      return tokens.filter((t) => t.trim().length > 0);
    }

    return tokens;
  }
}
