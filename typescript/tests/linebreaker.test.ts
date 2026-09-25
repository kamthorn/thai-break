import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { init, lines, wrap } from '../dist/index.js';
import { breakOpportunities, NO_BREAK, MANDATORY } from '../dist/uax14.js';

init({ dictPath: path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../data/words.txt') });

/** [name, input, expected output with '|' as the break marker] */
const LINE_BREAK_CASES: [string, string, string][] = [
  ['LB9 combining mark stays with its base', 'สวัสดี́ครับ', 'สวัสดี́|ครับ'],
  ['LB21 no break before a hyphen', 'สี-ขาว', 'สี-|ขาว'],
  ['LB21 no break before an en dash (BA)', 'ข้อความ–ข้อความ', 'ข้อความ–|ข้อความ'],
  ['LB13 no break before CJK closing punctuation (CL)', 'ไทย、ไทย', 'ไทย、|ไทย'],
  ['LB13 no break before ? (EX)', 'ไปไหม?ไปสิ', 'ไป|ไหม?|ไป|สิ'],
  ['LB13 no break before a solidus (SY)', 'ISO/IEC 29110', 'ISO/|IEC 29110'],
];

test('insertLineBreaks follows UAX #14', () => {
  for (const [name, input, expected] of LINE_BREAK_CASES) {
    assert.strictEqual(lines(input, false, '|'), expected, name);
  }
});

/** [name, input, width, expected lines] */
const WRAP_CASES: [string, string, number, string[]][] = [
  ['an opening bracket never ends a line', 'ประชาชน (ทั่วประเทศ) ไป', 9, ['ประชาชน', '(ทั่ว', 'ประเทศ)', 'ไป']],
  ['a dash never starts a line', 'ภาษาไทย–อังกฤษ', 7, ['ภาษา', 'ไทย–', 'อังกฤษ']],
  ['mai yamok never starts a line', 'ทดสอบเด็กๆๆๆๆๆๆ', 5, ['ทดสอบ', 'เด็กๆๆๆๆๆๆ']],
  ['an opening quote never ends a line', 'สวัสดีครับ “ท่านผู้ชม”', 11, ['สวัสดีครับ', '“ท่านผู้ชม”']],
  ['indentation that does not fit is dropped', '  ย่อหน้า ใหม่ ครับ', 6, ['ย่อหน้า', 'ใหม่', 'ครับ']],
];

test('wrap breaks only at UAX #14 opportunities', () => {
  for (const [name, input, width, expected] of WRAP_CASES) {
    assert.deepStrictEqual(wrap(input, width).split('\n'), expected, name);
  }
});

test('Mandatory breaks', () => {
  const cps = Array.from('a\r\nb', (c) => c.codePointAt(0)!);
  assert.deepStrictEqual(breakOpportunities(cps), [NO_BREAK, NO_BREAK, NO_BREAK, MANDATORY, MANDATORY]);
});
