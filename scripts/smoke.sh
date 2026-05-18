#!/usr/bin/env bash

set -e

rm -f tests/smoke.db

cargo run -- \
  --db tests/smoke.db \
  --cmd "collections create users"

cargo run -- \
  --db tests/smoke.db \
  --cmd 'documents insert users {"name":"Priyanshu","age":18}'

cargo run -- \
  --db tests/smoke.db \
  --cmd 'documents insert users {"name":"Alice","age":22}'

cargo run -- \
  --db tests/smoke.db \
  --cmd "documents find users"

echo "Smoke test passed."
