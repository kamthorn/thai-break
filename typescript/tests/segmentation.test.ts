import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { init, words } from '../dist/index.js';

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
];

test('segmentation', () => {
  for (const [name, input, expected] of SEGMENTATION_CASES) {
    assert.strictEqual(words(input).join('|'), expected, name);
  }
});
