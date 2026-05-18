#!/usr/bin/env bash

set -e

rm -f tests/persistence.db

cargo run -- \
  --db tests/persistence.db \
  --cmd "collections create users"

cargo run -- \
  --db tests/persistence.db \
  --cmd 'documents insert users {"name":"Persistent"}'

echo "Reopening database..."

cargo run -- \
  --db tests/persistence.db \
  --cmd "documents find users"

echo "Persistence test passed."
