# Terative — TODO

Status: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked
Tiers: tier-1 foundational · tier-2 new domain/feature · tier-3 targeted UX · tier-4 polish

---

## Tier 1 — Foundational / cross-cutting

- [ ] T01 [tier-1] Multi-organisation — add `org_id` across schema, repos, queries, use cases. Do before other tier-1 items.
- [ ] T02 [tier-1] Multi-currency — domain `Money`, per-invoice currency, FX, accounting roll-ups.
- [ ] T03 [tier-1] Database encryption / password — SQLCipher via `rusqlite`, key derivation, unlock at app start, backup/restore impact.
- [ ] T04 [tier-1] Activity history / audit log — `events` table written by use cases, covers invoices/clients/payments/backups.
- [ ] T05 [tier-1] Invoice file storage — `year/month/<name>.pdf` layout, numbering reset to `000001`, client name in filename, migrate existing files.
- [ ] T06 [tier-1] Discounts on invoices — line-level vs invoice-level, tax interaction, PDF, accounting.
- [ ] T07 [tier-1] Global search — FTS5 index across clients, invoices, catalog items.

## Tier 2 — New domains / significant features

- [ ] T08 [tier-2] Projects + time tracking — new aggregate, link to invoice/client, tray-running app, dashboard timer.
- [!] T09 [tier-2] Expenses + receipt OCR — blocked: mobile-upload scope conflicts with local-first. Decide flow (QR upload, desktop drop, local OCR vs API).
- [ ] T10 [tier-2] PO numbers on invoices — schema + UI + PDF.
- [ ] T11 [tier-2] Email send modal — preview, per-send overrides (recipient, template, extra body), drag-drop variable insert, localized variable picker, unbound-variable errors.
- [ ] T12 [tier-2] Client merge tool — similar-name matching, reassign allocations.
- [ ] T13 [tier-2] CSV transaction export — replaces accounting PDF export, row selection.
- [ ] T14 [tier-2] Accounting tax breakdown — pre/post-tax revenue + tax collected per period, trimester grouping.
- [ ] T15 [tier-2] CLI to scaffold domains — dev tooling.

## Tier 3 — Targeted feature / UX

- [ ] T16 [tier-3] Client creation: Individual vs Company toggle.
- [ ] T17 [tier-3] New-invoice page simplified — no payments/emails/preview, edit page keeps full form.
- [ ] T18 [tier-3] Highlight newly-created invoice + send-email upsell after finalize.
- [ ] T19 [tier-3] Default service auto-selected when only one exists.
- [ ] T20 [tier-3] Taxes toggled ON by default.
- [ ] T21 [tier-3] Private note field on invoice (not rendered, label clearly).
- [ ] T22 [tier-3] Due-date days flow from settings → invoice; remove placeholder; bigger processed due-date text.
- [ ] T23 [tier-3] Invoice note: rename to "public note (inserted in invoice)", drop duplicate hint.
- [ ] T24 [tier-3] "Create from model" dropdown reduced to default + selected.
- [ ] T25 [tier-3] Logo + signature missing from PDF — bug.
- [ ] T26 [tier-3] Pagination component (top + bottom).
- [ ] T27 [tier-3] Settings page → tabs for sub-sections.
- [ ] T28 [tier-3] Email templates: example-value preview + unbound-variable errors; prettier list.
- [ ] T29 [tier-3] Email history shows actual sent content from client/invoice.
- [ ] T30 [tier-3] Carnet link on empty client page.

## Tier 4 — Polish

- [ ] T31 [tier-4] Redo dropdowns.
- [ ] T32 [tier-4] Improve invoice item dropdowns.
- [ ] T33 [tier-4] Bookmark toolbar redesign to match app.
- [ ] T34 [tier-4] Remove all "back" buttons — top nav exists.
- [ ] T35 [tier-4] Fix non-functional breadcrumbs (e.g. email template → list).
- [ ] T36 [tier-4] Consistent table row actions (edit label vs icon vs row-click).
- [ ] T37 [tier-4] Full dates in invoice view.
- [ ] T38 [tier-4] "Encaisser" button uses coins icon.
- [ ] T39 [tier-4] Description on "cancel invoice" action.
- [ ] T40 [tier-4] Loaders on slow-action buttons (invoice create, email send).
- [ ] T41 [tier-4] Recharts (or similar) for dashboard + accounting graphs.
- [ ] T42 [tier-4] UI animations — tab switch, sidebar collapse.
- [ ] T43 [tier-4] Logo weight + add to sidebar top.
- [ ] T44 [tier-4] Remove hover on top-4 dashboard cards.
- [ ] T45 [tier-4] Error toast lasts 5s.
- [ ] T46 [tier-4] Remove duplicate workspace name (bottom-left).
- [ ] T47 [tier-4] Remove import button on client page.
- [ ] T48 [tier-4] Default values: countries, languages, phone/email labels.
- [ ] T49 [tier-4] Review product catalog defaults (unit, quantity).
