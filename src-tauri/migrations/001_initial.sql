-- Terative initial schema.
-- Money stored as INTEGER cents, dates as ISO 8601 TEXT, IDs as UUID v4 TEXT.

-- "TERA" in hex — identifies this file as a Terative database.
-- See adapters::sqlite::connection::APPLICATION_ID.
PRAGMA application_id = 0x54455241;

CREATE TABLE clients (
    id                  TEXT PRIMARY KEY,
    -- 'Individual' for natural persons, 'Company' for legal entities.
    kind                TEXT NOT NULL DEFAULT 'Individual'
                        CHECK (kind IN ('Individual', 'Company')),
    name                TEXT NOT NULL,
    contact_name        TEXT,
    tax_id              TEXT,
    registration_number TEXT,
    notes               TEXT,
    referred_by         TEXT REFERENCES clients(id) ON DELETE SET NULL,
    date_of_birth       TEXT,    -- ISO 8601 calendar date (YYYY-MM-DD)
    sex                 TEXT,    -- biological sex (male/female/intersex/...)
    gender              TEXT,    -- gender identity (free-form)
    pronouns            TEXT,
    occupation          TEXT,
    language            TEXT,    -- ISO 639-1 code (fr, en, nl, ...)
    -- Pre-fills the currency on new invoices created for this client. Does
    -- NOT restrict: the user can still invoice in any currency.
    default_currency    TEXT NOT NULL,
    archived_at         TEXT,    -- RFC 3339 timestamp; NULL = active
    created_at          TEXT NOT NULL
);

