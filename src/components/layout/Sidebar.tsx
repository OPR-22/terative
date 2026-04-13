import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";

const linkClass = ({ isActive }: { isActive: boolean }) =>
  [
    "block rounded-md px-3 py-2 text-sm font-medium transition-colors",
    isActive
      ? "bg-blue-600 text-white"
      : "text-zinc-700 hover:bg-zinc-100",
  ].join(" ");

export function Sidebar() {
  const { t } = useTranslation();
  const items = [
    { to: "/", label: t("nav.dashboard"), end: true },
    { to: "/invoices", label: t("nav.invoices") },
    { to: "/payments", label: t("nav.payments") },
    { to: "/clients", label: t("nav.clients") },
    { to: "/services", label: t("nav.services") },
    { to: "/accounting", label: t("nav.accounting") },
    { to: "/templates", label: t("nav.templates") },
    { to: "/settings", label: t("nav.settings") },
  ];
  return (
    <aside className="flex h-full w-56 shrink-0 flex-col border-r border-zinc-200 bg-zinc-50 p-3">
      <div className="mb-6 px-3 py-2">
        <span className="text-xl font-bold tracking-tight text-zinc-900">
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
    </aside>
  );
}
