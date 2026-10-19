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

  it('rejects corrupt DAWG data instead of reading past it', () => {
    const header = (numStates: number, ...body: number[]) => {
      const data = new Uint8Array(12 + body.length);
      data.set([0x54, 0x42, 0x44, 0x31]);
      new DataView(data.buffer).setUint32(4, numStates, true);
      data.set(body, 12);
      return data;
    };
    const cases: Record<string, Uint8Array> = {
      'more states than bytes': header(0xffffffff, 0),
      'no states': header(0),
      'state runs past the end': header(1, 3, 0x61, 0, 0, 0),
      'edge to a state that is not': header(1, 1, 0x61, 0, 5, 0),
      truncated: header(2, 0),
    };
    for (const [name, data] of Object.entries(cases)) {
      assert.throws(() => ThaiTrie.fromBinary(data), /DAWG/, name);
    }
    assert.doesNotThrow(() => ThaiTrie.fromBinary(header(1, 0)));
    assert.doesNotThrow(() => ThaiTrie.fromBinary(fs.readFileSync(dawgPath)));
  });
});
