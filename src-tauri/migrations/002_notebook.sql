-- Client notebook: global section templates, per-client notebook content,
-- and a freeform journal of client meetings.

CREATE TABLE notebook_sections (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE client_notebook_entries (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    section_id  TEXT NOT NULL REFERENCES notebook_sections(id) ON DELETE CASCADE,
    content     TEXT NOT NULL DEFAULT '',
    updated_at  TEXT NOT NULL,
    UNIQUE(client_id, section_id)
);

CREATE TABLE client_journal_entries (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    entry_date  TEXT NOT NULL,
    content     TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE INDEX idx_notebook_entries_client    ON client_notebook_entries(client_id);
CREATE INDEX idx_notebook_entries_section   ON client_notebook_entries(section_id);
CREATE INDEX idx_journal_client_date        ON client_journal_entries(client_id, entry_date DESC);
CREATE INDEX idx_notebook_sections_sort     ON notebook_sections(sort_order);
