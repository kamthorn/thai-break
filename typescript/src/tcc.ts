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
    'c[ั]([่-๋]c)?',
    'c[ั]([่-๋]c)?k',
    'เc็ck',
    'เcctาะk',
    'เccีtยะk',
    'เccีtย(?=[เ-ไก-ฮ]|$|\\s)k',
    'เc[ิีุู]tย(?=[เ-ไก-ฮ]|$|\\s)k',
    'เcc็ck',
    'เcิc์ck',
    'เcิtck',
    'เcีtยะ?k',
    'เcืtอะk',
    'เcื',
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
    'แcc็ck',
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

  const offsetToCharIdx = new Map<number, number>();
  for (let i = 0; i <= n; i++) {
    offsetToCharIdx.set(charOffsets[i], i);
  }

  const totalLen = text.length;
  let pos = 0;

  while (pos < totalLen) {
    const sub = text.slice(pos);
    const m = pattern.exec(sub);
    if (m && m[0].length > 0) {
      pos += m[0].length;
      const idx = offsetToCharIdx.get(pos);
      if (idx !== undefined) {
        valid[idx] = true;
      }
    } else {
      // Fallback: advance 1 Unicode character
      const curIdx = offsetToCharIdx.get(pos) ?? 0;
      const chLen = chars[curIdx]?.length ?? 1;
      pos += chLen;
      const idx = offsetToCharIdx.get(pos);
      if (idx !== undefined) {
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

  return valid;
}
