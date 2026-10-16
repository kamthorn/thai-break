import { test } from 'node:test';
import * as assert from 'node:assert';
import { containsHtmlTag, markupSpans } from '../dist/html.js';

// The regexes that markupSpans and containsHtmlTag replace
const PAT_HTML_TAGS = /(<!--[\s\S]*?-->|<script\b[^>]*>[\s\S]*?<\/script>|<style\b[^>]*>[\s\S]*?<\/style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;)/gi;
const PAT_HTML_DETECT = /<[a-zA-Z/!?][^>]*>/;

test('markupSpans and containsHtmlTag match the regexes', () => {
  const pieces = [
    '<', '>', '!', '-', '/', '?', '<!--', '-->', '<script', '</script>', '<SCRIPT', '</Script>',
    '<style', '</STYLE>', '<scripts', '<script_', ' ', '\n', 'a', 'Z', '&', '#', ';', '&amp;', 'ก', 'ไทย',
    '<b>', '</b>', '<ab',
  ];
  let seed = 0x2545f491;
  const next = (): number => {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    return seed >>> 0;
  };
  for (let n = 0; n < 20000; n++) {
    let text = '';
    for (let k = next() % 12; k > 0; k--) text += pieces[next() % pieces.length];
    const want = [...text.matchAll(PAT_HTML_TAGS)].map((m) => [m.index, m.index + m[0].length]);
    assert.deepStrictEqual(markupSpans(text), want, JSON.stringify(text));
    assert.strictEqual(containsHtmlTag(text), PAT_HTML_DETECT.test(text), JSON.stringify(text));
  }
});

test('unterminated markup takes linear time', () => {
  for (const piece of ['<script>', '<style x>', '<!-->', '<a', '<!--<script>']) {
    const text = piece.repeat(200000);
    const start = Date.now();
    markupSpans(text);
    containsHtmlTag(text);
    assert.ok(Date.now() - start < 2000, `${piece} took ${Date.now() - start} ms`);
  }
});
