#!/usr/bin/env bash
set -euo pipefail

DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/spandrel}"

psql "$DATABASE_URL" -c "TRUNCATE link_targets, links, snapshots CASCADE;"
echo "Database reset."
