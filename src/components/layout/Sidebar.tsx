import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  [
    "block rounded-field px-3 py-2 text-sm font-medium transition-colors",
    isActive
      ? "bg-brand text-brand-fg"
      : "text-fg-muted hover:bg-surface-muted",
  ].join(" ");

// Hardcoded bookmarks for MVP. Keep in sync with `BOOKMARKS` in BookmarkView.
const BOOKMARKS = [{ id: "example", label: "Google" }];

export function Sidebar() {
  const { t } = useTranslation();
  const items = [
    { to: "/", label: t("nav.dashboard"), end: true },
    { to: "/invoices", label: t("nav.invoices") },
    { to: "/payments", label: t("nav.payments") },
    { to: "/clients", label: t("nav.clients") },
    { to: "/catalog", label: t("nav.catalog") },
    { to: "/taxes", label: t("nav.taxes") },
    { to: "/accounting", label: t("nav.accounting") },
    { to: "/templates", label: t("nav.templates") },
    { to: "/email-templates", label: t("nav.email_templates") },
    { to: "/settings", label: t("nav.settings") },
  ];
  return (
    <aside className="flex h-full w-56 shrink-0 flex-col border-r border-border bg-surface-sunken p-3">
      <div className="mb-6 px-3 py-2">
        <span className="text-xl font-bold tracking-tight text-fg">
          {t("app.title")}
        </span>
      </div>
      <nav className="flex flex-col gap-1">
        {items.map((item) => (
          <NavLink key={item.to} to={item.to} end={item.end} className={linkClass}>
            {item.label}
          </NavLink>
        ))}
      </nav>
      {BOOKMARKS.length > 0 ? (
        <div className="mt-6">
          <h2 className="mb-1 px-3 text-xs font-semibold uppercase tracking-wide text-fg-muted">
            {t("nav.bookmarks")}
          </h2>
          <nav className="flex flex-col gap-1">
            {BOOKMARKS.map((b) => (
              <NavLink
                key={b.id}
                to={`/bookmarks/${b.id}`}
                className={linkClass}
              >
                {b.label}
              </NavLink>
            ))}
          </nav>
        </div>
      ) : null}
    </aside>
  );
}
