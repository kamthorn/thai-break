import { Tokenizer } from './tokenizer.js';

export const DEFAULT_BREAK_MARKER = '\u200B';

const PAT_NO_BREAK_AFTER = /^[([{"“‘<«฿$€¥£#@（【《]$/u;
const PAT_NO_BREAK_BEFORE = /^(?:[)\]}"”’>»,.:;!?ๆฯ๏๚๛）】》]|ฯลฯ)$/u;
const PAT_HTML_TAGS = /(<[^>]+>|&[a-zA-Z0-9#]+;)/g;
const RE_THAI_COMBINING = /[\u0E31\u0E34-\u0E3A\u0E47-\u0E4E\u200B]/g;

export function canBreakBetween(left: string, right: string): boolean {
  if (!left || !right) {
    return false;
  }

  // Whitespace safety: never insert break adjacent to spaces
  if (/\s$/.test(left) || /^\s/.test(right)) {
    return false;
  }

  // No break after opening symbols
  if (PAT_NO_BREAK_AFTER.test(left)) {
    return false;
  }

  // No break before closing symbols / postfixes (ๆ, ฯ)
  if (PAT_NO_BREAK_BEFORE.test(right)) {
    return false;
  }

  return true;
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
    const tokens = this.tokenizer.tokenize(text, true);
    const n = tokens.length;
    if (n <= 1) return text;

    let res = '';
    for (let i = 0; i < n; i++) {
      res += tokens[i];
      if (i + 1 < n && canBreakBetween(tokens[i], tokens[i + 1])) {
        res += marker;
      }
    }
    return res;
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
   * Hard-wrap text into lines with maximum visual display width.
   */
  wrap(text: string, width: number, isHtml: boolean = false): string {
    if (!text || width <= 0) return text;

    const broken = this.insertLineBreaks(text, DEFAULT_BREAK_MARKER, isHtml);
    const paragraphs = broken.split(/\r?\n/);
    const wrappedParagraphs: string[] = [];

    for (const para of paragraphs) {
      const units = para.split(DEFAULT_BREAK_MARKER);
      let curLine = '';
      let curWidth = 0;
      const lines: string[] = [];

      for (const unit of units) {
        const uWidth = thaiDisplayWidth(unit);
        if (curWidth + uWidth > width && curLine.length > 0) {
          lines.push(curLine);
          curLine = unit;
          curWidth = uWidth;
        } else {
          curLine += unit;
          curWidth += uWidth;
        }
      }
      if (curLine.length > 0) {
        lines.push(curLine);
      }
      wrappedParagraphs.push(lines.join('\n'));
    }

    return wrappedParagraphs.join('\n');
  }
}
