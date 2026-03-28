CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE links (
    id              UUID PRIMARY KEY,
    link_type       TEXT NOT NULL CHECK (link_type IN ('CONNECTS', 'CONTRADICTS')),
    here_file_path     TEXT    NOT NULL,
    here_start_line    INTEGER NOT NULL,
    here_start_column  INTEGER NOT NULL,
    here_end_line      INTEGER NOT NULL,
    here_end_column    INTEGER NOT NULL,
    there_file_path    TEXT    NOT NULL,
    there_start_line   INTEGER NOT NULL,
    there_start_column INTEGER NOT NULL,
    there_end_line     INTEGER NOT NULL,
    there_end_column   INTEGER NOT NULL,
    created_at      TIMESTAMP NOT NULL DEFAULT now()
);
