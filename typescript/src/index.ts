import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';
import { ThaiTrie } from './trie.js';
import { BigramModel } from './bigram.js';
import { Tokenizer } from './tokenizer.js';
import { LineBreaker, thaiDisplayWidth, DEFAULT_BREAK_MARKER } from './linebreaker.js';

export * from './tcc.js';
export * from './trie.js';
export * from './bigram.js';
export * from './tokenizer.js';
export * from './linebreaker.js';

let defaultTokenizer: Tokenizer | null = null;
let defaultLineBreaker: LineBreaker | null = null;

function tryLoadDefault(): void {
  if (defaultTokenizer) return;

  let trie = new ThaiTrie();
  let bigrams: BigramModel | undefined;

  // Attempt to find data directory under Node.js
  if (typeof process !== 'undefined' && process?.versions?.node) {
    const currentDir = path.dirname(fileURLToPath(import.meta.url));
    const candidates = [
      path.resolve(currentDir, '../../data/words.txt'),
      path.resolve(currentDir, '../data/words.txt'),
      path.resolve(process.cwd(), 'data/words.txt'),
    ];

    for (const c of candidates) {
      try {
        if (fs.existsSync(c)) {
          const content = fs.readFileSync(c, 'utf-8');
          trie = ThaiTrie.fromTsv(content);

          const bigramFile = path.join(path.dirname(c), 'bigrams.tsv');
          if (fs.existsSync(bigramFile)) {
            const bContent = fs.readFileSync(bigramFile, 'utf-8');
            bigrams = BigramModel.fromTsv(bContent);
          }
          break;
        }
      } catch {
        // ignore and fallback
      }
    }
  }

  defaultTokenizer = new Tokenizer(trie, bigrams);
  defaultLineBreaker = new LineBreaker(defaultTokenizer);
}

export interface ThaiBreakInitOptions {
  dictTsv?: string;
  bigramsTsv?: string;
  dictPath?: string;
  bigramPath?: string;
  tokenizer?: Tokenizer;
}

/**
 * Configure or re-initialize the shared global ThaiBreak instance.
 */
export function init(options: ThaiBreakInitOptions): void {
  if (options.tokenizer) {
    defaultTokenizer = options.tokenizer;
    defaultLineBreaker = new LineBreaker(defaultTokenizer);
    return;
  }

  let trie = new ThaiTrie();
  let bigrams: BigramModel | undefined;

  if (options.dictTsv) {
    trie = ThaiTrie.fromTsv(options.dictTsv);
  } else if (options.dictPath && typeof fs !== 'undefined') {
    const content = fs.readFileSync(options.dictPath, 'utf-8');
    trie = ThaiTrie.fromTsv(content);
  }

  if (options.bigramsTsv) {
    bigrams = BigramModel.fromTsv(options.bigramsTsv);
  } else if (options.bigramPath && typeof fs !== 'undefined') {
    const bContent = fs.readFileSync(options.bigramPath, 'utf-8');
    bigrams = BigramModel.fromTsv(bContent);
  }

  defaultTokenizer = new Tokenizer(trie, bigrams);
  defaultLineBreaker = new LineBreaker(defaultTokenizer);
}

function getBreaker(): LineBreaker {
  if (!defaultLineBreaker) {
    tryLoadDefault();
  }
  return defaultLineBreaker!;
}

function getTokenizer(): Tokenizer {
  if (!defaultTokenizer) {
    tryLoadDefault();
  }
  return defaultTokenizer!;
}

/**
 * Segment Thai text into an array of words.
 */
export function words(text: string, keepWhitespace: boolean = false): string[] {
  return getTokenizer().tokenize(text, keepWhitespace);
}

/**
 * Insert break opportunities (default: ZWSP U+200B) into text.
 */
export function lines(
  text: string,
  isHtml: boolean = false,
  marker: string = DEFAULT_BREAK_MARKER
): string {
  return getBreaker().insertLineBreaks(text, marker, isHtml);
}

/**
 * Hard-wrap text into lines with maximum visual display width.
 */
export function wrap(text: string, width: number, isHtml: boolean = false): string {
  return getBreaker().wrap(text, width, isHtml);
}

/**
 * Calculate visual terminal/column display width of Thai text.
 */
export function displayWidth(text: string): number {
  return thaiDisplayWidth(text);
}

/**
 * Join segmented words with a custom separator (e.g. "|").
 */
export function join(text: string, glue: string = '|'): string {
  return words(text).join(glue);
}

export default {
  init,
  words,
  lines,
  wrap,
  displayWidth,
  join,
  Tokenizer,
  LineBreaker,
  ThaiTrie,
  BigramModel,
};
