import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { init, words, tccPosArray } from '../dist/index.js';

init({ dictPath: path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../data/words.txt') });

/** [name, input, expected words joined by '|'] */
const SEGMENTATION_CASES: [string, string, string][] = [
  ['a word before a full stop is not cut into an abbreviation', 'เขากินข้าว.', 'เขา|กิน|ข้าว|.'],
  ['a word before a full stop, then more text', 'ฉันรักเธอ.ไปเที่ยวกัน', 'ฉัน|รัก|เธอ|.|ไป|เที่ยว|กัน'],
  ['abbreviations are still recognized', 'เมื่อ 5 มิ.ย. ที่ จ.พิษณุโลก', 'เมื่อ|5|มิ.ย.|ที่|จ.|พิษณุโลก'],
  ['an abbreviation after a word keeps the word whole', 'ในเขตจ.พิจิตร', 'ใน|เขต|จ.|พิจิตร'],
  ['an abbreviation after a word that ends like one', 'ในเดือนพ.ย.', 'ใน|เดือน|พ.ย.'],
  ['ties keep the earlier word whole', 'บอกว่าอึดอัด', 'บอก|ว่า|อึดอัด'],
  ['ties keep the earlier word whole (2)', 'ลาออกจากรองประธาน', 'ลาออก|จาก|รอง|ประธาน'],
  ['ก็ is not swallowed by the cluster before it', 'ทะเลก็สวย', 'ทะเล|ก็|สวย'],
  ['a final consonant before a vowel starts the next cluster', 'รึยัง', 'รึ|ยัง'],
];

test('long text is segmented to the end', () => {
  // Longer than the 50,000-edge limit that used to leave the rest of the text as one token
  const sentence = 'การประชุมสามัญผู้ถือหุ้นประจำปีจัดขึ้นที่โรงแรมในกรุงเทพมหานคร';
  const result = words(sentence.repeat(2000));
  assert.strictEqual(result.length, 2000 * words(sentence).length);
  assert.ok(result.every((w) => Array.from(w).length < 20));
});

test('TCC never splits before a vowel or tone mark', () => {
  for (const word of ['เมื่อ', 'เนื้อ', 'เบื่อ', 'ต้น', 'เกล็ด', 'เหม็น', 'ลั๊วะ']) {
    const chars = Array.from(word);
    const valid = tccPosArray(chars);
    chars.forEach((ch, i) => {
      if (i > 0 && /[\u0e30-\u0e3a\u0e45\u0e47-\u0e4e]/.test(ch)) {
        assert.ok(!valid[i], `boundary before ${ch} in ${word}`);
      }
    });
  }
});

test('segmentation', () => {
  for (const [name, input, expected] of SEGMENTATION_CASES) {
    assert.strictEqual(words(input).join('|'), expected, name);
  }
});
