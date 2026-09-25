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
  ['LB25 no break inside a date', 'วันที่ 1/2/2567 นะ', 'วัน|ที่ 1/2/2567 นะ'],
  ['LB25 no break inside a time', 'เวลา 10:30 น.', 'เวลา 10:30 น.'],
  ['LB25 no break inside a range or a signed number', 'ช่วง 10-20 คน ลบ -5 องศา', 'ช่วง 10-20 คน ลบ -5 องศา'],
  ['LB25 no break after a prefix or before a postfix', 'ราคา $(5) ลด 40%', 'ราคา $(5) ลด 40%'],
  ['LB25 no break between IS and NU', 'พ.ศ.2567', 'พ.ศ.2567'],
  ['LB29 no break after a full stop before a letter', 'รพ.ศิริราช', 'รพ.ศิริราช'],
  ['LB29 no break inside an abbreviation', 'e.g.ไทย', 'e.g.ไทย'],
  ['LB29 no break after an ellipsis of full stops', 'ทดสอบ...ต่อ', 'ทด|สอบ...ต่อ'],
  ['LB28 no break between Thai and Latin letters', 'ภาษาPHPเป็น', 'ภาษาPHPเป็น'],
  ['LB28 no break inside an email address', 'ติดต่อ user@example.com ได้', 'ติดต่อ user@example.com ได้'],
  ['LB28 no break around # (AL)', 'แท็ก#ไทยดี', 'แท็ก#ไทย|ดี'],
  ['LB28 no break inside a Latin word with a combining mark', 'café́ ไทย', 'café́ ไทย'],
  ['LB23 no break between Thai letters and digits', 'ราคา100บาท', 'ราคา100บาท'],
  ['LB23 no break between Thai letters and Thai digits', 'ปี๒๕๖๗นะ', 'ปี๒๕๖๗นะ'],
  ['LB23 no break between Latin letters and digits', 'เอกสารWP01ของ', 'เอกสารWP01ของ'],
  ['LB30 no break between a letter and an opening parenthesis', 'ประเทศไทย(สยาม)เป็นประเทศ', 'ประเทศ|ไทย(สยาม)เป็น|ประเทศ'],
  ['LB30 no break between a closing bracket and a letter', '[หมายเหตุ]ข้อความ', '[หมายเหตุ]ข้อความ'],
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
  ['LB14 no break after an opening parenthesis, even after spaces', 'ข้อความ ( ไทย ) ต่อ', 8, ['ข้อความ', '( ไทย )', 'ต่อ']],
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
