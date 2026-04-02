#!/usr/bin/env bash
set -euo pipefail

# ── gh CLI ────────────────────────────────────────────────────────────
if command -v gh &>/dev/null; then
    echo "gh CLI already installed: $(gh --version | head -1)"
else
    echo "Installing gh CLI..."
    (type -p wget >/dev/null || (sudo apt-get update && sudo apt-get install -y wget)) \
        && sudo mkdir -p -m 755 /etc/apt/keyrings \
        && out=$(mktemp) \
        && wget -nv -O "$out" https://cli.github.com/packages/githubcli-archive-keyring.gpg \
        && cat "$out" | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg >/dev/null \
        && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
        && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" \
            | sudo tee /etc/apt/sources.list.d/github-cli.list >/dev/null \
        && sudo apt-get update \
        && sudo apt-get install -y gh
    echo "gh CLI installed: $(gh --version | head -1)"
fi

# ── Rust toolchain ────────────────────────────────────────────────────
if command -v cargo &>/dev/null; then
    echo "Rust toolchain found: $(cargo --version)"
else
    echo "Installing Rust via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
    echo "Rust installed: $(cargo --version)"
fi

# ── Build spandrel ────────────────────────────────────────────────────
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "Building spandrel..."
cargo build --release --manifest-path "$SCRIPT_DIR/cli/Cargo.toml"

# ── Add to PATH ───────────────────────────────────────────────────────
BINARY="$SCRIPT_DIR/cli/target/release/spandrel"
INSTALL_DIR="$HOME/.local/bin"
mkdir -p "$INSTALL_DIR"
cp "$BINARY" "$INSTALL_DIR/spandrel"
echo "Installed spandrel to $INSTALL_DIR/spandrel"

SHELL_RC="$HOME/.bashrc"
[[ -n "${ZSH_VERSION:-}" ]] && SHELL_RC="$HOME/.zshrc"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$SHELL_RC"
    echo "Added $INSTALL_DIR to PATH in $SHELL_RC"
fi

if ! grep -q 'DATABASE_URL' "$SHELL_RC" 2>/dev/null; then
    echo 'export DATABASE_URL="postgres://postgres:postgres@localhost:5432/spandrel"' >> "$SHELL_RC"
    echo "Added DATABASE_URL to $SHELL_RC"
fi

# ── Create database & apply schema ───────────────────────────────────
DB_URL="${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/spandrel}"
DB_NAME="spandrel"

if ! psql "$DB_URL" -c '\q' 2>/dev/null; then
    echo "Creating database $DB_NAME..."
    createdb -U postgres "$DB_NAME"
    echo "Database $DB_NAME created."
fi

echo "Applying schema..."
psql "$DB_URL" -f "$SCRIPT_DIR/cli/schema.sql"
echo "Schema applied."

echo "Restart your shell or run: source $SHELL_RC"

echo "Done. Run 'spandrel help' to get started."
