# Handoff: Terative — UI redesign

## Overview

Terative is a French invoicing & light-accounting app for independent professionals
and small firms (the demo data is a notary's cabinet, "Cabinet Lemaire"). This package
is a complete UI redesign covering 11 surfaces: dashboard, invoices (list + editor),
payments, clients (list + detail), catalogue, taxes, accounting (3 tabs), invoice templates,
email templates, an in-app web bookmark view, and settings.

## About the design files

Files in `prototype/` are **design references created in HTML/React** — runnable
prototypes showing the intended look, density, and behavior. They are **not** production
code to copy verbatim.

The implementation task is to **recreate these designs in a new React + Tailwind codebase**
(greenfield — pick the most appropriate stack: Vite + React, Next.js App Router, etc.) using
idiomatic patterns. The HTML/CSS here uses raw CSS variables and class names so a developer
can read the values and translate them to Tailwind tokens / shadcn components / whatever is
chosen.

## Fidelity

**High-fidelity.** Final design direction: colors, typography, spacing, layout, copy.
Recreate pixel-perfectly. The only thing intentionally lo-fi is iconography (1.5px
Lucide-style strokes — use the real `lucide-react` package) and avatar imagery (use plain
2-letter initials in colored circles, as shown).

## Locked design direction

After exploration of multiple style directions (editorial, sober, Obsidian-like, Notion-like),
the user selected one combination. **Implement only this one:**

| Aspect | Decision |
|---|---|
| Style | **Sobre** (sober) — sans-only, no decorative serif headlines, sentence-case labels, no sidebar icons (clean text-only nav), 2–3 px corner radii, no shadows |
| Font | **Lato** (Google Fonts), weights 400 / 500 / 700 |
| Background palette | **Papier froid** — cool white paper, slight blue undertone |
| Accent color | **Terracotta** — warm orange-red |
| Theme | Light primary; a dark variant exists in tokens but ship light first |
| Language | French (UI copy is in French — keep it) |
| Number format | French locale: `1 524,00 €` (non-breaking space thousands, comma decimal) |
| Date format | `2 janv. 2026` / `13/04/2026` (mixed, see screens) |

The other styles ("Éditorial", "Obsidian", and the dashboard explorations Notion / Linear
/ Sage / iA Writer) are **not** part of this handoff. They were exploration-only.

## Design tokens

### Colors (OKLCH)

Use these as the source of truth. Tailwind-friendly hex equivalents in the second column.

```
PAPER (backgrounds — papier froid, cool white)
  --paper       oklch(0.985 0.004 240)   ≈ #fafbfc
  --paper-2     oklch(0.97  0.0048 240)  ≈ #f1f3f5    (sidebar, search input bg)
  --paper-3     oklch(0.945 0.006 240)   ≈ #e7eaee    (hover, badge bg, alternating rows)

LINES
  --line        oklch(0.88  0.008 240)   ≈ #c8ced5    (borders, dividers, table cell borders)
  --line-soft   oklch(0.93  0.0064 240)  ≈ #dde2e8    (very subtle dividers)

INK (text)
  --ink         oklch(0.22 0.015 60)     ≈ #2a2622    (primary text)
  --ink-2       oklch(0.38 0.015 60)     ≈ #4d4641    (secondary text, body)
  --ink-3       oklch(0.55 0.015 60)     ≈ #7d7468    (muted, captions, table headers)
  --ink-4       oklch(0.72 0.012 65)     ≈ #b1a89a    (disabled, very muted)

ACCENT (terracotta)
  --accent      oklch(0.62 0.13 40)      ≈ #c46944    (primary buttons, active states, focus)
  --accent-soft oklch(0.94 0.052 40)     ≈ #f7e5da    (active row bg, badge bg, avatar bg)
  --accent-ink  oklch(0.42 0.1235 40)    ≈ #8a4222    (text on accent-soft)

SEMANTIC (status — used in pills / dots / overdue indicators)
  --ok          oklch(0.58 0.11 155)     ≈ #3f8d6a    --ok-soft    oklch(0.94 0.05 155) ≈ #d6ebde
  --warn        oklch(0.68 0.14 70)      ≈ #c9923c    --warn-soft  oklch(0.95 0.06 75)  ≈ #f3e7c9
  --danger      oklch(0.55 0.17 25)      ≈ #c33b3b    --danger-soft oklch(0.94 0.05 25) ≈ #f5dad6
  --info        oklch(0.55 0.1 240)      ≈ #5076a8    --info-soft  oklch(0.94 0.04 240) ≈ #d8e1ee

DARK MODE (deferred — tokens are in styles.css under [data-theme="dark"] if needed later)
```

### Typography

```
Font family — sans only:
  --font-sans:  "Lato", ui-sans-serif, system-ui, sans-serif
  --font-mono:  "JetBrains Mono", ui-monospace, "SF Mono", Menlo, monospace
                (used ONLY for: invoice numbers, monetary amounts in tables,
                 keyboard shortcuts, account refs, dates in dense tables)
```

Weights actually used: 400 (body), 500 (medium / table headers / nav), 700 (bold — sparingly,
for emphasis in callouts and totals).

```
TYPE SCALE (final, after Sobre overrides)
  Page title (h1)         22px / 600 / -0.005em        (NOT serif, NOT 32px — that was the
                                                        editorial style which was rejected)
  Section title (h2)      15px / 600 / 0
  Card title              14px / 600
  Body                    14px / 400 / 1.45
  Label / table header    12px / 500 / sentence-case (NOT uppercase tracked)
  Tiny / kbd / numbers    10–12px
  KPI value (sober)       22px / 600 (NOT 32px)
```

`font-feature-settings: "ss01", "cv11"` is set globally — Lato ignores most of these but it's
harmless. Set `font-variant-numeric: tabular-nums` on every cell containing numbers.

### Spacing & geometry

```
Radius:
  --radius-sm   4px  (chips, kbd)
  --radius      6px  (buttons default — but Sobre pulls these to 2–3px)
  --radius-lg   10px (cards default — but Sobre pulls these to 3px)

Sobre overrides:
  Cards / KPIs            border-radius: 3px
  Buttons / inputs        border-radius: 2px

Shadows:
  --shadow-sm   0 1px 0 oklch(0.2 0 0 / 0.04)
  --shadow-md   0 1px 2px oklch(0.2 0 0 / 0.04), 0 4px 12px oklch(0.2 0 0 / 0.04)
  Sobre uses these very lightly; cards mostly rely on 1px borders, not shadows.
```

Page layout: `224px sidebar | 1fr main`. Topbar 14px×28px padding. Content area 24px×28px
padding. Page-head: title + subtitle on the left, action buttons on the right, separated
from content by an 18px-bottom-padded `1px solid var(--line)` divider.

### Iconography

Lucide (1.5px stroke). Already mapped in `prototype/shell.jsx` under the `I` object. Replace
the inline SVGs with `lucide-react` imports during implementation.

**Sobre rule:** sidebar nav items have **no** icons. Buttons keep their icon-left-of-text.

## Surfaces (11)

Source of truth: open `prototype/index.html` and click through. Each surface is also a
self-contained component file:

| # | Screen | File | Notes |
|---|---|---|---|
| 1 | Tableau de bord | `page-dashboard.jsx` → `DashboardPage` | KPI grid (4 cards) → "À traiter" (overdue table 1.6fr + activity feed 1fr) → 12-month revenue bar chart |
| 2 | Factures · liste | `page-invoices.jsx` → `InvoicesPage` | Pills filter (Toutes/Brouillons/Envoyées/Payées/En retard), table with avatar + name + status badge + amount, bulk actions |
| 2b | Facture · édition | `page-invoices.jsx` → `InvoiceEditorPage` | 2-pane: left = form (client picker, dates, line items with drag handles), right = live preview |
| 3 | Paiements | `page-clients-payments.jsx` → `PaymentsPage` | Stats strip + payments table; modal mock for "Saisir un paiement" |
| 4 | Clients · liste | `page-clients-payments.jsx` → `ClientsPage` | Avatar + name + email + phone + city + tags + CA total + last activity |
| 4b | Client · fiche | `page-clients-payments.jsx` → `ClientDetailPage` | Tabs: Info / Carnet (notes) / Journal (timeline). Header with avatar + summary stats |
| 5 | Catalogue | `page-data.jsx` → `CatalogPage` | Categorized item list, price/unit/VAT, search |
| 5b | Taxes | `page-data.jsx` → `TaxesPage` | TVA rates table, exemption rules, default rate selector |
| 6 | Comptabilité | `page-data.jsx` → `AccountingPage` | Tabs: Journal / Grand-livre / Bilan. Heavy data tables with totals |
| 7 | Modèles de facture | `page-templates-settings.jsx` → `TemplatesPage` | Card grid of templates + selected template editor (logo, fields, footer) |
| 7b | Modèles d'e-mail | `page-templates-settings.jsx` → `EmailsPage` | Trigger list + email body editor with `{{variable}}` chips |
| 8 | Favori (webview) | `page-templates-settings.jsx` → `BookmarkPage` | Embedded browser-style chrome around an external page (Wikipedia mock) |
| 8b | Paramètres | `page-templates-settings.jsx` → `SettingsPage` | Long form: Société, Coordonnées bancaires, Numérotation, Mentions légales, Préférences |

## Shared components

In `prototype/shell.jsx`:

- `Sidebar` — left nav, brand mark "Terative" (Sobre: text-only, no dot/icons), 10 nav items,
  3 favorites, user footer ("Camille L. · Cabinet Lemaire"). Active item: terracotta left border + accent-soft badge.
- `Topbar` — breadcrumbs + search input ("Rechercher partout" + ⌘K kbd) + action slot.
- `PageHead` — h1 + subtitle + action buttons row.
- `Money` — formats numbers as `1 524,00 €` with French locale.
- `Badge` — colored pill (kinds: `paid`, `sent`, `draft`, `overdue`, `partial`, `info`).
- `Pills` — segmented control for list filters with optional counts.
- `Frame` — macOS-style window chrome wrapper used in the canvas; **drop this in the real app** (only useful for previewing).
- `Shell` — composes Sidebar + Topbar + PageHead + content. **The implementation pattern.**

## Interactions & behavior

- **Sidebar nav** updates the active page; in real app this is router-driven.
- **Pills** above tables filter rows by status. Counts shown next to label.
- **Table row hover**: subtle `--paper-2` background.
- **Bulk select**: checkbox in first column; when any row checked, action bar slides above table.
- **"Saisir un paiement" / "Nouvelle facture" / "Nouveau client"**: open modal/drawer.
- **Invoice editor**: line items are drag-reorderable (placeholder grip icon); totals recalc live.
- **Templates page**: clicking a template card loads it into the right-side editor.
- **No animations beyond standard hover/focus transitions** (~150ms ease).

## State (suggested shape — implementation choice)

Greenfield, so pick anything (Zustand, Redux Toolkit, TanStack Query for server state).
Domain entities visible in the design:

```
Invoice    { id, number, client_id, issue_date, due_date, status, lines[], total_ht, vat, total_ttc, paid_amount }
Client     { id, name, email, phone, address, tags[], notes, created_at }
Payment    { id, invoice_id, date, amount, method, reference }
CatalogItem{ id, name, category, unit_price, unit, vat_rate }
TaxRate    { id, label, rate, is_default }
Template   { id, kind: "invoice"|"email", name, body, variables[] }
Settings   { company, bank, numbering, legal, prefs }
```

## Tailwind setup hint

Map the OKLCH tokens directly into `tailwind.config.ts` using arbitrary-value-friendly CSS
variables — that way the prototype's `var(--accent)` references survive translation.

```ts
// tailwind.config.ts
theme: {
  extend: {
    colors: {
      paper: { DEFAULT: 'var(--paper)', 2: 'var(--paper-2)', 3: 'var(--paper-3)' },
      ink:   { DEFAULT: 'var(--ink)',   2: 'var(--ink-2)',   3: 'var(--ink-3)', 4: 'var(--ink-4)' },
      line:  { DEFAULT: 'var(--line)',  soft: 'var(--line-soft)' },
      accent:{ DEFAULT: 'var(--accent)', soft: 'var(--accent-soft)', ink: 'var(--accent-ink)' },
      ok: 'var(--ok)', warn: 'var(--warn)', danger: 'var(--danger)', info: 'var(--info)',
    },
    fontFamily: {
      sans: ['Lato', 'ui-sans-serif', 'system-ui', 'sans-serif'],
      mono: ['JetBrains Mono', 'ui-monospace', 'SF Mono', 'Menlo', 'monospace'],
    },
    borderRadius: { sm: '2px', DEFAULT: '3px', md: '4px', lg: '6px' },
  },
}
```

Then put the `:root` token block from `prototype/index.html`'s frozen `<style>` into your
global stylesheet.

## Files in this package

```
prototype/
  index.html                       ← Open this. Frozen at the chosen direction
                                     (Sobre · Lato · Papier froid · Terracotta).
                                     Left nav switches between the 13 surfaces.
                                     URL hash persists the current screen.
  styles.css                       ← All shared CSS. Token block at the top.
  shell.jsx                        ← Icons, Sidebar, Topbar, Shell, Money, Badge…
  page-dashboard.jsx               ← Surface 1
  page-invoices.jsx                ← Surfaces 2 + 2b
  page-clients-payments.jsx        ← Surfaces 3 + 4 + 4b
  page-data.jsx                    ← Surfaces 5 + 5b + 6
  page-templates-settings.jsx      ← Surfaces 7 + 7b + 8 + 8b
  Terative.html                    ← Original full canvas with Tweaks panel
                                     (kept for reference — shows alternative
                                      directions that were explored and rejected)
  design-canvas.jsx, tweaks-panel.jsx
                                   ← Canvas / tweaks plumbing — DO NOT port to prod.
                                     Scaffold for design exploration only.
```

## Implementation checklist

1. Set up Vite + React + Tailwind + TypeScript.
2. Drop the OKLCH token block into `globals.css` under `:root`.
3. Configure Tailwind as above.
4. Add `lucide-react` and replace inline SVGs in `shell.jsx`.
5. Build `<Shell>` (sidebar + topbar + page head) as the layout primitive.
6. Build `<Money>`, `<Badge>`, `<Pills>` as shared primitives.
7. Build pages 1 → 8b in the order listed above. The Dashboard is the simplest start.
8. Wire React Router (or Next.js App Router) — sidebar nav drives routes.
9. Mock data first; defer real API integration.
10. Lato + JetBrains Mono via `next/font` or `<link>` preconnect — same as prototype.

## What NOT to ship

- The Tweaks panel and the design canvas (`Terative.html`, `design-canvas.jsx`, `tweaks-panel.jsx`).
- Anything from rejected style explorations (Notion / Obsidian / Linear / Sage / iA Writer dashboards). Those are not in `prototype/` but are in the original project.
- The `dot` brand mark and serif "Terative" wordmark — Sobre uses sans-only.
- Page-level KPIs at 32px serif — Sobre uses 22px sans.

## Questions to resolve with PM before coding

- Is dark mode in scope for v1? Tokens exist; light is the chosen default.
- Multi-tenant? The user is "Camille L. / Cabinet Lemaire" — is the workspace switcher in
  the sidebar footer interactive?
- Internationalization: is this French-only or i18n from day one?
- Real PDF rendering for invoices/templates is out of scope of this design — coordinate
  with backend on `/invoices/:id/pdf`.
