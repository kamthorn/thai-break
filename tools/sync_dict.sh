#!/usr/bin/env bash
# data/ is the single source of the dictionary. Some packages need their own
# copy inside the package directory so they can be built or published alone.
#
#   tools/sync_dict.sh rebuild build words.dawg, words.fst and words.php from data/words.txt, then stage
#   tools/sync_dict.sh stage   copy data/ into go/data (committed) and rust/data (gitignored)
#   tools/sync_dict.sh check   fail if a committed copy differs from data/
#
# After editing data/words.txt run `tools/sync_dict.sh rebuild`; it needs python3, php and cargo.
set -euo pipefail
cd "$(dirname "$0")/.."

# source -> destination (committed copies)
COMMITTED=(
  "data/words.dawg go/data/words.dawg"
  "data/words.txt go/data/words.txt"
  "data/word-weights.tsv go/data/word-weights.tsv"
  "NOTICE go/NOTICE"
  "NOTICE rust/NOTICE"
  "NOTICE typescript/NOTICE"
)
# staged for packaging only, never committed
STAGED=(
  "data/words.fst rust/data/words.fst"
  "data/word-weights.tsv rust/data/word-weights.tsv"
)

case "${1:-}" in
  rebuild)
    python3 tools/build_dawg.py data/words.txt data/words.dawg
    (cd rust && cargo run -q --bin thaibreak_compile_fst -- ../data/words.txt ../data/words.fst)
    php tools/build_php_dict.php
    exec "$0" stage
    ;;
  stage)
    for pair in "${COMMITTED[@]}" "${STAGED[@]}"; do
      set -- $pair
      mkdir -p "$(dirname "$2")"
      cp "$1" "$2"
      echo "staged $2"
    done
    ;;
  check)
    status=0
    for pair in "${COMMITTED[@]}"; do
      set -- $pair
      if ! cmp -s "$1" "$2"; then
        echo "out of sync: $2 differs from $1 (run tools/sync_dict.sh stage)" >&2
        status=1
      fi
    done
    for pair in "${STAGED[@]}"; do
      set -- $pair
      if [ -e "$2" ] && ! cmp -s "$1" "$2"; then
        echo "stale staged copy: $2 differs from $1 (run tools/sync_dict.sh stage)" >&2
        status=1
      fi
    done
    [ "$status" -eq 0 ] && echo "dictionary copies are in sync"
    exit "$status"
    ;;
  *)
    echo "usage: $0 rebuild|stage|check" >&2
    exit 2
    ;;
esac
