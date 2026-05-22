# Terative — TODO

Status: `[ ]` todo · `[~]` in progress · `[x]` done · `[!]` blocked
Tiers: tier-1 foundational · tier-2 new domain/feature · tier-3 targeted UX · tier-4 polish

Task IDs encode the tier (`T<tier>.<seq>`). Adding a task within a tier
appends without reshuffling other tiers.

---

## Tier 1 — Foundational / cross-cutting

- [x] T1.01 Multi-organisation — add `org_id` across schema, repos, queries, use cases. Do before other tier-1 items.
- [x] T1.02 Multi-currency — domain `Money`, per-invoice currency, FX, accounting roll-ups.
- [x] T1.03 Database encryption / password — SQLCipher via `rusqlite`, key derivation, unlock on org selection, backup/restore impact.
- [x] T1.04 Activity history / audit log — `events` table written by use cases, covers invoices/clients/payments/backups.
- [x] T1.05 Invoice file storage — PDFs filed under `<year>/<month>/<number>-<client-slug>.pdf` (year/month from invoice date); configurable starting number in Settings, editable until the first invoice is finalized.
- [x] T1.07 Global search — FTS5 index across clients, invoices, catalog items.
- [ ] T1.08 App updates (like discord ? Like freetube ? apple/microsoft dev account required ?)
- [ ] T1.09 (optional) Families (parent is the client and child is under that parent) - think of a way or something.
- [ ] T1.10 Tax groups (grouping up multiple taxes so it shows up as a single checkbox in UI + default group applied to invoice) 
- [ ] T1.11 Cleanup/refactor of code, split command/queries, move ipc commands to interface, cleanup domains and imports (std::), centralized tests (repo impl for each use case, try to group them up maybe ?)...


## Tier 2 — New domains / significant features

- [ ] T2.00 replace invoice status, have status=draft+finalized+cancelled AND sending_status='unsent,sent'. not grouped as single one for frontend. + update filters for sent+finalized status.
- [ ] T2.01 Projects + time tracking — new aggregate, link to invoice/client, tray-running app, dashboard timer.
- [!] T2.02 Expenses + receipt OCR — blocked: mobile-upload scope conflicts with local-first. Decide flow (QR upload, desktop drop, local OCR vs API).
- [ ] T2.03 PO numbers on invoices — schema + UI + PDF.
- [ ] T2.04 Email send modal — preview, per-send overrides (recipient, template, extra body), drag-drop variable insert, localized variable picker, unbound-variable errors.
- [ ] T2.05 Client merge tool — similar-name matching, reassign allocations.
- [ ] T2.06 CSV transaction export — replaces accounting PDF export, row selection.
- [ ] T2.07 Accounting tax breakdown — pre/post-tax revenue + tax collected per period, trimester grouping.
- [ ] T2.08 CLI to scaffold domains — dev tooling.
- [ ] T2.09 in accounting tab, revenues should be calculated based on either accrual or cash (revenu in 2026 based on invoiced $ OR revenu in 2026 based on payments made). Should support the 2 choices.
- [ ] T2.10 in catalog, the unit should be selectable from a list, not created by user. Actually, almost nothing from dropdowns should be created by users. 
- [ ] T1.11 Discounts on invoices — line-level vs invoice-level, tax interaction, PDF, accounting.
- [ ] T1.12 Promotions, temporary promotions that apply discounts for certain rules (combination of products, quantities etc.)
- [ ] T1.12 Forfaits - user buys a forfait of 100$ that gives him a code, can apply the code for item discounts.
- [ ] t1.13 Recurring invoices (cannot be automatic really, so could just show it in the dashboard in the 'À traiter' section)

## Tier 3 — Targeted feature / UX

- [ ] T3.01 Client creation: Individual vs Company toggle.
- [ ] T3.02 New-invoice page simplified — no payments/emails/preview, edit page keeps full form.
- [ ] T3.03 Highlight newly-created invoice + send-email upsell after finalize.
- [ ] T3.04 Default service auto-selected when only one exists.
- [ ] T3.06 Private note field on invoice (not rendered, label clearly).
- [ ] T3.07 Due-date days flow from settings → invoice; remove placeholder; bigger processed due-date text.
- [ ] T3.08 Invoice note: rename to "public note (inserted in invoice)", drop duplicate hint.
- [ ] T3.09 "Create from model" dropdown reduced to default + selected.
- [ ] T3.10 Logo + signature missing from PDF — bug.
- [ ] T3.11 Pagination component (top + bottom).
- [ ] T3.12 Settings page → tabs for sub-sections.
- [ ] T3.13 Email templates: example-value preview + unbound-variable errors, drag-drop variable insert, localized variable picker, prettier list.
- [ ] T3.14 Email history shows actual sent content from client/invoice.
- [ ] T3.15 Carnet link to settings on empty client page.
- [ ] T3.17 add sorting by invoice # and date 
- [ ] T3.18 Smart save forms in memory (ex: so a user can click on a user profile and come back to the invoice without losing invoice entered data)
- [ ] T3.19 in payments tab, period filter doesnt work, search neither.
- [ ] T3.20 links on dashboard pages dont do what they should do (relance, view activity, see all ...)


## Tier 4 — Polish

- [ ] T4.01 Redo dropdowns.
- [ ] T4.02 Improve invoice item dropdowns.
- [ ] T4.03 Bookmark toolbar redesign to match app.
- [ ] T4.04 Remove all "back" buttons — top nav exists.
- [ ] T4.05 Fix non-functional breadcrumbs (e.g. email template → list).
- [ ] T4.06 Consistent table row actions (edit label vs icon vs row-click).
- [ ] T4.07 Full dates in invoice view.
- [ ] T4.08 "Encaisser" button must use coins icon everywhere.
- [ ] T4.09 Description on "cancel invoice" action.
- [ ] T4.10 Loaders on slow-action buttons (invoice create, email send).
- [ ] T4.11 Recharts (or similar) for dashboard + accounting graphs.
- [ ] T4.12 UI animations — tab switch, sidebar collapse.
- [ ] T4.13 Increase logo icon weight + add to sidebar top.
- [ ] T4.14 Remove hover on top-4 dashboard cards.
- [ ] T4.15 Error toast lasts 5s.
- [ ] T4.16 Remove duplicate workspace name (bottom-left).
- [ ] T4.17 Remove import button on client page.
- [ ] T4.18 Default values: countries, languages, phone/email labels.
- [ ] T4.19 Review product catalog defaults (unit, quantity).
- [ ] T4.20 Make sure backups/restore work cross-platform.
- [ ] T4.21 Add 'mark as paid' button on the 'late invoice' section in dashboard.
- [ ] T4.22 Hide $ numbers on dashboards, like wealthsimple (replace number by dots)

## Tier 5 - Marketing
- [ ] T5.01 Website  
