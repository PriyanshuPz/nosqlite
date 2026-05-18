#!/usr/bin/env bash

set -e

rm -f tests/bench.db

cargo run -- \
  --db tests/bench.db \
  --cmd "collections create users"

echo "Starting benchmark..."

time (
  for i in {1..2000}
  do
    cargo run --quiet -- \
      --db tests/bench.db \
      --cmd "documents insert users {\"id\":$i}"
  done
)

echo "Benchmark complete."
