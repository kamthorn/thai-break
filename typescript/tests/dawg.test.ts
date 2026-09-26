import { describe, it } from 'node:test';
import * as assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { ThaiTrie, Tokenizer } from '../dist/index.js';

const currentDir = path.dirname(fileURLToPath(import.meta.url));
const dawgPath = path.resolve(currentDir, '../../data/words.dawg');
const txtPath = path.resolve(currentDir, '../../data/words.txt');

describe('DAWG / FST Trie in TypeScript', () => {
  it('loads binary DAWG and matches TSV tokenization results 100%', () => {
    const dawgBuf = fs.readFileSync(dawgPath);
    const txtContent = fs.readFileSync(txtPath, 'utf-8');

    const trieDawg = ThaiTrie.fromBinary(dawgBuf);
    const trieTsv = ThaiTrie.fromTsv(txtContent);

    const tokDawg = new Tokenizer(trieDawg);
    const tokTsv = new Tokenizer(trieTsv);

    const testSentences = [
      'สถาบันวิจัยดาราศาสตร์แห่งชาติกระทรวงการอุดมศึกษาวิทยาศาสตร์วิจัยและนวัตกรรม',
      'กรุงเทพมหานครอมรรัตนโกสินทร์มหินทรายุธยามหาดิลกภพนพรัตน์ราชธานีบุรีรมย์',
      'ปัญญาประดิษฐ์กำลังเปลี่ยนแปลงวิถีชีวิตและระบบการทำงานของมนุษย์ทั่วโลก',
      'ทดสอบคำสั้นคำยาว กก กกหู กงการ กฎหมาย',
    ];

    for (const s of testSentences) {
      const resDawg = tokDawg.tokenize(s, false);
      const resTsv = tokTsv.tokenize(s, false);
      assert.deepStrictEqual(resDawg, resTsv, `Mismatch on sentence: ${s}`);
    }
  });

  it('supports dynamic word addition via overlay on DAWG', () => {
    const dawgBuf = fs.readFileSync(dawgPath);
    const trie = ThaiTrie.fromBinary(dawgBuf);

    const sample = 'สวัสดีชาวโลกและทดสอบคำแปลกใหม่จ้า';
    const tok = new Tokenizer(trie);
    const before = tok.tokenize(sample, false);

    // Dynamically register new word
    trie.add('คำแปลกใหม่', 2.0);
    const after = tok.tokenize(sample, false);

    assert.ok(after.includes('คำแปลกใหม่'), `Expected 'คำแปลกใหม่' in ${after.join('|')}`);
  });
});
