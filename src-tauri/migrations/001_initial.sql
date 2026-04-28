-- Terative initial schema.
-- Money stored as INTEGER cents, dates as ISO 8601 TEXT, IDs as UUID v4 TEXT.

-- "TERA" in hex — identifies this file as a Terative database.
-- See adapters::sqlite::connection::APPLICATION_ID.
PRAGMA application_id = 0x54455241;

CREATE TABLE clients (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    address         TEXT,
    notes           TEXT,
    referred_by     TEXT REFERENCES clients(id) ON DELETE SET NULL,
    date_of_birth   TEXT,    -- ISO 8601 calendar date (YYYY-MM-DD)
    sex             TEXT,    -- biological sex (male/female/intersex/...)
    gender          TEXT,    -- gender identity (free-form)
    pronouns        TEXT,
    occupation      TEXT,
    language        TEXT,    -- ISO 639-1 code (fr, en, nl, ...)
    archived_at     TEXT,    -- RFC 3339 timestamp; NULL = active
    created_at      TEXT NOT NULL
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

CREATE INDEX idx_client_emails_client ON client_emails(client_id);
CREATE INDEX idx_client_phones_client ON client_phones(client_id);
CREATE INDEX idx_clients_referred_by  ON clients(referred_by);

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
    default_price   INTEGER NOT NULL,
    currency        TEXT NOT NULL DEFAULT 'EUR',
    unit            TEXT,
    reference       TEXT,
    archived_at     TEXT
);

CREATE INDEX idx_catalog_items_reference ON catalog_items(reference);

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
    id          TEXT PRIMARY KEY,
    invoice_id  TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity    REAL NOT NULL DEFAULT 1.0,
    unit_price  INTEGER NOT NULL,
    total       INTEGER NOT NULL,
    sort_order  INTEGER NOT NULL DEFAULT 0
);

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

CREATE TABLE invoice_email_sends (
    id              TEXT PRIMARY KEY,
    invoice_id      TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    template_type   TEXT NOT NULL,
    template_name   TEXT NOT NULL,
    to_address      TEXT NOT NULL,
    subject         TEXT NOT NULL,
    sent_at         TEXT NOT NULL
);

CREATE INDEX idx_email_sends_invoice ON invoice_email_sends(invoice_id);

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

CREATE VIEW v_client_balance AS
SELECT
    c.id,
    c.name,
    COALESCE(inv.total_invoiced, 0) AS total_invoiced,
    COALESCE(pay.total_paid, 0) AS total_paid,
    COALESCE(inv.total_invoiced, 0) - COALESCE(pay.total_paid, 0) AS outstanding
FROM clients c
LEFT JOIN (
    SELECT client_id, SUM(total) AS total_invoiced
    FROM invoices
    WHERE status IN ('Finalized', 'Sent')
    GROUP BY client_id
) inv ON inv.client_id = c.id
LEFT JOIN (
    SELECT client_id, SUM(amount) AS total_paid
    FROM payments
    GROUP BY client_id
) pay ON pay.client_id = c.id
WHERE c.archived_at IS NULL;

CREATE VIEW v_aging_report AS
SELECT
    i.id,
    i.number,
    i.client_id,
    c.name AS client_name,
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
