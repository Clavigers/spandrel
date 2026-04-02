# Spandrel CLI

A Rust CLI for creating and validating source-code links between two locations in a codebase, backed by PostgreSQL.

## Commands

### `spandrel link <JSON>`

Creates a validated link between two source spans.

**JSON schema:**

```json
{
  "link_type": "CONNECTS" | "CONTRADICTS",
  "here": SourceSpan,
  "there": [SourceSpan, ...]
}
```

`there` is an array of one or more target spans. A single target is valid.

**SourceSpan schema:**

```json
{
  "file_path": string,
  "start_line_number": integer,
  "start_column": integer,
  "end_line_number": integer,
  "end_column": integer
}
```

**Example:**

```bash
spandrel link '{"link_type": "CONNECTS", "here": {"file_path": "src/main.rs", "start_line_number": 1, "start_column": 1, "end_line_number": 5, "end_column": 10}, "there": [{"file_path": "src/lib.rs", "start_line_number": 10, "start_column": 1, "end_line_number": 15, "end_column": 20}]}'
```

### `spandrel print <UUID>`

Prints the source content of a single link's spans as markdown.

### `spandrel print --all`

Prints every link in the database as one markdown document, separated by horizontal rules.

### `spandrel help`

Prints usage information (provided by clap's built-in `--help` / `help` subcommand).

## Validation

Before writing a link to the database, the CLI validates:

1. **Schema compliance** — The input JSON must deserialize into the expected structure. On failure, serde's error is wrapped in a human-readable message that names the offending field and explains the expected type/value.
2. **File existence** — Both `here.file_path` and `there.file_path` must point to files that exist on disk.
3. **Span bounds** — For each SourceSpan the `end_line_number` must not exceed the total number of lines in the referenced file, and `start_line_number <= end_line_number`. Line and column numbers are 1-based.

If any validation step fails, the CLI prints a descriptive error and exits with a non-zero status code. Nothing is written to the database.

## Scoping

Every link is scoped to a **snapshot** — a (repo, commit) pair. The CLI auto-detects both:

- **repo** — resolved via `gh repo view` (GitHub CLI), giving the `owner/repo` identifier (e.g. `wmurra/spandrel`)
- **commit** — the current `HEAD` commit hash via `git rev-parse HEAD`

The user does not provide these; they are derived from the working directory.

## Storage

Validated links are persisted to a PostgreSQL database. The connection is configured via the `DATABASE_URL` environment variable.

**Schema (snapshots table):**

| Column      | Type      | Notes                                  |
|-------------|-----------|----------------------------------------|
| id          | UUID      | Primary key, generated                 |
| repo_path   | TEXT      | GitHub `owner/repo` identifier         |
| commit_hash | TEXT      | Full SHA of the commit                 |
| created_at  | TIMESTAMP | Defaults to now()                      |

Unique constraint on `(repo_path, commit_hash)`.

**Schema (links table):**

| Column       | Type      | Notes                        |
|--------------|-----------|------------------------------|
| id           | UUID      | Primary key, generated       |
| snapshot_id  | UUID      | FK to snapshots              |
| link_type    | TEXT      | `CONNECTS` or `CONTRADICTS`  |
| file_path    | TEXT      | Source span file              |
| start_line   | INTEGER   |                              |
| start_column | INTEGER   |                              |
| end_line     | INTEGER   |                              |
| end_column   | INTEGER   |                              |
| created_at   | TIMESTAMP | Defaults to now()            |

**Schema (link_targets table):**

| Column       | Type      | Notes                        |
|--------------|-----------|------------------------------|
| id           | UUID      | Primary key, generated       |
| link_id      | UUID      | FK to links (CASCADE delete) |
| file_path    | TEXT      | Target span file             |
| start_line   | INTEGER   |                              |
| start_column | INTEGER   |                              |
| end_line     | INTEGER   |                              |
| end_column   | INTEGER   |                              |
| created_at   | TIMESTAMP | Defaults to now()            |

## Prerequisites

| Tool       | Purpose                                    |
|------------|---------------------------------------------|
| git        | Detect current commit hash                  |
| [gh](https://cli.github.com/) | Resolve GitHub repo identity (`gh repo view`). Must be installed and authenticated (`gh auth login`). |
| PostgreSQL | Link storage                                |

## Rust Dependencies

| Crate      | Purpose                        |
|------------|--------------------------------|
| clap       | CLI argument parsing           |
| serde      | JSON deserialization           |
| serde_json | JSON parsing                   |
| sqlx       | Async PostgreSQL driver        |
| tokio      | Async runtime (required by sqlx) |
| uuid       | UUID generation for primary key |

## Configuration

| Env var        | Required | Description                         |
|----------------|----------|-------------------------------------|
| DATABASE_URL   | Yes      | PostgreSQL connection string        |
