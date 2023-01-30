#!/bin/sh
set -eu

fail() {
    echo "$1"
    return 1
}

git diff-index --quiet HEAD \
    || fail 'Git Tree Dirty commit changes before benchmarking'

commit="$(git rev-parse --short HEAD)"
file="bench-$commit"

[ -e "$file" ] && fail "$file exists refusing to benchmark again"

cargo build --release
hyperfine "cargo run --release -- test --sudoku '......2.1..4.3.............37.....8.6..2........5.....54....6......7..4...2..1...'" | tee "$file"
echo "Benchmarked $commit"
