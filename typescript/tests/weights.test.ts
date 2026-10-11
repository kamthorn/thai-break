import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import * as fs from 'node:fs';
import { fileURLToPath } from 'node:url';
import { ThaiTrie, Tokenizer } from '../dist/index.js';

const dataDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../data');

test('setWeight replaces the weight of a dictionary word', () => {
  const tries = {
    flat: ThaiTrie.fromTsv('แมว\nแมวน้ำ\nหมา\n'),
    dawg: ThaiTrie.fromBinary(fs.readFileSync(path.join(dataDir, 'words.dawg'))),
  };
  for (const [name, trie] of Object.entries(tries)) {
    const base = trie.totalWeight;
    assert.ok(trie.setWeight('แมว', 5), name);
    assert.strictEqual(trie.totalWeight, base + 4, name);
    assert.ok(trie.setWeight('แมว', 2), name);
    assert.strictEqual(trie.totalWeight, base + 1, name);
    assert.ok(!trie.setWeight('ฟลีบโลก', 3), `${name}: a word outside the dictionary is not added`);
    const cat = trie.prefixesFromChars(Array.from('แมวน้ำ'), 0).find((m) => m.end === 3);
    assert.strictEqual(cat?.weight, 2, name);
    assert.strictEqual(trie.applyWeights('# comment\nหมา\t3.5\nฟลีบโลก\t2\n'), 1, name);
    assert.throws(() => trie.applyWeights('หมา\tx\n'), name);
  }
});

test('the bundled word weights load onto the bundled dictionary', () => {
  const trie = ThaiTrie.fromBinary(fs.readFileSync(path.join(dataDir, 'words.dawg')));
  assert.ok(trie.applyWeights(fs.readFileSync(path.join(dataDir, 'word-weights.tsv'), 'utf-8')) > 1000);
  assert.ok(trie.maxWeight > 1);
});

test('dictionary words of up to 40 characters are matched', () => {
  // พระบาทสมเด็จพระเจ้าอยู่หัว has 26 characters, one more than the old limit of 25
  const title = 'พระบาทสมเด็จพระเจ้าอยู่หัว';
  const long = 'สมเด็จพระนางเจ้าสิริกิติ์พระบรมราชินีนาถ';
  assert.strictEqual(Array.from(long).length, 40);
  const tokenizer = new Tokenizer(ThaiTrie.fromTsv(`พระ\nบาท\nสมเด็จ\n${title}\n${long}\nเสด็จ\n`));
  for (const w of [title, long]) {
    assert.deepStrictEqual(tokenizer.tokenize(`${w}เสด็จ`), [w, 'เสด็จ']);
  }
});
