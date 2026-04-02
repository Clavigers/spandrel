CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE snapshots (
    id          UUID PRIMARY KEY,
    repo_path   TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (repo_path, commit_hash)
);

CREATE TABLE links (
    id              UUID PRIMARY KEY,
    link_type       TEXT NOT NULL CHECK (link_type IN ('CONNECTS', 'CONTRADICTS', 'DEFINES')),
    snapshot_id     UUID NOT NULL REFERENCES snapshots(id),
    file_path       TEXT    NOT NULL,
    start_line      INTEGER NOT NULL,
    start_column    INTEGER NOT NULL,
    end_line        INTEGER NOT NULL,
    end_column      INTEGER NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE link_targets (
    id               UUID PRIMARY KEY,
    link_id          UUID NOT NULL REFERENCES links(id) ON DELETE CASCADE,
    file_path        TEXT    NOT NULL,
    start_line       INTEGER NOT NULL,
    start_column     INTEGER NOT NULL,
    end_line         INTEGER NOT NULL,
    end_column       INTEGER NOT NULL,
    created_at       TIMESTAMP NOT NULL DEFAULT now()
);
