import { Tokenizer } from './tokenizer.js';
import { ALLOWED, breakOpportunities } from './uax14.js';

export const DEFAULT_BREAK_MARKER = '\u200B';

const PAT_HTML_TAGS = /(<!--[\s\S]*?-->|<script\b[^>]*>[\s\S]*?<\/script>|<style\b[^>]*>[\s\S]*?<\/style>|<[^>]+>|&[a-zA-Z0-9#]+;)/gi;
const RE_THAI_COMBINING = /[\u0E31\u0E34-\u0E3A\u0E47-\u0E4E\u200B]/g;


/**
 * Whether a line break is permissible between two adjacent tokens. The tokens
 * are treated as separate dictionary words, so a Thai|Thai junction is a word
 * boundary.
 */
export function canBreakBetween(left: string, right: string): boolean {
  if (!left || !right) {
    return false;
  }

  const cps = codePoints(left + right);
  const at = codePoints(left).length;
  const dict: boolean[] = new Array(cps.length + 1).fill(false);
  dict[at] = true;

  // Whitespace safety: never insert break adjacent to spaces
  if (/\s$/.test(left) || /^\s/.test(right)) {
    return false;
  }

  return breakOpportunities(cps, dict)[at] === ALLOWED;
}

function codePoints(text: string): number[] {
  return Array.from(text, (ch) => ch.codePointAt(0) ?? 0);
}

/**
 * Calculates visual display width of Thai text.
 * Thai combining vowels/tone marks and ZWSP count as 0 width.
 * CJK fullwidth characters count as 2.
 */
export function thaiDisplayWidth(text: string): number {
  const stripped = text.replace(RE_THAI_COMBINING, '');
  let width = 0;
  for (const ch of stripped) {
    const cp = ch.codePointAt(0) ?? 0;
    if (
      (cp >= 0x2e80 && cp <= 0x9fff) ||
      (cp >= 0xac00 && cp <= 0xd7af) ||
      (cp >= 0xf900 && cp <= 0xfaff)
    ) {
      width += 2;
    } else {
      width += 1;
    }
  }
  return width;
}

export class LineBreaker {
  private tokenizer: Tokenizer;

  constructor(tokenizer: Tokenizer) {
    this.tokenizer = tokenizer;
  }

  /**
   * Insert line break opportunities (default: ZWSP U+200B).
   */
  insertLineBreaks(
    text: string,
    marker: string = DEFAULT_BREAK_MARKER,
    isHtml: boolean = false
  ): string {
    if (!text) return '';

    if (isHtml || (text.includes('<') && text.includes('>'))) {
      return this.processHtml(text, marker);
    }
    return this.processPlain(text, marker);
  }

  private processPlain(text: string, marker: string): string {
    let res = '';
    let prev = '';
    for (const seg of this.breakSegments(text)) {
      // Breaks after spaces and ZWSP are already implicit: never put a marker next to them
      if (prev && !/[\s\u200B]$/.test(prev)) {
        res += marker;
      }
      res += seg;
      prev = seg;
    }
    return res;
  }

  /**
   * Split plain text into segments that must not be broken internally. A line
   * may break between any two segments; spaces stay at the end of the segment
   * they follow.
   *
   * UAX #14 is evaluated at every position; the tokenizer only supplies the
   * word boundaries inside Thai runs.
   */
  private breakSegments(text: string): string[] {
    if (!text) return [];

    const chars = Array.from(text);
    const cps = chars.map((ch) => ch.codePointAt(0) ?? 0);
    const n = chars.length;
    const dict: boolean[] = new Array(n + 1).fill(false);
    let pos = 0;
    for (const tok of this.tokenizer.tokenize(text, true)) {
      pos += Array.from(tok).length;
      dict[Math.min(pos, n)] = true;
    }
    const actions = breakOpportunities(cps, dict);

    const segments: string[] = [];
    let start = 0;
    for (let i = 1; i < n; i++) {
      if (actions[i] === ALLOWED) {
        segments.push(chars.slice(start, i).join(''));
        start = i;
      }
    }
    segments.push(chars.slice(start).join(''));
    return segments;
  }

  private processHtml(html: string, marker: string): string {
    // Split into tags/entities and text chunks
    const parts = html.split(PAT_HTML_TAGS);
    let res = '';
    for (const part of parts) {
      if (!part) continue;
      if (
        (part.startsWith('<') && part.endsWith('>')) ||
        (part.startsWith('&') && part.endsWith(';'))
      ) {
        res += part;
      } else {
        res += this.processPlain(part, marker);
      }
    }
    return res;
  }

  /**
   * Hard-wrap text into lines with maximum visual display width, breaking only
   * at the same opportunities that insertLineBreaks() marks or after spaces.
   * Trailing spaces are trimmed.
   */
  wrap(text: string, width: number, isHtml: boolean = false): string {
    if (!text || width <= 0) return text;

    const wrappedParagraphs: string[] = [];
    for (const para of text.split(/\r?\n/)) {
      // Tags stay intact in insertLineBreaks(): break at its markers and after spaces
      const segments = isHtml
        ? this.insertLineBreaks(para, DEFAULT_BREAK_MARKER, true).split(/\u200B|(?<=\s)(?=\S)/)
        : this.breakSegments(para);
      wrappedParagraphs.push(fillLines(segments, width).join('\n'));
    }

    return wrappedParagraphs.join('\n');
  }
}

/** Greedily fill lines of at most `width` display columns with unbreakable segments. */
function fillLines(segments: string[], width: number): string[] {
  const lines: string[] = [];
  let curLine = '';
  let curWidth = 0;
  for (const seg of segments) {
    // Trailing spaces may hang past the margin, so only the visible part must fit
    const visibleWidth = thaiDisplayWidth(seg.trimEnd());
    if (curLine && curWidth + visibleWidth > width) {
      // Indentation that does not fit is dropped rather than left as an empty line
      if (curLine.trimEnd()) {
        lines.push(curLine.trimEnd());
      }
      curLine = '';
      curWidth = 0;
    }
    curLine += seg;
    curWidth += thaiDisplayWidth(seg);
  }
  if (curLine) {
    lines.push(curLine.trimEnd());
  }
  return lines;
}
