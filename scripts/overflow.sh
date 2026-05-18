#!/usr/bin/env bash

set -e

rm -f tests/overflow.db

cargo run -- \
  --db tests/overflow.db \
  --cmd "collections create users"

for i in {1..500}
do
  cargo run -- \
    --db tests/overflow.db \
    --cmd "documents insert users {\"id\":$i,\"name\":\"user$i\"}"
done

cargo run -- \
  --db tests/overflow.db \
  --cmd "documents find users"

echo "Overflow test passed."
