import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import * as fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import {
  init,
  words,
  lines,
  wrap,
  displayWidth,
  join,
  DEFAULT_BREAK_MARKER,
} from '../dist/index.js';

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const dictPath = path.resolve(currentDir, '../../data/words.txt');

// Initialize with project dictionary
if (fs.existsSync(dictPath)) {
  init({
    dictPath,
  });
}

test('Words tokenization', () => {
  const result = words('ฉันรักภาษาไทย');
  assert.deepStrictEqual(result, ['ฉัน', 'รัก', 'ภาษา', 'ไทย']);

  const joined = join('สวัสดีครับ', '|');
  assert.strictEqual(joined, 'สวัสดี|ครับ');
});

test('Typographic rules', () => {
  const text = 'เดินเล่นๆ (ที่นี่ๆ) กรุงเทพฯ ฿100';
  const broken = lines(text, false);

  assert.ok(!broken.includes(DEFAULT_BREAK_MARKER + 'ๆ'), 'Should not break before ๆ');
  assert.ok(!broken.includes(DEFAULT_BREAK_MARKER + 'ฯ'), 'Should not break before ฯ');
  assert.ok(!broken.includes('(' + DEFAULT_BREAK_MARKER), 'Should not break after (');
  assert.ok(!broken.includes(DEFAULT_BREAK_MARKER + ')'), 'Should not break before )');
  assert.ok(!broken.includes(' ' + DEFAULT_BREAK_MARKER), 'Should not break after space');
  assert.ok(!broken.includes(DEFAULT_BREAK_MARKER + ' '), 'Should not break before space');
});

test('Latin letters and digits stay together (UAX #14 LB23)', () => {
  const broken = lines('ตาม WP01 และมาตรฐาน ISO29110', false, '|');
  assert.ok(broken.includes('WP01'), broken);
  assert.ok(broken.includes('ISO29110'), broken);
});

test('No break before a solidus (UAX #14 LB13)', () => {
  const broken = lines('ISO/IEC 29110 กำหนดให้มี', false, '|');
  assert.ok(!broken.includes('|/'), broken);
});

test('HTML preservation', () => {
  const html = '<div class="title"><b>สวัสดี</b> &amp; ประเทศไทย</div><script>var x = "สวัสดีประเทศไทย";</script><!-- คอมเมนต์ -->';
  const broken = lines(html, true);

  assert.ok(broken.includes('<div class="title">'), 'Opening tag intact');
  assert.ok(broken.includes('&amp;'), 'Entity intact');
  assert.ok(broken.includes('<script>var x = "สวัสดีประเทศไทย";</script>'), 'Script intact');
  assert.ok(broken.includes('<!-- คอมเมนต์ -->'), 'Comment intact');
  assert.strictEqual(
    broken.replaceAll(DEFAULT_BREAK_MARKER, ''),
    html,
    'Clean text matches original HTML'
  );
});

test('Display width', () => {
  assert.strictEqual(displayWidth('ก'), 1);
  assert.strictEqual(displayWidth('ก็'), 1); // Tone/combining mark = 0
  assert.strictEqual(displayWidth('ที่'), 1); // Sara I + Mai Ek = 0
  assert.strictEqual(displayWidth('ภาษาไทย'), 7);
  assert.strictEqual(displayWidth('A'), 1);
  assert.strictEqual(displayWidth('中'), 2); // CJK = 2
});

test('Line wrapping', () => {
  const text = 'ฉันรักภาษาไทยมากที่สุดในโลก';
  const wrapped = wrap(text, 12);
  const lineArr = wrapped.split('\n');

  assert.ok(lineArr.length >= 2, 'Should wrap into multiple lines');
  for (const line of lineArr) {
    assert.ok(
      displayWidth(line) <= 15, // leeway for unbreakable word
      `Line display width ${displayWidth(line)} should be reasonable`
    );
  }
});

test('Line wrapping breaks at spaces', () => {
  const wrapped = wrap('the quick brown fox jumps over the lazy dog', 10);
  assert.deepStrictEqual(wrapped.split('\n'), [
    'the quick',
    'brown fox',
    'jumps over',
    'the lazy',
    'dog',
  ]);
});
