import { tccPosArray } from './tcc.js';
import { ThaiTrie } from './trie.js';
import { BigramModel } from './bigram.js';

/**
 * The longest dictionary word that is looked up, in characters (a base word has at most 19, and the
 * royal title พระบาทสมเด็จพระเจ้าอยู่หัว has 26).
 */
const MAX_WORD_LEN = 40;
/**
 * Names of the letters of the Latin alphabet in Thai. A name that starts another one comes first
 * (เอ็ม before เอ), so the longest name wins.
 */
const LETTER_NAMES = [
  'ดับเบิลยู', 'เอ็กซ์', 'แอล', 'เอ็ม', 'เอ็น', 'เอฟ', 'เอช', 'เอส', 'อาร์', 'คิว', 'แซด', 'ซี', 'ดี', 'จี', 'ไอ', 'เจ', 'เค',
  'โอ', 'พี', 'ยู', 'วี', 'วาย', 'บี', 'ที', 'อี', 'เอ',
].map((n) => Array.from(n));
/** Cost of a run of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น), relative to the cost of the rarest word. */
const LETTER_RUN_COST_FACTOR = 1.0;
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
/**
 * An out-of-vocabulary word may span up to this many TCC clusters. Such an edge competes with the
 * dictionary words, so a long unknown word is kept whole instead of being cut into short words.
 */
const OOV_MAX_CLUSTERS = 6;
/** Cost of an out-of-vocabulary edge of one cluster, relative to the cost of the rarest word. */
const OOV_COST_FACTOR = 3.0;
/**
 * Extra cost per further cluster of an out-of-vocabulary edge, relative to the cost of the rarest
 * word. Below about 0.5 an unknown word beats real words and F1 drops sharply.
 */
const OOV_CLUSTER_COST_FACTOR = 0.8;
/**
 * Frequent function words. An out-of-vocabulary edge may not start or end with one of them, nor
 * contain one at a cluster boundary, so an unknown word does not swallow its neighbours
 * (ฮิวจ์ส|ไม่|ได้, not ฮิวจ์สไม่ได้; ชวรัตน์|ที่|ลืม|ไว้, not ชวรัตน์ที่ลืมไว้): the same name would
 * otherwise be tokenized differently on its own and next to a function word, and a search for it
 * would miss the document.
 */
const OOV_BOUNDARY_WORDS = [
  'ที่', 'และ', 'ของ', 'ใน', 'ได้', 'ให้', 'ไม่', 'ว่า', 'เป็น',
  'มี', 'จะ', 'ไป', 'มา', 'ก็', 'กับ', 'แต่', 'หรือ', 'จาก',
  'โดย', 'เพื่อ', 'แล้ว', 'อยู่', 'นี้', 'นั้น', 'ซึ่ง', 'การ', 'ความ',
  'ต่อ', 'ถึง', 'ยัง', 'เมื่อ', 'ทั้ง', 'ตาม', 'ด้วย', 'อีก', 'คือ',
  'ไว้',
  // words that often come right before a name
  'นาย', 'นาง', 'พี่', 'น้อง', 'แพทย์', 'ตัว',
];
/**
 * Function words that may still occur inside an out-of-vocabulary word: they are frequent
 * syllables of transliterated names (อาห์มาดิเนจัด, อะมีรุ้ลฮัจย์).
 */
const OOV_INNER_ALLOWED = ['มา', 'มี'];

/**
 * Tables of OOV_BOUNDARY_WORDS. Word k is bit k of a mask; as bit operations have 32 bits, a mask is
 * two numbers (words 0-31 and words 32 up).
 */
const FW_CHARS = OOV_BOUNDARY_WORDS.map((f) => Array.from(f));
const FW_UTF16_LEN = OOV_BOUNDARY_WORDS.map((f) => f.length);
/** By first character, as an offset from U+0E00 */
const FW_BY_FIRST: { k: number; chars: string[] }[][] = Array.from({ length: 0x80 }, () => []);
let FW_INNER_LO = 0;
let FW_INNER_HI = 0;
OOV_BOUNDARY_WORDS.forEach((f, k) => {
  FW_BY_FIRST[FW_CHARS[k][0].charCodeAt(0) - 0x0e00].push({ k, chars: FW_CHARS[k] });
  if (!OOV_INNER_ALLOWED.includes(f)) {
    if (k < 32) FW_INNER_LO |= 1 << k;
    else FW_INNER_HI |= 1 << (k - 32);
  }
});

