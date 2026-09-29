#!/usr/bin/env python3
"""
tools/select_names.py

Selects which Thai given names from a names corpus (default: PyThaiNLP's
person_names_female_th.txt / person_names_male_th.txt, Apache-2.0) are safe to
merge into data/words.txt as regular dictionary entries.

Why: many two-syllable Thai given names are composed of two morphemes that are
*both* already real dictionary words (e.g. สมศักดิ์ = สม + ศักดิ์), so the
Viterbi tokenizer correctly-by-the-rules but wrongly-in-meaning splits them
(สมศักดิ์ -> สม|ศักดิ์) since a single dictionary word always beats a two-word
decomposition once the whole name is itself a listed word.

A name is excluded ("risky") when its surface form also occurs as an ordinary
two-word sequence *outside* any person-name (PER) span in the LST20
gold-segmented corpus at least twice — meaning it is also common vocabulary
(e.g. "โชคดี" = lucky), and forcing it to always merge would wrongly glue
together ordinary sentences that happen to contain it (LST20 is used only to
validate/select which words to add; no LST20 text is copied into words.txt).

Usage:
  python3 tools/select_names.py [names_in.txt ...] [--lst20 DIR] [--out FILE] [--min-risk N]

Default: reads PyThaiNLP's female+male name lists if given as arguments,
writes the safe subset to stdout (one word per line) for merging into
data/words.txt, e.g.:

  python3 tools/select_names.py person_names_female_th.txt person_names_male_th.txt \\
      --lst20 /path/to/LST20_Corpus > /tmp/safe-names.txt
"""

import argparse
import glob
import os
import sys
from collections import Counter


def load_names(paths):
    names = set()
    for p in paths:
        with open(p, encoding='utf-8') as f:
            for line in f:
                line = line.strip()
                if line:
                    names.add(line)
    return names


def ordinary_bigrams(lst20_dir):
    """Counter of 'word1word2' (concatenated) for consecutive gold tokens in
    LST20 train+eval that do NOT both fall inside the same named-entity span
    (so the count reflects ordinary running text, not name-internal structure)."""
    files = sorted(
        f for f in glob.glob(f'{lst20_dir}/train/*.txt') + glob.glob(f'{lst20_dir}/eval/*.txt')
        if not os.path.basename(f).startswith('._')
    )
    bigrams = Counter()
    for f in files:
        prev_word, prev_ne = None, None
        with open(f, encoding='utf-8', errors='ignore') as fh:
            for raw in fh:
                line = raw.rstrip('\n')
                if not line:
                    prev_word, prev_ne = None, None
                    continue
                parts = line.split('\t')
                if len(parts) < 3:
                    prev_word, prev_ne = None, None
                    continue
                word, ne = parts[0], parts[2]
                if word == '_' or not word:
                    prev_word, prev_ne = None, None
                    continue
                if prev_word is not None:
                    same_entity = (
                        ne.startswith('I_') and prev_ne is not None
                        and prev_ne[2:] == ne[2:]
                        and (prev_ne.startswith('B_') or prev_ne.startswith('I_'))
                    )
                    if not same_entity:
                        bigrams[prev_word + word] += 1
                prev_word, prev_ne = word, ne
    return bigrams


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('names_files', nargs='+', help='input word list(s), one name per line')
    ap.add_argument('--lst20', required=True, help='path to the LST20_Corpus directory (train/eval/test subdirs)')
    ap.add_argument('--min-risk', type=int, default=2,
                     help='exclude a name if it occurs as an ordinary (non-PER) bigram at least this many times (default: 2)')
    ap.add_argument('--dict', default=None,
                     help='existing dictionary (e.g. data/words.txt) to skip names already present')
    ap.add_argument('--out', default=None, help='output file (default: stdout)')
    args = ap.parse_args()

    names = load_names(args.names_files)
    if args.dict:
        existing = load_names([args.dict])
        names -= existing
    print(f'{len(names)} candidate names (after removing existing dictionary entries)', file=sys.stderr)

    bigrams = ordinary_bigrams(args.lst20)
    print(f'{len(bigrams):,} ordinary (non-PER) bigrams found in LST20 train+eval', file=sys.stderr)

    risky = sorted((n for n in names if bigrams.get(n, 0) >= args.min_risk), key=lambda n: -bigrams[n])
    safe = sorted(n for n in names if bigrams.get(n, 0) < args.min_risk)
    print(f'Excluded (collide with ordinary phrases): {len(risky)}', file=sys.stderr)
    for n in risky[:25]:
        print(f'  {n:<12} used as an ordinary phrase {bigrams[n]}x', file=sys.stderr)
    print(f'Selected: {len(safe)}', file=sys.stderr)

    out = open(args.out, 'w', encoding='utf-8') if args.out else sys.stdout
    for n in safe:
        print(n, file=out)
    if args.out:
        out.close()


if __name__ == '__main__':
    main()
