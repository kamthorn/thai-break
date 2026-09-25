import { test } from 'node:test';
import * as assert from 'node:assert';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { init, lines } from '../dist/index.js';
import { breakOpportunities, NO_BREAK, MANDATORY } from '../dist/uax14.js';

init({ dictPath: path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../data/words.txt') });

/** [name, input, expected output with '|' as the break marker] */
const LINE_BREAK_CASES: [string, string, string][] = [
  ['LB9 combining mark stays with its base', 'สวัสดี́ครับ', 'สวัสดี́|ครับ'],
];

test('insertLineBreaks follows UAX #14', () => {
  for (const [name, input, expected] of LINE_BREAK_CASES) {
    assert.strictEqual(lines(input, false, '|'), expected, name);
  }
});

test('Mandatory breaks', () => {
  const cps = Array.from('a\r\nb', (c) => c.codePointAt(0)!);
  assert.deepStrictEqual(breakOpportunities(cps), [NO_BREAK, NO_BREAK, NO_BREAK, MANDATORY, MANDATORY]);
});