const PAT_NONTHAI = /^(?:[a-zA-Z]+(?:[-_'][a-zA-Z0-9]+)*|\d+(?:,\d+)*(?:\.\d+)?%?|[ \t]+|\r?\n|[^\u0e00-\u0e7fa-zA-Z0-9 \t\r\n])/u;
const PAT_ABBR = /^(?:(?:[เแโใไ]?[ก-ฮ][ัิีึืุู็่้๊๋]?|[ก-ฮ]{1,4})\.)+/u;

/** An edge from the current position to `to`. */
interface DagEdge {
  to: number;
  word: string;
  cost: number;
  /** true for an out-of-vocabulary edge, which is merged with neighbouring unknown words */
  unknown: boolean;
}

/**
 * Normalize common Thai spelling variants for dictionary matching:
 * เ + เ → แ, ํ + า → ำ, ํ + tone + า → tone + ำ (e.g. "นํ้า" → "น้ำ"), and a
 * tone mark typed after Sara Am is moved before it (ำ + tone → tone + ำ, "นำ้" → "น้ำ").
 * Returns the normalized characters and, for each of them, the index of the
 * original character it starts at (plus a final entry for the end). Token
 * boundaries never fall inside a replaced pair, so tokens map back to exact
 * slices of the original text.
 */
function normalizeForMatching(chars: string[]): [string[], number[]] {
  const norm: string[] = [];
  const orig: number[] = [];
  const n = chars.length;
  for (let i = 0; i < n; i++) {
    const next = chars[i + 1] ?? '';
    if (chars[i] === 'เ' && next === 'เ') {
      norm.push('แ');
      orig.push(i++);
    } else if (chars[i] === 'ำ' && isToneMark(next)) {
      norm.push(next, 'ำ');
      orig.push(i, i);
      i += 1;
    } else if (chars[i] === '\u0e4d' && next === 'า' && isToneMark(chars[i + 2] ?? '')) {
      norm.push(chars[i + 2], 'ำ');
      orig.push(i, i);
      i += 2;
    } else if (chars[i] === '\u0e4d' && next === 'า') {
      norm.push('ำ');
      orig.push(i++);
    } else if (chars[i] === '\u0e4d' && isToneMark(next) && chars[i + 2] === 'า') {
      norm.push(next, 'ำ');
      orig.push(i, i);
      i += 2;
    } else {
      norm.push(chars[i]);
      orig.push(i);
    }
  }
  orig.push(n);
  return [norm, orig];
}

function isToneMark(ch: string): boolean {
  return ch >= '่' && ch <= '๋';
}

/** Letter names (LETTER_NAMES) by first character, as an offset from U+0E00, in LETTER_NAMES order */
const LETTER_NAMES_BY_FIRST: string[][][] = Array.from({ length: 0x80 }, () => []);
for (const name of LETTER_NAMES) {
  LETTER_NAMES_BY_FIRST[name[0].charCodeAt(0) - 0x0e00].push(name);
}

/**
 * Whether an out-of-vocabulary edge may cover a character: Thai letters, vowels and marks, but not ๆ, ฯ,
 * digits or other symbols, which are tokens of their own.
 */
function isOovCode(code: number): boolean {
  return (
    (code >= 0x0e01 && code <= 0x0e2e) ||
    (code >= 0x0e30 && code <= 0x0e3a) ||
    (code >= 0x0e40 && code <= 0x0e45) ||
    (code >= 0x0e47 && code <= 0x0e4e)
  );
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

  /**
   * Word boundaries as UTF-16 offsets (JavaScript string indices) from 0 to `text.length`, in the
   * style of an ICU BreakIterator: `text.slice(b[i], b[i + 1])` are consecutive segments that
   * cover the text. Whitespace and punctuation are segments of their own.
   */
  boundaries(text: string): number[] {
    const offsets = [0];
    let pos = 0;
    for (const token of this.tokenize(text, true)) {
      pos += token.length;
      offsets.push(pos);
    }
    return offsets;
  }

  /**
   * Whether what the abbreviation pattern matched is really words and a full stop. The pattern
   * accepts any one to four consonant letters before a full stop, and อ ว ย count as consonants, so
   * it matches "ไป." in "ต่อไป." and "ของส." in "ของส.ส.". It is not an abbreviation when it starts
   * with a function word followed by more letters ("ของส.ส." is ของ + ส.ส.), or when it is one
   * dictionary word with a vowel or tone mark and a final full stop that is not followed by a letter
   * ("ต่อไป."), unless the dictionary has the abbreviation itself. A string of consonants only (มทส. นพ.)
   * stays an abbreviation even when the dictionary has it without the full stop. `end` is the index of
   * the character after the match.
   */
  private abbreviationIsWords(chars: string[], end: number, abbr: string): boolean {
    const first = abbr.split('.')[0];
    if (OOV_BOUNDARY_WORDS.some((f) => first.length > f.length && first.startsWith(f))) {
      return true;
    }
    const next = end < chars.length ? chars[end] : '';
    const nextIsThai = next >= '\u0e01' && next <= '\u0e5b';
    return (
      abbr.split('.').length === 2 &&
      abbr.endsWith('.') &&
      Array.from(first).length >= 2 &&
      /[^ก-ฮ]/u.test(first) &&
      !nextIsThai &&
      this.trie.has(first) &&
      !this.trie.has(abbr)
    );
  }

  tokenize(text: string, keepWhitespace: boolean = false): string[] {
    if (!text) {
      return [];
    }

    // Match against a normalized copy, but return the original characters
    const chars = Array.from(text);
    const [norm, origPos] = normalizeForMatching(chars);
    let tokens = this.segment(norm);
    if (norm.join('') !== text) {
      let pos = 0;
      tokens = tokens.map((tok) => {
        const end = pos + Array.from(tok).length;
        const original = chars.slice(origPos[pos], origPos[end]).join('');
        pos = end;
        return original;
      });
    }

    if (!keepWhitespace) {
      return tokens.filter((t) => t.trim().length > 0);
    }
    return tokens;
  }

  /**
   * Run the Viterbi segmentation over chars and return all tokens, including whitespace. Every edge
   * into j covers chars[from[j]..j), so the DP keeps no word and the tokens are cut out at the end.
   */
  private segment(chars: string[]): string[] {
    const text = chars.join('');
    const n = chars.length;
    if (n === 0) {
      return [];
    }

    const validPos = tccPosArray(chars);

    const codes = new Int32Array(n);
    for (let i = 0; i < n; i++) {
      codes[i] = chars[i].codePointAt(0) ?? 0;
    }

    // Bit k of fwStartsLo/Hi[p] is set when OOV_BOUNDARY_WORDS[k] begins at character p, and bit k of
    // fwEndsLo/Hi[p] when it ends there (Lo: words 0-31, Hi: words 32 up)
    const fwStartsLo = new Int32Array(n + 1);
    const fwStartsHi = new Int32Array(n + 1);
    const fwEndsLo = new Int32Array(n + 1);
    const fwEndsHi = new Int32Array(n + 1);
    for (let p = 0; p < n; p++) {
      const code = codes[p];
      if (code < 0x0e00 || code > 0x0e7f) {
        continue;
      }
      for (const { k, chars: fc } of FW_BY_FIRST[code - 0x0e00]) {
        const e = p + fc.length;
        if (e > n) continue;
        let same = true;
        for (let t = 1; t < fc.length; t++) {
          if (chars[p + t] !== fc[t]) {
            same = false;
            break;
          }
        }
        if (!same) continue;
        if (k < 32) {
          fwStartsLo[p] |= 1 << k;
          fwEndsLo[e] |= 1 << k;
        } else {
          fwStartsHi[p] |= 1 << (k - 32);
          fwEndsHi[e] |= 1 << (k - 32);
        }
      }
    }

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
    const dp = new Float64Array(n + 1).fill(Infinity);
    const from = new Int32Array(n + 1).fill(-1);
    const isUnk = new Uint8Array(n + 1);
    dp[0] = 0.0;

    // The edges from the current position: end, cost, and whether it is out-of-vocabulary
    const edgeTo: number[] = [];
    const edgeCost: number[] = [];
    const edgeUnk: boolean[] = [];

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
            isUnk[i] = 1;
            break;
          }
        }
      }

      if (i === n) {
        break;
      }

      // Edges starting at i
      let edges = 0;
      const code = codes[i];

      if (isThaiRune(code)) {
        // 1. Thai dictionary words starting at i
        for (const m of this.trie.prefixesFromChars(chars, i, MAX_WORD_LEN)) {
          if (m.end <= n && validPos[m.end]) {
            edgeTo[edges] = m.end;
            edgeCost[edges] = Math.log(normalizer / m.weight);
            edgeUnk[edges++] = false;
          }
        }

        // 2. Thai abbreviation patterns (the pattern needs a full stop within four characters)
        let dot = false;
        for (let t = i + 1, tEnd = Math.min(i + 4, n - 1); t <= tEnd; t++) {
          if (chars[t] === '.') {
            dot = true;
            break;
          }
        }
        const mAbbr = dot ? PAT_ABBR.exec(text.slice(charOffsets[i])) : null;
        if (mAbbr && mAbbr[0].length > 0) {
          const mStr = mAbbr[0];
          const abbrLen = Array.from(mStr).length;
          const j = i + abbrLen;
          if (j <= n && validPos[j] && !this.abbreviationIsWords(chars, j, mStr)) {
            const letters = abbrLen - mStr.split('.').length + 1;
            edgeTo[edges] = j;
            edgeCost[edges] = (ABBR_COST_FACTOR + ABBR_LETTER_COST_FACTOR * letters) * rareCost;
            edgeUnk[edges++] = false;
          }
        }

        // 3. Runs of two or more spelled-out letters (เอทีเอ็ม, ยูเอ็น, ดีเอสไอ)
        for (let end = i, letters = 0; ; ) {
          const e = end;
          const endCode = e < n ? codes[e] : 0;
          const names = endCode >= 0x0e00 && endCode <= 0x0e7f ? LETTER_NAMES_BY_FIRST[endCode - 0x0e00] : [];
          const name = names.find((nm) => e + nm.length <= n && validPos[e + nm.length] && nm.every((c, k) => chars[e + k] === c));
          if (!name) break;
          end += name.length;
          letters++;
          if (letters >= 2) {
            edgeTo[edges] = end;
            edgeCost[edges] = LETTER_RUN_COST_FACTOR * rareCost;
            edgeUnk[edges++] = false;
          }
        }

        // 4. Out-of-vocabulary words of 1..OOV_MAX_CLUSTERS TCC clusters
        let clusters = 0;
        const startsLo = fwStartsLo[i];
        const startsHi = fwStartsHi[i];
        oov: for (let j = i + 1; j <= n && isOovCode(codes[j - 1]); j++) {
          if (!validPos[j]) {
            continue;
          }
          clusters++;
          if (clusters > OOV_MAX_CLUSTERS) {
            break;
          }
          let hitsLo = startsLo | fwEndsLo[j];
          let hitsHi = startsHi | fwEndsHi[j];
          if ((hitsLo | hitsHi) !== 0) {
            // The edge borders a function word that is shorter than the edge
            const edgeLen = charOffsets[j] - charOffsets[i];
            let borders = false;
            for (; hitsLo !== 0; hitsLo ^= hitsLo & -hitsLo) {
              if (edgeLen > FW_UTF16_LEN[31 - Math.clz32(hitsLo & -hitsLo)]) {
                borders = true;
                break;
              }
            }
            for (; !borders && hitsHi !== 0; hitsHi ^= hitsHi & -hitsHi) {
              if (edgeLen > FW_UTF16_LEN[63 - Math.clz32(hitsHi & -hitsHi)]) {
                borders = true;
              }
            }
            if (borders) {
              // Every longer edge from i would contain a function word that ends here
              let innerLo = fwEndsLo[j] & FW_INNER_LO;
              let innerHi = fwEndsHi[j] & FW_INNER_HI;
              for (; innerLo !== 0; innerLo ^= innerLo & -innerLo) {
                const len = FW_CHARS[31 - Math.clz32(innerLo & -innerLo)].length;
                if (j - i > len && validPos[j - len]) break oov;
              }
              for (; innerHi !== 0; innerHi ^= innerHi & -innerHi) {
                const len = FW_CHARS[63 - Math.clz32(innerHi & -innerHi)].length;
                if (j - i > len && validPos[j - len]) break oov;
              }
              continue;
            }
          }
          edgeTo[edges] = j;
          edgeCost[edges] = (OOV_COST_FACTOR + OOV_CLUSTER_COST_FACTOR * (clusters - 1)) * rareCost;
          edgeUnk[edges++] = true;
        }
      } else {
        // 5. Non-Thai tokens
        const mNonThai = PAT_NONTHAI.exec(text.slice(charOffsets[i]));
        if (mNonThai && mNonThai[0].length > 0) {
          const j = i + Array.from(mNonThai[0]).length;
          if (j <= n && validPos[j]) {
            edgeTo[edges] = j;
            edgeCost[edges] = rareCost;
            edgeUnk[edges++] = false;
          }
        }
      }

      // Relax the edges
      const prev = this.bigramModel && from[i] >= 0 ? chars.slice(from[i], i).join('') : '';
      for (let e = 0; e < edges; e++) {
        const j = edgeTo[e];
        let cost = edgeCost[e];
        if (prev !== '') {
          const bonus = this.bigramModel!.getBonus(prev, chars.slice(i, j).join(''), j - i);
          cost = Math.max(0.01, cost - bonus);
        }

        // Ties go to the later start, i.e. the shorter last word (see TIE_EPSILON)
        const newCost = dp[i] + cost;
        if (newCost <= dp[j] + TIE_EPSILON) {
          dp[j] = newCost;
          from[j] = i;
          isUnk[j] = edgeUnk[e] ? 1 : 0;
        }
      }
    }

    // Traceback
    if (!isFinite(dp[n])) {
      return [text];
    }

    const spans: number[][] = [];
    let pos = n;
    while (pos > 0) {
      spans.push([from[pos], pos, isUnk[pos]]);
      pos = from[pos];
      if (pos < 0) {
        break;
      }
    }
    spans.reverse();

    // Syllable-based OOV chunking: merge consecutive unknown Thai clusters
    const tokens: string[] = [];
    let curChunk = '';

    for (const [start, end, unknown] of spans) {
      const tokStr = chars.slice(start, end).join('');
      if (unknown && isThaiString(tokStr)) {
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

    return tokens;
  }
}
