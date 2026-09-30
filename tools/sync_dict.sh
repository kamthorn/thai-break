#!/usr/bin/env bash
# data/ is the single source of the dictionary. Some packages need their own
# copy inside the package directory so they can be built or published alone.
#
#   tools/sync_dict.sh stage   copy data/ into go/data (committed) and rust/data (gitignored)
#   tools/sync_dict.sh check   fail if a committed copy differs from data/
set -euo pipefail
cd "$(dirname "$0")/.."

# source -> destination (committed copies)
COMMITTED=(
  "data/words.dawg go/data/words.dawg"
  "data/words.txt go/data/words.txt"
)
# staged for packaging only, never committed
STAGED=(
  "data/words.fst rust/data/words.fst"
)

case "${1:-}" in
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
    echo "usage: $0 stage|check" >&2
    exit 2
    ;;
esac
