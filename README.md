# Terative v2 — Architecture Document

## Overview

Local-first invoice generator + accounting tool. Built with Tauri v2 (Rust backend, React frontend). Hexagonal architecture with SQLite storage. Single-device install, no cloud dependency.

---

## Tech Stack

| Layer | Choice | Rationale |
|---|---|---|
| Shell | Tauri v2 | Rust backend, lightweight, cross-platform |
| Frontend | React + TypeScript + Vite | TS expertise, rich Tauri ecosystem |
| Styling | Tailwind CSS | Fast iteration, built-in dark mode |
| State | Zustand | Minimal, no boilerplate, clean port interface |
| PDF | `typst` (Rust crate) | Programmable typesetting, templates as code |
| Email | `lettre` (Rust crate) | SMTP from backend, async, TLS |
| DB | SQLite via `rusqlite` | Single-file, transactional, queryable |
| Secrets | Tauri plugin `stronghold` | Encrypted credential vault |
| i18n | `react-i18next` | FR/EN, easy to extend |

---

## Hexagonal Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                     FRONTEND (React + TS)                    │
│                                                              │
│  Pages:                                                      │
│  ├─ Dashboard (revenue chart, outstanding, overdue, recent)  │
│  ├─ Invoice Create/Edit (multi-line, preview, draft flow)    │
│  ├─ Invoice List (status badges, filters, bulk actions)      │
│  ├─ Payments (record, allocate, history)                     │
│  ├─ Accounting (revenue analytics, aging, year-end report)   │
│  ├─ Clients (CRUD, balance, invoice history per client)      │
│  ├─ Template Editor (layout picker, customization, preview)  │
│  └─ Settings (profile, services, taxes, email, currency,     │
│               appearance, data import/export)                │
│                                                              │
│  Driven Adapter: Tauri IPC invoke() calls                    │
└──────────────────────────┬───────────────────────────────────┘
                           │ Tauri Commands (IPC boundary)
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                    TAURI COMMAND LAYER                        │
│                                                              │
│  Thin glue: deserialize args → call use case → serialize     │
│  One #[tauri::command] per use case                          │
│  Owns no logic. Maps IPC ↔ Application layer.                │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                   APPLICATION LAYER (Rust)                    │
│                                                              │
│  Use Cases (one struct per use case, injected ports):        │
│                                                              │
│  Invoice                                                     │
│  ├─ CreateDraftInvoice                                       │
│  ├─ UpdateDraftInvoice                                       │
│  ├─ FinalizeInvoice  (assigns number, generates PDF)         │
│  ├─ SendInvoice      (emails PDF, status → Sent)             │
│  ├─ CancelInvoice    (watermarks PDF, status → Cancelled)    │
│  └─ DuplicateInvoice (clone as new draft)                    │
│                                                              │
│  Payment                                                     │
│  ├─ RecordPayment    (with allocations to invoices)          │
│  ├─ UpdatePayment                                            │
│  ├─ DeletePayment                                            │
│  └─ AllocatePayment  (link unallocated → invoices)           │
│                                                              │
│  Accounting (read-only queries)                              │
│  ├─ GetOutstandingInvoices                                   │
│  ├─ GetOverdueInvoices                                       │
│  ├─ GetRevenueByPeriod(start, end, grouping)                 │
│  ├─ GetRevenueByClient(start, end)                           │
│  ├─ GetClientBalance(client_id)                              │
│  ├─ GetAgingReport                                           │
│  ├─ GetDashboardSummary                                      │
│  └─ GenerateYearEndReport(year) → PDF                        │
│                                                              │
│  Client                                                      │
│  ├─ CreateClient                                             │
│  ├─ UpdateClient                                             │
│  ├─ DeleteClient  (soft delete if has invoices)              │
│  ├─ ListClients   (with search/filter)                       │
│  └─ GetClientDetail (balance + invoice history)              │
│                                                              │
│  Service                                                     │
│  ├─ CreateService                                            │
│  ├─ UpdateService                                            │
│  ├─ DeleteService                                            │
│  └─ ListServices                                             │
│                                                              │
│  Template                                                    │
│  ├─ CreateTemplate                                           │
│  ├─ UpdateTemplate                                           │
│  ├─ DeleteTemplate   (block if used by invoices)             │
│  ├─ DuplicateTemplate                                        │
│  ├─ SetDefaultTemplate                                       │
│  ├─ ListTemplates                                            │
│  └─ PreviewTemplate  (render sample PDF for live preview)    │
│                                                              │
│  Settings                                                    │
│  ├─ GetSettings                                              │
│  ├─ UpdateSellerProfile                                      │
│  ├─ UpdateEmailConfig                                        │
│  ├─ UpdateCurrency                                           │
│  ├─ UpdateTaxDefinitions                                     │
│  └─ UpdateAppPreferences (theme, language)                   │
│                                                              │
│  Data Management                                             │
│  ├─ ImportLegacyJson  (old electron-store → SQLite)          │
│  ├─ ExportDatabase    (copy .sqlite to chosen path)          │
│  ├─ CreateBackup                                             │
│  └─ RestoreBackup                                            │
│                                                              │
│  Ports (traits — defined here, implemented in adapters):     │
│  ├─ InvoiceRepository                                        │
│  ├─ PaymentRepository                                        │
│  ├─ ClientRepository                                         │
│  ├─ ServiceRepository                                        │
│  ├─ SettingsRepository                                       │
│  ├─ TaxRepository                                            │
│  ├─ TemplateRepository                                       │
│  ├─ PdfGenerator   (Invoice + Template + SellerProfile + Currency → PDF) │
│  ├─ EmailSender                                              │
│  ├─ CredentialStore                                          │
│  └─ InvoiceNumberGenerator                                   │
└──────────────────────────┬───────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                      DOMAIN LAYER (Rust)                     │
│                    Pure — no dependencies                     │
│                                                              │
│  Aggregates                                                  │
│  ═══════════                                                 │
│                                                              │
│  Invoice                                                     │
│  ├─ id: InvoiceId (UUID)                                     │
│  ├─ number: Option<InvoiceNumber>    (assigned on finalize)  │
│  ├─ client_id: ClientId                                      │
│  ├─ template_id: Option<TemplateId>  (None → use default)    │
│  ├─ date: NaiveDate                                          │
│  ├─ due_date: Option<NaiveDate>                              │
│  ├─ line_items: Vec<LineItem>                                │
│  ├─ taxes_applied: Vec<AppliedTax>                           │
│  ├─ subtotal: Money                  (sum of line items)     │
│  ├─ tax_total: Money                 (sum of tax lines)      │
│  ├─ total: Money                     (subtotal + tax_total)  │
│  ├─ status: InvoiceStatus                                    │
│  ├─ pdf_path: Option<PathBuf>                                │
│  ├─ notes: Option<String>                                    │
│  ├─ created_at: DateTime<Utc>                                │
│  └─ updated_at: DateTime<Utc>                                │
│                                                              │
│  Payment                                                     │
│  ├─ id: PaymentId (UUID)                                     │
│  ├─ client_id: ClientId                                      │
│  ├─ date: NaiveDate                                          │
│  ├─ amount: Money                                            │
│  ├─ method: PaymentMethod                                    │
│  ├─ reference: Option<String>        (wire ref, check #)     │
│  ├─ allocations: Vec<PaymentAllocation>                      │
│  ├─ notes: Option<String>                                    │
│  └─ created_at: DateTime<Utc>                                │
│                                                              │
│  Client                                                      │
│  ├─ id: ClientId (UUID)                                      │
│  ├─ name: String                                             │
│  ├─ email: Option<String>                                    │
│  ├─ address: Option<String>                                  │
│  ├─ phone: Option<String>                                    │
│  ├─ notes: Option<String>                                    │
│  ├─ active: bool                     (soft delete flag)      │
│  └─ created_at: DateTime<Utc>                                │
│                                                              │
│  Service                                                     │
│  ├─ id: ServiceId (UUID)                                     │
│  ├─ name: String                                             │
│  ├─ default_price: Money                                     │
│  └─ active: bool                                             │
│                                                              │
│  TaxDefinition                                               │
│  ├─ id: TaxId (UUID)                                         │
│  ├─ name: String                     (e.g. "TVA")            │
│  ├─ percentage: Decimal                                      │
│  ├─ tax_id_number: Option<String>    (your tax reg #)        │
│  └─ active: bool                                             │
│                                                              │
│  InvoiceTemplate                                             │
│  ├─ id: TemplateId (UUID)                                    │
│  ├─ name: String                     ("Classic", "Minimal")  │
│  ├─ base_layout: TemplateLayout      (enum: shipped layouts) │
│  ├─ logo_image: Option<Vec<u8>>                              │
│  ├─ accent_color: Option<String>     (hex, e.g. "#2563EB")   │
│  ├─ font_family: FontChoice          (Serif|SansSerif|Mono)  │
│  ├─ show_seller_phone: bool                                  │
│  ├─ show_seller_email: bool                                  │
│  ├─ show_registration_id: bool                               │
│  ├─ show_tax_id_numbers: bool                                │
│  ├─ show_signature: bool                                     │
│  ├─ show_due_date: bool                                      │
│  ├─ show_total_in_words: bool        ("vingt-trois euros…")  │
│  ├─ header_text: Option<String>      (above the invoice)     │
│  ├─ footer_text: Option<String>      (legal, payment terms)  │
│  └─ is_default: bool                 (only one at a time)    │
│                                                              │
│  TemplateLayout: Classic | Modern | Minimal                  │
│  ── Each variant maps to a shipped .typ file                 │
│  ── User picks layout, customizes parameters around it       │
│                                                              │
│  FontChoice: Serif | SansSerif | Mono                        │
│                                                              │
│  SellerProfile                                               │
│  ├─ name: String                                             │
│  ├─ title: Option<String>                                    │
│  ├─ registration_id: Option<String>                          │
│  ├─ address: Option<String>                                  │
│  ├─ phone: Option<String>                                    │
│  ├─ email: Option<String>                                    │
│  └─ signature_image: Option<Vec<u8>>  (binary, not base64)   │
│                                                              │
│  EmailConfig                                                 │
│  ├─ smtp_host: String                                        │
│  ├─ smtp_port: u16                                           │
│  ├─ sender_address: String                                   │
│  ├─ subject_template: String         (supports {{placeholders}}) │
│  └─ body_template: String                                    │
│                                                              │
│  CurrencyConfig                                              │
│  ├─ code: String                     (e.g. "EUR")            │
│  ├─ symbol: String                   (e.g. "€")             │
│  ├─ symbol_before: bool              (€100 vs 100€)         │
│  ├─ main_unit_name: String           (e.g. "euros")          │
│  └─ sub_unit_name: String            (e.g. "centimes")       │
│                                                              │
│  AppPreferences                                              │
│  ├─ theme: Theme                     (Light | Dark)          │
│  ├─ language: Language               (FR | EN)               │
│  └─ pdf_output_dir: PathBuf                                  │
│                                                              │
│                                                              │
│  Value Objects                                               │
│  ═════════════                                               │
│                                                              │
│  Money { amount_cents: i64, currency: String }               │
│  ── All arithmetic in cents, no floats ever                  │
│                                                              │
│  InvoiceNumber(u64)                                          │
│  ── Auto-increment, assigned on finalize only                │
│                                                              │
│  InvoiceStatus: Draft | Finalized | Sent | Cancelled         │
│  ── Draft → Finalized → Sent (each transition is one-way)    │
│  ── Finalized → Cancelled (cannot cancel a Draft)            │
│  ── Sent → Cancelled                                         │
│                                                              │
│  LineItem {                                                  │
│      description: String,                                    │
│      quantity: Decimal,                                      │
│      unit_price: Money,                                      │
│      total: Money,          (quantity × unit_price)          │
│  }                                                           │
│                                                              │
│  AppliedTax {                                                │
│      tax_definition_id: TaxId,                               │
│      tax_name: String,       (snapshot, not FK)              │
│      percentage: Decimal,    (snapshot)                       │
│      tax_id_number: Option<String>,  (snapshot)              │
│      computed_amount: Money,                                 │
│  }                                                           │
│  ── Snapshots tax values at invoice creation time.           │
│  ── If tax rate changes later, old invoices keep their rate. │
│                                                              │
│  PaymentAllocation {                                         │
│      invoice_id: InvoiceId,                                  │
│      amount: Money,                                          │
│  }                                                           │
│  ── Sum of allocations ≤ payment.amount (enforced in domain) │
│  ── Sum of allocations for an invoice ≤ invoice.total        │
│                                                              │
│  PaymentMethod: BankTransfer | Cash | Check | Card | Other(String) │
│                                                              │
│                                                              │
│  Derived / Computed (not stored)                             │
│  ══════════════════════════════                               │
│                                                              │
│  InvoicePaymentStatus (for any invoice):                     │
│  ├─ Unpaid   → allocated = 0                                 │
│  ├─ Partial  → 0 < allocated < total                         │
│  ├─ Paid     → allocated >= total                            │
│  └─ Overdue  → (Unpaid | Partial) AND due_date < today       │
│                                                              │
│  amount_paid(invoice)  = sum(allocations targeting invoice)   │
│  amount_due(invoice)   = invoice.total - amount_paid          │
│  unallocated(payment)  = payment.amount - sum(allocations)    │
│                                                              │
│  client_balance(client):                                     │
│  ├─ total_invoiced  = sum(finalized invoices.total)          │
│  ├─ total_paid      = sum(payments.amount)                   │
│  └─ outstanding     = total_invoiced - total_paid             │
│                                                              │
└──────────────────────────────────────────────────────────────┘
                           │
┌──────────────────────────▼───────────────────────────────────┐
│                    ADAPTER LAYER (Rust)                       │
│              Implements port traits                           │
│                                                              │
│  Persistence                                                 │
│  ├─ SqliteInvoiceRepository                                  │
│  ├─ SqlitePaymentRepository                                  │
│  ├─ SqliteClientRepository                                   │
│  ├─ SqliteServiceRepository                                  │
│  ├─ SqliteTaxRepository                                      │
│  ├─ SqliteTemplateRepository                                 │
│  └─ SqliteSettingsRepository                                 │
│                                                              │
│  Infrastructure                                              │
│  ├─ TypstPdfGenerator                                        │
│  ├─ LettreEmailSender                                        │
│  ├─ StrongholdCredentialStore                                │
│  └─ SqliteInvoiceNumberGenerator                             │
│                                                              │
│  Migration                                                   │
│  └─ LegacyJsonImporter                                       │
│     ├─ Reads old electron-store JSON                         │
│     ├─ Maps cachedClients → Client entities                  │
│     ├─ Maps sellerServices → Service entities                │
│     ├─ Maps sellerTaxes → TaxDefinition entities             │
│     ├─ Maps seller fields → SellerProfile                    │
│     ├─ Maps email config → EmailConfig                       │
│     ├─ Maps currency fields → CurrencyConfig                 │
│     ├─ Seeds invoice number sequence from receiptNumber      │
│     └─ Reports import results to UI                          │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

---

## SQLite Schema

```sql
-- All money stored as INTEGER (cents). No floats.
-- All dates stored as TEXT (ISO 8601).
-- All IDs are TEXT (UUID v4).

CREATE TABLE clients (
    id          TEXT PRIMARY KEY,
    name        TEXT NOT NULL,
    email       TEXT,
    address     TEXT,
    phone       TEXT,
    notes       TEXT,
    active      INTEGER NOT NULL DEFAULT 1,
    created_at  TEXT NOT NULL
);

CREATE TABLE services (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    default_price   INTEGER NOT NULL,       -- cents
    currency        TEXT NOT NULL DEFAULT 'EUR',
    active          INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE tax_definitions (
    id              TEXT PRIMARY KEY,
    name            TEXT NOT NULL,
    percentage      REAL NOT NULL,          -- e.g. 21.0
    tax_id_number   TEXT,
    active          INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE invoice_templates (
    id                  TEXT PRIMARY KEY,
    name                TEXT NOT NULL,
    base_layout         TEXT NOT NULL DEFAULT 'Classic',  -- Classic|Modern|Minimal
    logo_image          BLOB,
    accent_color        TEXT,                -- hex e.g. "#2563EB"
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
    id          TEXT PRIMARY KEY,
    number      INTEGER UNIQUE,             -- NULL while Draft
    client_id   TEXT NOT NULL REFERENCES clients(id),
    template_id TEXT REFERENCES invoice_templates(id),  -- NULL → use default
    date        TEXT NOT NULL,
    due_date    TEXT,
    subtotal    INTEGER NOT NULL,            -- cents
    tax_total   INTEGER NOT NULL,
    total       INTEGER NOT NULL,
    currency    TEXT NOT NULL DEFAULT 'EUR',
    status      TEXT NOT NULL DEFAULT 'Draft',
    pdf_path    TEXT,
    notes       TEXT,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL
);

CREATE TABLE invoice_line_items (
    id          TEXT PRIMARY KEY,
    invoice_id  TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    description TEXT NOT NULL,
    quantity    REAL NOT NULL DEFAULT 1.0,
    unit_price  INTEGER NOT NULL,            -- cents
    total       INTEGER NOT NULL,            -- cents (quantity × unit_price)
    sort_order  INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE invoice_taxes (
    id                  TEXT PRIMARY KEY,
    invoice_id          TEXT NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
    tax_definition_id   TEXT REFERENCES tax_definitions(id),
    tax_name            TEXT NOT NULL,        -- snapshot
    percentage          REAL NOT NULL,        -- snapshot
    tax_id_number       TEXT,                 -- snapshot
    computed_amount     INTEGER NOT NULL      -- cents
);

CREATE TABLE payments (
    id          TEXT PRIMARY KEY,
    client_id   TEXT NOT NULL REFERENCES clients(id),
    date        TEXT NOT NULL,
    amount      INTEGER NOT NULL,            -- cents
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
    amount      INTEGER NOT NULL             -- cents
);

-- Settings as single-row tables (key-value is tempting but typed rows are safer)

CREATE TABLE seller_profile (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),  -- singleton
    name                TEXT NOT NULL DEFAULT '',
    title               TEXT,
    registration_id     TEXT,
    address             TEXT,
    phone               TEXT,
    email               TEXT,
    signature_image     BLOB
);

CREATE TABLE email_config (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    smtp_host           TEXT NOT NULL DEFAULT '',
    smtp_port           INTEGER NOT NULL DEFAULT 587,
    sender_address      TEXT NOT NULL DEFAULT '',
    subject_template    TEXT NOT NULL DEFAULT '',
    body_template       TEXT NOT NULL DEFAULT ''
);

CREATE TABLE currency_config (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    code                TEXT NOT NULL DEFAULT 'EUR',
    symbol              TEXT NOT NULL DEFAULT '€',
    symbol_before       INTEGER NOT NULL DEFAULT 0,  -- 0 = after (100€)
    main_unit_name      TEXT NOT NULL DEFAULT 'euros',
    sub_unit_name       TEXT NOT NULL DEFAULT 'centimes'
);

CREATE TABLE app_preferences (
    id                  INTEGER PRIMARY KEY CHECK (id = 1),
    theme               TEXT NOT NULL DEFAULT 'Light',
    language            TEXT NOT NULL DEFAULT 'fr',
    pdf_output_dir      TEXT NOT NULL DEFAULT ''
);

CREATE TABLE invoice_number_seq (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    next_number INTEGER NOT NULL DEFAULT 1
);

-- === INDEXES ===

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

-- === KEY ACCOUNTING VIEWS ===

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
WHERE c.active = 1;

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
```

---

## Rust Project Structure

```
src-tauri/
├── Cargo.toml
├── src/
│   ├── main.rs                     -- Tauri bootstrap, DI wiring
│   ├── commands/                   -- Tauri #[command] handlers (thin glue)
│   │   ├── mod.rs
│   │   ├── invoice_commands.rs
│   │   ├── payment_commands.rs
│   │   ├── client_commands.rs
│   │   ├── service_commands.rs
│   │   ├── template_commands.rs
│   │   ├── accounting_commands.rs
│   │   ├── settings_commands.rs
│   │   └── data_commands.rs
│   │
│   ├── domain/                     -- Pure domain, zero dependencies
│   │   ├── mod.rs
│   │   ├── invoice.rs              -- Invoice aggregate + InvoiceStatus
│   │   ├── payment.rs              -- Payment aggregate + PaymentAllocation
│   │   ├── client.rs
│   │   ├── service.rs
│   │   ├── tax.rs                  -- TaxDefinition
│   │   ├── template.rs             -- InvoiceTemplate, TemplateLayout, FontChoice
│   │   ├── money.rs                -- Money value object
│   │   ├── line_item.rs
│   │   └── settings.rs             -- SellerProfile, EmailConfig, CurrencyConfig, AppPreferences
│   │
│   ├── application/                -- Use cases, port trait definitions
│   │   ├── mod.rs
│   │   ├── ports/                  -- Trait definitions (interfaces)
│   │   │   ├── mod.rs
│   │   │   ├── invoice_repository.rs
│   │   │   ├── payment_repository.rs
│   │   │   ├── client_repository.rs
│   │   │   ├── service_repository.rs
│   │   │   ├── tax_repository.rs
│   │   │   ├── template_repository.rs
│   │   │   ├── settings_repository.rs
│   │   │   ├── pdf_generator.rs
│   │   │   ├── email_sender.rs
│   │   │   ├── credential_store.rs
│   │   │   └── invoice_number_generator.rs
│   │   │
│   │   ├── invoice_usecases.rs
│   │   ├── payment_usecases.rs
│   │   ├── client_usecases.rs
│   │   ├── service_usecases.rs
│   │   ├── template_usecases.rs
│   │   ├── accounting_queries.rs   -- Read-only: revenue, aging, year-end
│   │   ├── settings_usecases.rs
│   │   └── data_usecases.rs        -- Import/export/backup
│   │
│   └── adapters/                   -- Port implementations
│       ├── mod.rs
│       ├── sqlite/
│       │   ├── mod.rs
│       │   ├── connection.rs        -- Pool setup, migrations
│       │   ├── invoice_repo.rs
│       │   ├── payment_repo.rs
│       │   ├── client_repo.rs
│       │   ├── service_repo.rs
│       │   ├── tax_repo.rs
│       │   ├── template_repo.rs
│       │   ├── settings_repo.rs
│       │   └── number_generator.rs
│       │
│       ├── typst_pdf.rs             -- TypstPdfGenerator
│       ├── lettre_email.rs          -- LettreEmailSender
│       ├── stronghold_creds.rs      -- StrongholdCredentialStore
│       └── legacy_import.rs         -- LegacyJsonImporter
│
├── migrations/                      -- SQL migration files
│   └── 001_initial.sql
│
└── templates/                       -- Shipped Typst layout files
    ├── classic.typ
    ├── modern.typ
    └── minimal.typ
```

---

## Frontend Structure

```
src/
├── main.tsx
├── App.tsx                          -- Router + layout
├── api/                             -- Tauri invoke wrappers (port to backend)
│   ├── invoices.ts
│   ├── payments.ts
│   ├── clients.ts
│   ├── services.ts
│   ├── templates.ts
│   ├── accounting.ts
│   ├── settings.ts
│   └── data.ts
│
├── stores/                          -- Zustand stores
│   ├── invoiceStore.ts
│   ├── paymentStore.ts
│   ├── clientStore.ts
│   ├── templateStore.ts
│   └── settingsStore.ts
│
├── pages/
│   ├── Dashboard.tsx
│   ├── InvoiceCreate.tsx
│   ├── InvoiceList.tsx
│   ├── InvoiceDetail.tsx
│   ├── PaymentList.tsx
│   ├── PaymentRecord.tsx
│   ├── Accounting.tsx
│   ├── ClientList.tsx
│   ├── ClientDetail.tsx
│   ├── TemplateEditor.tsx           -- Layout picker + customization + live preview
│   └── Settings.tsx
│
├── components/
│   ├── layout/                      -- Shell, Sidebar, Header
│   ├── invoice/                     -- LineItemEditor, InvoicePreview, StatusBadge
│   ├── payment/                     -- AllocationForm, PaymentMethodSelect
│   ├── accounting/                  -- RevenueChart, AgingTable, PeriodPicker
│   ├── template/                    -- LayoutPicker, ColorPicker, ToggleGrid, LivePreview
│   ├── client/                      -- ClientForm, ClientSearch
│   └── common/                      -- MoneyInput, DatePicker, DataTable, Modal
│
├── hooks/                           -- Custom React hooks
├── i18n/                            -- FR/EN translation files
│   ├── fr.json
│   └── en.json
│
├── types/                           -- TypeScript types mirroring Rust domain
│   ├── invoice.ts
│   ├── payment.ts
│   ├── client.ts
│   ├── template.ts
│   └── accounting.ts
│
└── styles/
    └── tailwind.css
```

---

## Implementation Phases

### Phase 1 — Foundation
Tauri v2 + React + TS scaffolding. SQLite schema + migrations. Domain entities in Rust. Repository traits + SQLite implementations for Client and Service. Basic React shell with sidebar routing. Settings page (seller profile, currency, preferences). CRUD for Clients and Services.

### Phase 2 — Invoicing Core
Invoice domain aggregate with status machine. Draft → Finalize flow with number assignment. Multi-line item editor. Tax application (snapshot on create). Invoice template system: CRUD, layout picker (Classic/Modern/Minimal), customization panel (logo, accent color, font, field visibility toggles, header/footer text), live preview rendering sample invoice in real time. PDF generation via Typst using selected template. Invoice list with status filtering. Template selector on invoice creation (defaults to is_default template).

### Phase 3 — Email
Lettre SMTP adapter. Credential storage via Stronghold. Email template editor with placeholder preview. Send flow (Finalized → Sent). Test email connection button.

### Phase 4 — Payments & Accounting
Payment recording with allocation UI. Invoice payment status derivation. Outstanding / overdue views. Revenue by period + by client charts. Aging report. Client balance view.

### Phase 5 — Reports & Migration
Year-end report PDF generation. Legacy JSON importer. Database export / backup / restore. Invoice cancellation (PDF watermark).

### Phase 6 — Polish
Design system pass (typography, color, motion). Dark / light theme. i18n (FR/EN). Onboarding flow for first-time setup. Duplicate invoice. Keyboard shortcuts.