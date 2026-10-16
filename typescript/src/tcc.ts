/**
 * Thai Character Cluster (TCC) boundary detector for TypeScript / JavaScript.
 *
 * Implements the Theeramunkong 30-rule TCC grammar.
 * Characters within a TCC must never be split. Word boundaries may only occur
 * at TCC boundaries.
 */

let tccPattern: RegExp | null = null;

export function getTCCPattern(): RegExp {
  if (tccPattern !== null) {
    return tccPattern;
  }

  const c = '[ก-ฮ]';
  const t = '[่-๋]?';
  const d = '[ุู]';
  const k = '([ก-ฮ][ก-ฮ]?[ุูิ]?์)?';

  const rawRules = [
    // Mai Han-akat always has a final: -ัวะ, or a consonant (with or without a tone mark).
    'cั[่-๋]?วะ',
    'cั[่-๋]?ck',
    'cั[่-๋]?',
    'เc็ck',
    'เcctาะk',
    'เccีtยะk',
    'เccีtย(?=[เ-ไก-ฮ]|$|\\s)k',
    'เc[ิีุู]tย(?=[เ-ไก-ฮ]|$|\\s)k',
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

  const patterns = rawRules.map((rule) => {
    return rule
      .replaceAll('k', k)
      .replaceAll('c', c)
      .replaceAll('t', t)
      .replaceAll('d', d);
  });

  tccPattern = new RegExp(`^(?:${patterns.join('|')})`, 'u');
  return tccPattern;
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

  const pattern = getTCCPattern();
  const text = chars.join('');

  // Build character index to UTF-16 code unit offset mapping
  const charOffsets: number[] = [0];
  let offset = 0;
  for (const ch of chars) {
    offset += ch.length;
    charOffsets.push(offset);
  }

  const totalLen = text.length;
  // Character index at each code unit offset that starts a character, else -1
  const offsetToCharIdx = new Int32Array(totalLen + 1).fill(-1);
  for (let i = 0; i <= n; i++) {
    offsetToCharIdx[charOffsets[i]] = i;
  }
  let pos = 0;

  while (pos < totalLen) {
    const sub = text.slice(pos);
    const m = pattern.exec(sub);
    if (m && m[0].length > 0) {
      pos += clusterLength(m[0], sub.slice(m[0].length));
      const idx = offsetToCharIdx[pos];
      if (idx >= 0) {
        valid[idx] = true;
      }
    } else {
      // Fallback: advance 1 Unicode character
      const curIdx = Math.max(offsetToCharIdx[pos], 0);
      const chLen = chars[curIdx]?.length ?? 1;
      pos += chLen;
      const idx = offsetToCharIdx[pos];
      if (idx >= 0) {
        valid[idx] = true;
      }
    }
  }

  // Non-Thai characters are always safe break positions
  for (let i = 0; i < n; i++) {
    const code = chars[i].codePointAt(0) ?? 0;
    if (code < 0x0e00 || code > 0x0e7f) {
      valid[i] = true;
      valid[i + 1] = true;
    }
  }

  // Never a boundary before a dependent vowel or mark (e.g. inside "เมื่อ")
  for (let i = 1; i < n; i++) {
    if (isDependentChar(chars[i])) {
      valid[i] = false;
    }
  }

  return valid;
}

/** Dependent vowels and marks that can never start a cluster: ะ ั า ำ ิ–ฺ ๅ ็–๎. */
const DEPENDENT = /^[\u0e30-\u0e3a\u0e45\u0e47-\u0e4e]$/;

function isDependentChar(ch: string): boolean {
  if (ch.length !== 1) return false;
  const code = ch.charCodeAt(0);
  return (code >= 0x0e30 && code <= 0x0e3a) || code === 0x0e45 || (code >= 0x0e47 && code <= 0x0e4e);
}

/**
 * Length of a matched cluster. A final consonant followed by a dependent vowel
 * or mark starts the next cluster instead: "รึยัง" is "รึ" + "ยัง", not
 * "รึย" + "ัง". Except ว before ะ, which is part of the vowel -ัวะ ("ผัวะ").
 */
function clusterLength(match: string, rest: string): number {
  const last = match[match.length - 1];
  const next = rest[0] ?? '';
  if (match.length > 1 && last >= 'ก' && last <= 'ฮ' && DEPENDENT.test(next) && !(last === 'ว' && next === 'ะ')) {
    return match.length - 1;
  }
  return match.length;
}
