CREATE TABLE snapshots (
    id          UUID PRIMARY KEY,
    repo_path   TEXT NOT NULL,
    commit_hash TEXT NOT NULL,
    created_at  TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE (repo_path, commit_hash)
);

ALTER TABLE links ADD COLUMN snapshot_id UUID NOT NULL REFERENCES snapshots(id);
