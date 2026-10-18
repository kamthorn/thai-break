import { test } from 'node:test';
import * as assert from 'node:assert';
import { tccPosArray, getTCCPattern } from '../dist/index.js';

const DEPENDENT = /^[ะ-ฺๅ็-๎]$/;

/** The regular-expression implementation that the generated DFAs replaced, kept as the reference. */
function referenceTccPosArray(chars: string[]): boolean[] {
  const n = chars.length;
  const valid: boolean[] = new Array(n + 1).fill(false);
  valid[0] = true;
  valid[n] = true;
  if (n === 0) return valid;
  const pattern = getTCCPattern();
  const text = chars.join('');
  const offsetToCharIdx = new Map<number, number>();
  let offset = 0;
  for (let i = 0; i < n; i++) {
    offsetToCharIdx.set(offset, i);
    offset += chars[i].length;
  }
  offsetToCharIdx.set(offset, n);
  let pos = 0;
  while (pos < text.length) {
    const sub = text.slice(pos);
    const m = pattern.exec(sub);
    if (m && m[0].length > 0) {
      const match = m[0];
      const last = match[match.length - 1];
      const next = sub[match.length] ?? '';
      const trim = match.length > 1 && last >= 'ก' && last <= 'ฮ' && DEPENDENT.test(next) && !(last === 'ว' && next === 'ะ');
      pos += match.length - (trim ? 1 : 0);
    } else {
      pos += chars[offsetToCharIdx.get(pos) ?? 0].length;
    }
    const idx = offsetToCharIdx.get(pos);
    if (idx !== undefined) valid[idx] = true;
  }
  for (let i = 0; i < n; i++) {
    const code = chars[i].codePointAt(0) ?? 0;
    if (code < 0x0e00 || code > 0x0e7f) {
      valid[i] = true;
      valid[i + 1] = true;
    }
  }
  for (let i = 1; i < n; i++) {
    if (DEPENDENT.test(chars[i])) valid[i] = false;
  }
  return valid;
}

test('tccPosArray matches the regular expression rules on random strings', () => {
  // xorshift, so failures are reproducible
  let state = 0x2545f491;
  const next = () => {
    state ^= state << 13;
    state ^= state >>> 17;
    state ^= state << 5;
    return state >>> 0;
  };
  // Consonants and vowels are more likely than the other characters, so the rules often match
  const alphabet: string[] = [];
  for (let c = 0x0e01; c <= 0x0e5b; c++) alphabet.push(String.fromCodePoint(c));
  for (let c = 0x0e01; c <= 0x0e2e; c++) alphabet.push(String.fromCodePoint(c), String.fromCodePoint(c));
  for (const ch of 'เแโใไะาำิีึืุู็่้๊๋์ๆ') alphabet.push(ch, ch, ch);
  alphabet.push('a', ' ', ' ', '1', '.', ')', '😀');
  for (let i = 0; i < 100_000; i++) {
    const chars = Array.from({ length: 1 + (next() % 40) }, () => alphabet[next() % alphabet.length]);
    assert.deepStrictEqual(tccPosArray(chars), referenceTccPosArray(chars), JSON.stringify(chars.join('')));
  }
});

test('tccPosArray keeps the lookahead rules of the -ีย vowel', () => {
  for (const text of ['เสียง', 'เปลี่ยน', 'เขียน ', 'เมีย', 'เสีย)', 'เกี่ยว', 'เลี้ยง']) {
    const chars = Array.from(text);
    assert.deepStrictEqual(tccPosArray(chars), referenceTccPosArray(chars), text);
  }
});
