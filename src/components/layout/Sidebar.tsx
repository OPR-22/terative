import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import {
  BookOpen,
  Bookmark,
  FileText,
  LayoutDashboard,
  LayoutTemplate,
  Mail,
  PanelLeftClose,
  PanelLeftOpen,
  Package,
  Percent,
  Settings as SettingsIcon,
  Users,
  Wallet,
  type LucideIcon,
} from "lucide-react";

import { BOOKMARKS } from "../../bookmarks";
import { useSidebarStore } from "../../stores/sidebarStore";

interface NavItem {
  to: string;
  label: string;
  icon: LucideIcon;
  end?: boolean;
}

export function Sidebar() {
  const { t } = useTranslation();
  const collapsed = useSidebarStore((s) => s.collapsed);
  const toggle = useSidebarStore((s) => s.toggle);

  const items: NavItem[] = [
    { to: "/", label: t("nav.dashboard"), end: true, icon: LayoutDashboard },
    { to: "/invoices", label: t("nav.invoices"), icon: FileText },
    { to: "/payments", label: t("nav.payments"), icon: Wallet },
    { to: "/clients", label: t("nav.clients"), icon: Users },
    { to: "/catalog", label: t("nav.catalog"), icon: Package },
    { to: "/taxes", label: t("nav.taxes"), icon: Percent },
    { to: "/accounting", label: t("nav.accounting"), icon: BookOpen },
    { to: "/templates", label: t("nav.templates"), icon: LayoutTemplate },
    { to: "/email-templates", label: t("nav.email_templates"), icon: Mail },
    { to: "/settings", label: t("nav.settings"), icon: SettingsIcon },
  ];

  const asideClass = [
    "flex h-full shrink-0 flex-col border-r border-border bg-surface-sunken p-3",
    collapsed ? "w-16 items-center" : "w-56",
  ].join(" ");

  const linkClass = (isActive: boolean) =>
    [
      "flex items-center rounded-field text-sm font-medium transition-colors",
      collapsed ? "h-10 w-10 justify-center" : "gap-3 px-3 py-2",
      isActive
        ? "bg-brand text-brand-fg"
        : "text-fg-muted hover:bg-surface-muted",
    ].join(" ");

  const toggleLabel = collapsed
    ? t("nav.expand_sidebar")
    : t("nav.collapse_sidebar");

  return (
    <aside className={asideClass}>
      <div
        className={[
          "mb-6 flex items-center",
          collapsed ? "flex-col gap-2" : "justify-between gap-2 px-1",
        ].join(" ")}
      >
        {collapsed ? null : (
          <span className="text-xl font-bold tracking-tight text-fg">
            {t("app.title")}
          </span>
        )}
        <button
          type="button"
          onClick={toggle}
          title={toggleLabel}
          aria-label={toggleLabel}
          className="rounded-field p-2 text-fg-muted transition-colors hover:bg-surface-muted hover:text-fg"
        >
          {collapsed ? (
            <PanelLeftOpen className="h-5 w-5" />
          ) : (
            <PanelLeftClose className="h-5 w-5" />
          )}
        </button>
      </div>
      <nav className="flex flex-col gap-1">
        {items.map((item) => {
          const Icon = item.icon;
          return (
            <NavLink
              key={item.to}
              to={item.to}
              end={item.end}
              title={collapsed ? item.label : undefined}
              className={({ isActive }) => linkClass(isActive)}
            >
              <Icon className="h-5 w-5 shrink-0" />
              {collapsed ? null : <span className="truncate">{item.label}</span>}
            </NavLink>
          );
        })}
      </nav>
      {BOOKMARKS.length > 0 ? (
        <div className="mt-6">
          {collapsed ? null : (
            <h2 className="mb-1 px-3 text-xs font-semibold uppercase tracking-wide text-fg-muted">
              {t("nav.bookmarks")}
            </h2>
          )}
          <nav className="flex flex-col gap-1">
            {BOOKMARKS.map((b) => (
              <NavLink
                key={b.id}
                to={`/bookmarks/${b.id}`}
                title={collapsed ? b.label : undefined}
                className={({ isActive }) =>
                  [
                    "flex items-center rounded-field text-sm font-medium transition-colors",
                    collapsed
                      ? "h-10 w-10 justify-center"
                      : "gap-3 truncate px-3 py-2",
                    isActive
                      ? "bg-brand text-brand-fg"
                      : "text-fg-muted hover:bg-surface-muted",
                  ].join(" ")
                }
              >
                <Bookmark className="h-5 w-5 shrink-0" />
                {collapsed ? null : <span className="truncate">{b.label}</span>}
              </NavLink>
            ))}
          </nav>
        </div>
      ) : null}
    </aside>
  );
}
