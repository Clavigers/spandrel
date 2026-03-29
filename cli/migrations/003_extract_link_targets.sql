CREATE TABLE link_targets (
    id                 UUID PRIMARY KEY,
    link_id            UUID NOT NULL REFERENCES links(id) ON DELETE CASCADE,
    file_path          TEXT    NOT NULL,
    start_line         INTEGER NOT NULL,
    start_column       INTEGER NOT NULL,
    end_line           INTEGER NOT NULL,
    end_column         INTEGER NOT NULL,
    created_at         TIMESTAMP NOT NULL DEFAULT now()
);

INSERT INTO link_targets (id, link_id, file_path, start_line, start_column, end_line, end_column, created_at)
SELECT uuid_generate_v4(), id, there_file_path, there_start_line, there_start_column, there_end_line, there_end_column, created_at
FROM links;

ALTER TABLE links
    DROP COLUMN there_file_path,
    DROP COLUMN there_start_line,
    DROP COLUMN there_start_column,
    DROP COLUMN there_end_line,
    DROP COLUMN there_end_column;

ALTER TABLE links
    RENAME COLUMN here_file_path    TO file_path;
ALTER TABLE links
    RENAME COLUMN here_start_line   TO start_line;
ALTER TABLE links
    RENAME COLUMN here_start_column TO start_column;
ALTER TABLE links
    RENAME COLUMN here_end_line     TO end_line;
ALTER TABLE links
    RENAME COLUMN here_end_column   TO end_column;
