#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DATABASE_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/spandrel}"
DB_NAME="spandrel"

dropdb -U postgres --if-exists "$DB_NAME"
createdb -U postgres "$DB_NAME"
psql "$DATABASE_URL" -f "$SCRIPT_DIR/cli/schema.sql"
echo "Database recreated."
