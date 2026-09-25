import { test } from 'node:test';
import * as assert from 'node:assert';
import * as fs from 'node:fs';
import * as path from 'node:path';
import { fileURLToPath } from 'node:url';
import { lbClass, isEastAsian, isExtPictUnassigned, breakOpportunities, NO_BREAK } from '../dist/uax14.js';

// Class ids in the order of the LB enum in src/uax14.ts.
const NAMES = (
  'AL BK CR LF NL SP ZW WJ GL CL CP EX IS SY OP QU QI QF NS B2 BA BB HY HH CB IN HL NU PR PO ' +
  'ID EB EM H2 H3 JL JV JT RI ZWJ CM AK AP AS VF VI SA SM'
).split(' ');

test('Line_Break classes', () => {
  const expected: [string, string][] = [
    ['a', 'AL'], ['1', 'NU'], [' ', 'SP'], ['-', 'HY'], ['–', 'BA'],
    ['(', 'OP'], [')', 'CP'], ['}', 'CL'], ['、', 'CL'], ['/', 'SY'],
    ['.', 'IS'], ['!', 'EX'], ['"', 'QU'], ['“', 'QI'], ['”', 'QF'],
    ['$', 'PR'], ['฿', 'PR'], ['\\', 'PR'], ['%', 'PO'], ['#', 'AL'],
    ['ก', 'SA'], ['ๆ', 'SA'], ['ั', 'SM'], ['่', 'SM'], ['๐', 'NU'],
    ['๚', 'BA'], ['中', 'ID'], ['가', 'H2'], ['א', 'HL'],
    ['​', 'ZW'], [' ', 'GL'], ['‍', 'ZWJ'], ['́', 'CM'],
    ['😀', 'ID'], ['\u{1F1F9}', 'RI'], ['\u{10FFFF}', 'AL'],
  ];
  for (const [ch, name] of expected) {
    const cp = ch.codePointAt(0)!;
    assert.strictEqual(NAMES[lbClass(cp)], name, `class of U+${cp.toString(16)}`);
  }
});

test('Line_Break flags', () => {
  assert.ok(isEastAsian('（'.codePointAt(0)!));
  assert.ok(!isEastAsian('('.codePointAt(0)!));
  assert.ok(isExtPictUnassigned(0x1fffd));
  assert.ok(!isExtPictUnassigned('😀'.codePointAt(0)!));
});

test('LineBreakTest conformance (SA resolved to AL, no dictionary)', (t) => {
  const file = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../testdata/LineBreakTest-16.0.0.txt');
  if (!fs.existsSync(file)) {
    t.skip('LineBreakTest data not found');
    return;
  }
  const failures: string[] = [];
  for (const line of fs.readFileSync(file, 'utf8').split('\n')) {
    if (!line || line.startsWith('#')) continue;
    const cps: number[] = [];
    const expected: boolean[] = [];
    for (const field of line.trim().split(/\s+/)) {
      if (field === '×') expected.push(false);
      else if (field === '÷') expected.push(true);
      else cps.push(parseInt(field, 16));
    }
    const actual = breakOpportunities(cps).map((a) => a !== NO_BREAK);
    if (expected.some((e, i) => actual[i] !== e)) failures.push(line);
  }
  assert.deepStrictEqual(failures.slice(0, 10), [], `${failures.length} LineBreakTest cases failed`);
});
