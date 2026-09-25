import { test } from 'node:test';
import * as assert from 'node:assert';
import { lbClass, isEastAsian, isExtPictUnassigned } from '../dist/uax14.js';

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
