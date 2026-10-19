/**
 * Thai Character Cluster (TCC) boundary detector for TypeScript / JavaScript.
 *
 * Implements the Theeramunkong 30-rule TCC grammar.
 * Characters within a TCC must never be split. Word boundaries may only occur
 * at TCC boundaries.
 */

import { TCC_DFA_GENERAL, TCC_DFA_LOOKAHEAD, type TccDfaTables } from './tcc_dfa.js';

const C = '[ก-ฮ]';
const T = '[่-๋]?';
const D = '[ุู]';
const K = '([ก-ฮ][ก-ฮ]?[ุูิ]?์)?';

/** The general cluster rules in order; the first rule that matches wins. */
const RAW_RULES = [
  // Mai Han-akat always has a final: -ัวะ, or a consonant (with or without a tone mark).
  'cั[่-๋]?วะ',
  'cั[่-๋]?ck',
  'cั[่-๋]?',
  'เc็ck',
  'เcctาะk',
  'เccีtยะk',
  'เ(?:c[รลว]|หc)็ck',
  'เcิc์ck',
  // A true cluster or ห-led onset between เ and the vowel belongs to the syllable (เครื่อง, เพลิง, เปล่า).
  'เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ิtck',
  'เcิtck',
  'เcีtยะ?k',
  'เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])ืtอะ?k',
  'เcืtอะk',
  'เcืtอ?k',
  'เ(?:[กขคตปพผบด][รล]|[กขค]ว|ทร|ห[งญนมยรลว])tาk',
  'เctา?ะ?k',
  'c[ึื]tck',
  'c[ะ-ู]tk',
  'c[ิุู]์',
  'cรรc์',
  '[ข-ฮ]็อck',
  'c็',
  'ct[ะาำ]?k',
  'แc็ck',
  'แcc์k',
  'แctะk',
  'แ(?:c[รลว]|หc)็ck',
  'แccc์k',
  'โctะk',
  '[เ-ไ]ctk',
  'ก็',
  'อึ',
  'หึ',
];

/**
 * Rules that apply only when the next character can start a syllable (see canFollowLookaheadCluster);
 * they are tried before the general rules.
 */
const LOOKAHEAD_RULES = ['เccีtยk', 'เc[ิีุู]tยk'];

function expandRule(rule: string): string {
  return rule.replaceAll('k', K).replaceAll('c', C).replaceAll('t', T).replaceAll('d', D);
}

let tccPattern: RegExp | null = null;
let lookaheadPattern: RegExp | null = null;

/** The general cluster rules as one regular expression anchored at the start of the string. */
export function getTCCPattern(): RegExp {
  if (tccPattern === null) {
    tccPattern = new RegExp(`^(?:${RAW_RULES.map(expandRule).join('|')})`, 'u');
  }
  return tccPattern;
}

/** The lookahead rules as one regular expression anchored at the start of the string. */
export function getTCCLookaheadPattern(): RegExp {
  if (lookaheadPattern === null) {
    lookaheadPattern = new RegExp(`^(?:${LOOKAHEAD_RULES.map(expandRule).join('|')})`, 'u');
  }
  return lookaheadPattern;
}

/** A DFA over classes of U+0E00..U+0E7F with its tables in typed arrays. */
class TccDfa {
  private readonly classes: number;
  private readonly start: number;
  private readonly cls: Uint8Array;
  private readonly trans: Uint16Array;
  private readonly isMatch: Uint8Array;

  constructor(tables: TccDfaTables) {
    this.classes = tables.classes;
    this.start = tables.start;
    this.cls = Uint8Array.from(tables.class);
    this.trans = Uint16Array.from(tables.trans);
    this.isMatch = Uint8Array.from(tables.match);
  }

  /** End of the match that starts at `codes[start]`, or -1. */
  find(codes: Int32Array, start: number): number {
    let state = this.start;
    let found = -1;
    for (let p = start; p < codes.length; p++) {
      const off = codes[p] - 0x0e00;
      if (off < 0 || off >= 128) break;
      state = this.trans[state * this.classes + this.cls[off]];
      if (state === 0) break;
      if (this.isMatch[state]) found = p + 1;
    }
    return found;
  }
}

// DFAs generated from the rules of the Rust core (tcc_dfa.ts): a match is the match of the first rule
// that matches, as with a regular expression of the rules in order.
const GENERAL = new TccDfa(TCC_DFA_GENERAL);
const LOOKAHEAD = new TccDfa(TCC_DFA_LOOKAHEAD);

/**
 * Whether a lookahead cluster (-ีย) may end before codes[i]: at the end of the text, or before a
 * consonant (ก-ฮ), a leading vowel (เ-ไ) or a character outside the Thai block.
 */
function canFollowLookaheadCluster(codes: Int32Array, i: number): boolean {
  if (i === codes.length) return true;
  const r = codes[i];
  return (r >= 0x0e01 && r <= 0x0e2e) || (r >= 0x0e40 && r <= 0x0e44) || r < 0x0e00 || r > 0x0e7f;
}

/**
 * Computes a boolean array of valid TCC break positions for the given character array.
 *
 * @param chars Unicode characters of the text (e.g. Array.from(text))
 * @returns boolean array of length chars.length + 1 where true indicates a safe break boundary
 */
export function tccPosArray(chars: string[]): boolean[] {
  const n = chars.length;
  const valid: boolean[] = new Array(n + 1).fill(false);
  valid[0] = true;
  valid[n] = true;

  if (n === 0) {
    return valid;
  }

  const codes = new Int32Array(n);
  for (let i = 0; i < n; i++) {
    codes[i] = chars[i].codePointAt(0) ?? 0;
  }
  let pos = 0;
  while (pos < n) {
    // First, the lookahead rules (all of them start with เ)
    if (codes[pos] === 0x0e40) {
      const end = LOOKAHEAD.find(codes, pos);
      if (end >= 0 && canFollowLookaheadCluster(codes, end)) {
        pos = end;
        valid[pos] = true;
        continue;
      }
    }

    // Second, the general rules; otherwise advance one character
    const end = GENERAL.find(codes, pos);
    pos = end >= 0 ? clusterEnd(codes, pos, end) : pos + 1;
    valid[pos] = true;
  }

  for (let i = 0; i < n; i++) {
    const code = codes[i];
    if (code < 0x0e00 || code > 0x0e7f) {
      // Non-Thai characters are always safe break positions
      valid[i] = true;
      valid[i + 1] = true;
    } else if (i > 0 && isDependentCode(code)) {
      // Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
      valid[i] = false;
    }
  }

  return valid;
}

/** Dependent vowels and marks that can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎. */
function isDependentCode(code: number): boolean {
  return (code >= 0x0e30 && code <= 0x0e3a) || code === 0x0e45 || (code >= 0x0e47 && code <= 0x0e4e);
}

/**
 * End of a matched cluster `codes[start..end)`. A final consonant followed by a dependent vowel
 * or mark starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not "รึย" + "ัง". Except ว
 * before ะ, which is part of the vowel -ัวะ ("ผัวะ").
 */
function clusterEnd(codes: Int32Array, start: number, end: number): number {
  if (end < codes.length && end - start > 1) {
    const last = codes[end - 1];
    const next = codes[end];
    if (last >= 0x0e01 && last <= 0x0e2e && isDependentCode(next) && !(last === 0x0e27 && next === 0x0e30)) {
      return end - 1;
    }
  }
  return end;
}