CREATE TABLE client_emails (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    value       TEXT NOT NULL,
    label       TEXT,
    is_default  INTEGER NOT NULL DEFAULT 0,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE client_phones (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    value       TEXT NOT NULL,
    label       TEXT,
    is_default  INTEGER NOT NULL DEFAULT 0,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE client_addresses (
    id              TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    label           TEXT,
    street          TEXT NOT NULL,
    apt_suite       TEXT,
    city            TEXT NOT NULL,
    state_province  TEXT,
    postal_code     TEXT NOT NULL,
    country         TEXT NOT NULL, -- ISO 3166-1 alpha-2 codes
    -- "Active" per-role flags. A client may have many addresses on file,
    -- but only one is the currently active billing address (`is_billing
    -- = 1`) and only one is the currently active shipping address.
    -- The same row can carry both flags (common case for individuals).
    -- An address with both flags off is fine — it's an inactive address
    -- on file (e.g. an old site, or one not currently used for either
    -- purpose). The partial unique indexes below enforce the at-most-one
    -- cap per role at the DB layer.
    is_billing      INTEGER NOT NULL DEFAULT 0,
    is_shipping     INTEGER NOT NULL DEFAULT 0,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_client_emails_client     ON client_emails(client_id);
CREATE INDEX idx_client_phones_client     ON client_phones(client_id);
CREATE INDEX idx_client_addresses_client  ON client_addresses(client_id);
CREATE INDEX idx_clients_referred_by      ON clients(referred_by);

-- At most one default billing and one default shipping address per client.
-- A client may have many addresses for each role (multi-site companies),
-- but only one is marked as the default that auto-fills new invoices /
-- shipments. Partial indexes make this a pure DB-level invariant; the
-- repo layer must clear the previous default in the same transaction
-- when a new one is set.
CREATE UNIQUE INDEX uniq_client_billing
    ON client_addresses(client_id)
    WHERE is_billing = 1;
CREATE UNIQUE INDEX uniq_client_shipping
    ON client_addresses(client_id)
    WHERE is_shipping = 1;

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

CREATE INDEX idx_notebook_sections_sort     ON notebook_sections(sort_order);
CREATE INDEX idx_notebook_entries_client    ON client_notebook_entries(client_id);
CREATE INDEX idx_notebook_entries_section   ON client_notebook_entries(section_id);
CREATE INDEX idx_journal_client_date        ON client_journal_entries(client_id, entry_date DESC);

CREATE TABLE catalog_items (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    kind            TEXT NOT NULL CHECK (kind IN ('Product', 'Service')),
    unit            TEXT,
    reference       TEXT,
    archived_at     TEXT
);

-- Per-currency prices for a catalog item. A single item may have one row
-- per currency it can be quoted in. Strict-silos accounting: there is no
-- conversion between rows here; each price is the authoritative amount
-- in that currency. An item with no row for a given currency simply
-- can't be auto-priced when added to an invoice in that currency
-- (the user enters the unit price manually for that line).
CREATE TABLE catalog_item_prices (
    catalog_item_id TEXT NOT NULL REFERENCES catalog_items(id) ON DELETE CASCADE,
    currency        TEXT NOT NULL,
    amount          INTEGER NOT NULL,             -- minor units of `currency`
    PRIMARY KEY (catalog_item_id, currency)
);

CREATE INDEX idx_catalog_items_reference ON catalog_items(reference);
CREATE INDEX idx_catalog_item_prices_item ON catalog_item_prices(catalog_item_id);

CREATE TABLE tax_definitions (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    percentage      REAL NOT NULL,
    tax_id_number   TEXT,
    archived_at     TEXT
);

CREATE TABLE bookmarks (
    id          TEXT PRIMARY KEY,
    label       TEXT NOT NULL,
    url         TEXT NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_bookmarks_sort ON bookmarks(sort_order);

CREATE TABLE invoice_templates (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    base_layout         TEXT NOT NULL DEFAULT 'Classic',
    logo_image          BLOB,
    accent_color        TEXT,
    font_family         TEXT NOT NULL DEFAULT 'SansSerif',
    show_seller_phone   INTEGER NOT NULL DEFAULT 1,
    show_seller_email   INTEGER NOT NULL DEFAULT 1,
    show_registration_id INTEGER NOT NULL DEFAULT 1,
    show_tax_id_numbers INTEGER NOT NULL DEFAULT 1,
    show_signature      INTEGER NOT NULL DEFAULT 1,
    show_due_date       INTEGER NOT NULL DEFAULT 1,
    show_total_in_words INTEGER NOT NULL DEFAULT 1,
    header_text         TEXT,
    footer_text         TEXT,
    is_default          INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE invoices (
    id                  TEXT PRIMARY KEY,
    number              INTEGER UNIQUE,
    client_id           TEXT NOT NULL REFERENCES clients(id),
    template_id         TEXT REFERENCES invoice_templates(id),
    date                TEXT NOT NULL,
    due_date            TEXT,
    subtotal            INTEGER NOT NULL,
    tax_total           INTEGER NOT NULL,
    total               INTEGER NOT NULL,
    currency            TEXT NOT NULL DEFAULT 'EUR',
    status              TEXT NOT NULL DEFAULT 'Draft',
    pdf_path            TEXT,
    notes               TEXT,
    created_at          TEXT NOT NULL,
    updated_at          TEXT NOT NULL
);

CREATE TABLE invoice_line_items (
    id              TEXT PRIMARY KEY,
    invoice_id      TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    -- Optional FK back to the catalog item the line was seeded from. The `unit_price` is a denormalized snapshot
    catalog_item_id TEXT REFERENCES catalog_items(id) ON DELETE SET NULL,
    description     TEXT NOT NULL,
    quantity        REAL NOT NULL DEFAULT 1.0,
    unit_price      INTEGER NOT NULL,
    total           INTEGER NOT NULL,
    sort_order      INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_invoice_line_items_catalog_item
    ON invoice_line_items(catalog_item_id)
    WHERE catalog_item_id IS NOT NULL;

CREATE TABLE invoice_taxes (
    id                  TEXT PRIMARY KEY,
    invoice_id          TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    tax_definition_id   TEXT REFERENCES tax_definitions(id),
    tax_name            TEXT NOT NULL,
    percentage          REAL NOT NULL,
    tax_id_number       TEXT,
    computed_amount     INTEGER NOT NULL
);

CREATE TABLE payments (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id),
    date        TEXT NOT NULL,
    amount      INTEGER NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'EUR',
    method      TEXT NOT NULL DEFAULT 'BankTransfer',
    reference   TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL
);

CREATE TABLE payment_allocations (
    id          TEXT PRIMARY KEY,
    payment_id  TEXT NOT NULL REFERENCES payments(id) ON DELETE CASCADE,
    invoice_id  TEXT NOT NULL REFERENCES invoices(id),
    amount      INTEGER NOT NULL
);

CREATE TABLE seller_profile (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    name                TEXT NOT NULL DEFAULT '',
    title               TEXT,
    registration_id     TEXT,
    address             TEXT,
    phone               TEXT,
    email               TEXT,
    signature_image     BLOB
);
INSERT INTO seller_profile (id) VALUES (1);

CREATE TABLE email_templates (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    template_type   TEXT NOT NULL CHECK (template_type IN ('InitialContact', 'FollowUp')),
    subject_template TEXT NOT NULL DEFAULT '',
    body_template   TEXT NOT NULL DEFAULT '',
    is_default      INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE email_config (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    smtp_host           TEXT NOT NULL DEFAULT '',
    smtp_port           INTEGER NOT NULL DEFAULT 587,
    sender_address      TEXT NOT NULL DEFAULT ''
);
INSERT INTO email_config (id) VALUES (1);

CREATE TABLE currency_config (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    code                TEXT NOT NULL DEFAULT 'EUR'
);
INSERT INTO currency_config (id) VALUES (1);

CREATE TABLE app_preferences (
    id                          INTEGER PRIMARY KEY CHECK (id = 1),
    theme                       TEXT NOT NULL DEFAULT 'Light',
    language                    TEXT NOT NULL DEFAULT 'fr',
    pdf_output_dir              TEXT NOT NULL DEFAULT '',
    user_backup_dir             TEXT NOT NULL DEFAULT '',
    auto_backup_enabled         INTEGER NOT NULL DEFAULT 1,
    auto_backup_interval_hours  INTEGER NOT NULL DEFAULT 24,
    retention_mode              TEXT NOT NULL DEFAULT 'KeepLast' CHECK (retention_mode IN ('All', 'KeepLast')),
    retention_count             INTEGER NOT NULL DEFAULT 30,
    default_invoice_due_days    INTEGER NOT NULL DEFAULT 30
);
INSERT INTO app_preferences (id) VALUES (1);

CREATE TABLE invoice_number_seq (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    next_number INTEGER NOT NULL DEFAULT 1
);
INSERT INTO invoice_number_seq (id) VALUES (1);

CREATE INDEX idx_invoices_client      ON invoices(client_id);
CREATE INDEX idx_invoices_status      ON invoices(status);
CREATE INDEX idx_invoices_date        ON invoices(date);
CREATE INDEX idx_invoices_due_date    ON invoices(due_date);
CREATE INDEX idx_payments_client      ON payments(client_id);
CREATE INDEX idx_payments_date        ON payments(date);
CREATE INDEX idx_alloc_payment        ON payment_allocations(payment_id);
CREATE INDEX idx_alloc_invoice        ON payment_allocations(invoice_id);
CREATE INDEX idx_line_items_invoice   ON invoice_line_items(invoice_id);
CREATE INDEX idx_invoice_taxes_inv    ON invoice_taxes(invoice_id);

CREATE TABLE email_logs (
    id              TEXT PRIMARY KEY,
    client_id       TEXT NOT NULL REFERENCES clients(id) ON DELETE CASCADE,
    invoice_id      TEXT REFERENCES invoices(id) ON DELETE SET NULL,
    template_type   TEXT,
    template_name   TEXT,
    to_address      TEXT NOT NULL,
    subject         TEXT NOT NULL,
    sent_at         TEXT NOT NULL
);

CREATE INDEX idx_email_logs_client  ON email_logs(client_id);
CREATE INDEX idx_email_logs_invoice ON email_logs(invoice_id);
CREATE INDEX idx_email_logs_sent_at ON email_logs(sent_at);

-- T1.04 — Audit log. Append-only projection of domain events, written by
-- the AuditProjector handlers and read back by the dashboard feed, the
-- per-client tab and the per-invoice strip.
--
-- `event_type` is the dotted DomainEvent::event_name (e.g. "invoice.finalized").
-- `entity_type` / `entity_id` identify the event's *subject*.
-- `client_id` / `invoice_id` are denormalised *scope pointers*: they make the
-- per-client and per-invoice views single indexed lookups, and may differ from
-- the subject (a payment's row carries the payment id in `entity_id` but still
-- points at a `client_id`). FKs are ON DELETE SET NULL so a hard-deleted
-- client/invoice leaves the audit row behind as a renderable tombstone —
-- the human-readable bits live in `metadata_json`.
CREATE TABLE audits (
    id              TEXT PRIMARY KEY,
    event_type      TEXT NOT NULL,
    entity_type     TEXT NOT NULL,
    entity_id       TEXT,
    client_id       TEXT REFERENCES clients(id)  ON DELETE SET NULL,
    invoice_id      TEXT REFERENCES invoices(id) ON DELETE SET NULL,
    metadata_json   TEXT NOT NULL DEFAULT '{}',
    occurred_at     TEXT NOT NULL
);

-- Dashboard feed: newest-first across the whole org.
CREATE INDEX idx_audits_occurred_at ON audits(occurred_at DESC);
-- Per-client tab: filter by client, newest first.
CREATE INDEX idx_audits_client      ON audits(client_id, occurred_at DESC);
-- Per-invoice strip: filter by invoice, ordered chronologically.
CREATE INDEX idx_audits_invoice     ON audits(invoice_id, occurred_at);

CREATE VIEW v_invoice_payment_status AS
SELECT
    i.id,
    i.number,
    i.client_id,
    i.date,
    i.due_date,
    i.total,
    i.currency,
    i.status,
    COALESCE(SUM(pa.amount), 0) AS amount_paid,
    i.total - COALESCE(SUM(pa.amount), 0) AS amount_due,
    CASE
        WHEN i.status = 'Cancelled' THEN 'Cancelled'
        WHEN i.status = 'Draft' THEN 'Draft'
        WHEN COALESCE(SUM(pa.amount), 0) = 0 AND i.due_date < date('now')
            THEN 'Overdue'
        WHEN COALESCE(SUM(pa.amount), 0) = 0
            THEN 'Unpaid'
        WHEN COALESCE(SUM(pa.amount), 0) < i.total AND i.due_date < date('now')
            THEN 'Overdue'
        WHEN COALESCE(SUM(pa.amount), 0) < i.total
            THEN 'Partial'
        ELSE 'Paid'
    END AS payment_status
FROM invoices i
LEFT JOIN payment_allocations pa ON pa.invoice_id = i.id
GROUP BY i.id;

-- Per-currency client balance. Strict silos: there is no cross-currency
-- conversion, so each (client, currency) pair gets its own row. Clients
-- that have never been invoiced and have never paid produce no rows
-- here — the clients table is the source of truth for "clients I have",
-- this view is the source of truth for "balances I'm carrying".
CREATE VIEW v_client_balance AS
WITH inv AS (
    SELECT client_id, currency, SUM(total) AS total_invoiced
    FROM invoices
    WHERE status IN ('Finalized', 'Sent')
    GROUP BY client_id, currency
),
pay AS (
    SELECT client_id, currency, SUM(amount) AS total_paid
    FROM payments
    GROUP BY client_id, currency
),
keys AS (
    SELECT client_id, currency FROM inv
    UNION
    SELECT client_id, currency FROM pay
)
SELECT
    c.id,
    c.name,
    k.currency,
    COALESCE(inv.total_invoiced, 0) AS total_invoiced,
    COALESCE(pay.total_paid, 0) AS total_paid,
    COALESCE(inv.total_invoiced, 0) - COALESCE(pay.total_paid, 0) AS outstanding
FROM clients c
JOIN keys k ON k.client_id = c.id
LEFT JOIN inv ON inv.client_id = c.id AND inv.currency = k.currency
LEFT JOIN pay ON pay.client_id = c.id AND pay.currency = k.currency
WHERE c.archived_at IS NULL;

CREATE VIEW v_aging_report AS
SELECT
    i.id,
    i.number,
    i.client_id,
    c.name AS client_name,
    i.currency,
    i.total,
    i.total - COALESCE(alloc.allocated, 0) AS amount_due,
    i.due_date,
    CASE
        WHEN julianday('now') - julianday(i.due_date) <= 0  THEN 'Current'
        WHEN julianday('now') - julianday(i.due_date) <= 30 THEN '1-30 days'
        WHEN julianday('now') - julianday(i.due_date) <= 60 THEN '31-60 days'
        WHEN julianday('now') - julianday(i.due_date) <= 90 THEN '61-90 days'
        ELSE '90+ days'
    END AS aging_bucket
FROM invoices i
JOIN clients c ON c.id = i.client_id
LEFT JOIN (
    SELECT invoice_id, SUM(amount) AS allocated
    FROM payment_allocations
    GROUP BY invoice_id
) alloc ON alloc.invoice_id = i.id
WHERE i.status IN ('Finalized', 'Sent')
  AND i.total - COALESCE(alloc.allocated, 0) > 0;

-- === GLOBAL SEARCH (T1.07) ===
--
-- A single FTS5 full-text index spanning the three entity types a user
-- searches across: clients, catalog items and invoices. One standalone
-- (non external-content) FTS5 table keeps the design simple — ordinary
-- INSERT/UPDATE/DELETE work, so the sync triggers below are plain SQL.
--
-- `entity_type` / `entity_id` are UNINDEXED: they are stored and returned
-- but never tokenised. `title` is the primary display string; `body` holds
-- the secondary searchable text (contact names, tax IDs, references).
--
-- The `unicode61 remove_diacritics 2` tokenizer folds accents, so a search
-- for "cafe" matches "café" — important for the FR-first user base.
CREATE VIRTUAL TABLE search_index USING fts5(
    entity_type UNINDEXED,
    entity_id   UNINDEXED,
    title,
    body,
    tokenize = "unicode61 remove_diacritics 2"
);

-- One AFTER INSERT / UPDATE / DELETE trio per source table keeps the index
-- in sync. UPDATE is a delete-then-insert so a row that loses its
-- searchable content is dropped from the index rather than left stale.
-- Only finalized invoices are indexed, by their number. A draft has no
-- number and nothing else searchable, so the `WHEN` guard skips it.

-- ── clients ──
CREATE TRIGGER trg_search_clients_ai AFTER INSERT ON clients BEGIN
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('client', new.id, new.name,
            trim(coalesce(new.contact_name, '') || ' ' ||
                 coalesce(new.tax_id, '') || ' ' ||
                 coalesce(new.registration_number, '')));
END;

CREATE TRIGGER trg_search_clients_ad AFTER DELETE ON clients BEGIN
    DELETE FROM search_index WHERE entity_type = 'client' AND entity_id = old.id;
END;

CREATE TRIGGER trg_search_clients_au AFTER UPDATE ON clients BEGIN
    DELETE FROM search_index WHERE entity_type = 'client' AND entity_id = old.id;
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('client', new.id, new.name,
            trim(coalesce(new.contact_name, '') || ' ' ||
                 coalesce(new.tax_id, '') || ' ' ||
                 coalesce(new.registration_number, '')));
END;

-- ── catalog_items ──
CREATE TRIGGER trg_search_catalog_ai AFTER INSERT ON catalog_items BEGIN
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('catalog_item', new.id, new.name, coalesce(new.reference, ''));
END;

CREATE TRIGGER trg_search_catalog_ad AFTER DELETE ON catalog_items BEGIN
    DELETE FROM search_index WHERE entity_type = 'catalog_item' AND entity_id = old.id;
END;

CREATE TRIGGER trg_search_catalog_au AFTER UPDATE ON catalog_items BEGIN
    DELETE FROM search_index WHERE entity_type = 'catalog_item' AND entity_id = old.id;
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('catalog_item', new.id, new.name, coalesce(new.reference, ''));
END;

-- ── invoices ──
CREATE TRIGGER trg_search_invoices_ai AFTER INSERT ON invoices
WHEN new.number IS NOT NULL BEGIN
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('invoice', new.id, CAST(new.number AS TEXT), '');
END;

CREATE TRIGGER trg_search_invoices_ad AFTER DELETE ON invoices BEGIN
    DELETE FROM search_index WHERE entity_type = 'invoice' AND entity_id = old.id;
END;

CREATE TRIGGER trg_search_invoices_au AFTER UPDATE ON invoices
WHEN new.number IS NOT NULL BEGIN
    DELETE FROM search_index WHERE entity_type = 'invoice' AND entity_id = old.id;
    INSERT INTO search_index (entity_type, entity_id, title, body)
    VALUES ('invoice', new.id, CAST(new.number AS TEXT), '');
END;
