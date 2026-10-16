/**
 * The next position of a fixed terminator, remembered between lookups. Lookups
 * come from increasing positions, so a cached hit at or after the new start is
 * still the first one and a cached miss stays a miss: every character is
 * searched at most once.
 */
class Terminator {
  private pos: number | undefined;

  constructor(readonly needle: string) {}

  find(lower: string, from: number): number {
    if (this.pos !== undefined && (this.pos < 0 || this.pos >= from)) return this.pos;
    this.pos = lower.indexOf(this.needle, from);
    return this.pos;
  }
}

const isAsciiLetter = (c: string): boolean => (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z');
const isAsciiAlnum = (c: string): boolean => isAsciiLetter(c) || (c >= '0' && c <= '9');
const isAsciiWordChar = (c: string): boolean => isAsciiAlnum(c) || c === '_';

/**
 * Index ranges of the comments, script and style blocks, tags and entities in
 * `html`. Finds the same spans as
 * `/(<!--[\s\S]*?-->|<script\b[^>]*>[\s\S]*?<\/script>|<style\b[^>]*>[\s\S]*?<\/style>|<[a-zA-Z/!?][^>]*>|&[a-zA-Z0-9#]+;)/gi`
 * but in linear time: with that regex, every `<script>`, `<!--` or `<a` whose
 * terminator is missing scans to the end of the text again.
 */
export function markupSpans(html: string): Array<[number, number]> {
  // ASCII lowercasing keeps string indices
  const lower = html.replace(/[A-Z]+/g, (s) => s.toLowerCase());
  const commentEnd = new Terminator('-->');
  const tagEnd = new Terminator('>');
  const rawBlocks: Array<[string, Terminator]> = [
    ['<script', new Terminator('</script>')],
    ['<style', new Terminator('</style>')],
  ];

  const spans: Array<[number, number]> = [];
  const n = html.length;
  let i = 0;
  while (i < n) {
    let end = -1;
    const c = html[i];
    if (c === '<') {
      if (html.startsWith('<!--', i)) {
        const j = commentEnd.find(lower, i + 4);
        if (j >= 0) end = j + 3;
      }
      for (const [open, close] of rawBlocks) {
        const after = i + open.length;
        if (end < 0 && lower.startsWith(open, i) && (after === n || !isAsciiWordChar(html[after]))) {
          const k = tagEnd.find(lower, after);
          const j = k >= 0 ? close.find(lower, k + 1) : -1;
          if (j >= 0) end = j + close.needle.length;
        }
      }
      if (end < 0 && i + 1 < n) {
        const next = html[i + 1];
        if (isAsciiLetter(next) || next === '/' || next === '!' || next === '?') {
          const k = tagEnd.find(lower, i + 1);
          if (k >= 0) end = k + 1;
        }
      }
    } else if (c === '&') {
      let j = i + 1;
      while (j < n && (isAsciiAlnum(html[j]) || html[j] === '#')) j++;
      if (j > i + 1 && html[j] === ';') end = j + 1;
    }
    if (end < 0) {
      i++;
    } else {
      spans.push([i, end]);
      i = end;
    }
  }
  return spans;
}

/** Whether `text` has a tag: "<" then a letter, "/", "!" or "?", and a ">" later on. */
export function containsHtmlTag(text: string): boolean {
  const lastGt = text.lastIndexOf('>');
  for (let i = text.indexOf('<'); i >= 0 && i + 1 < lastGt; i = text.indexOf('<', i + 1)) {
    const next = text[i + 1];
    if (isAsciiLetter(next) || next === '/' || next === '!' || next === '?') return true;
  }
  return false;
}
