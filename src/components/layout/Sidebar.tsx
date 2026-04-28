import { useEffect } from "react";
import { NavLink } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Bookmark, PanelLeftClose, PanelLeftOpen } from "lucide-react";

import { useBookmarkStore } from "../../stores/bookmarkStore";
import { useSidebarStore } from "../../stores/sidebarStore";

interface NavItem {
  to: string;
  label: string;
  end?: boolean;
}

export function Sidebar() {
  const { t } = useTranslation();
  const collapsed = useSidebarStore((s) => s.collapsed);
  const toggle = useSidebarStore((s) => s.toggle);
  const bookmarks = useBookmarkStore((s) => s.bookmarks);
  const ensureLoaded = useBookmarkStore((s) => s.ensureLoaded);

  useEffect(() => {
    void ensureLoaded();
  }, [ensureLoaded]);

  const items: NavItem[] = [
    { to: "/dashboard", label: t("nav.dashboard") },
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
    <aside
      className={[
        "flex flex-col bg-paper-2 border-r border-line shrink-0 min-h-0",
        collapsed ? "w-16 items-center" : "w-56",
      ].join(" ")}
    >
      {/* Brand row */}
      <div
        className={[
          "flex items-center justify-between border-b border-line-soft",
          collapsed ? "py-3.5 flex-col gap-2" : "px-5 py-[18px]",
        ].join(" ")}
      >
        {collapsed ? null : (
          <span className="text-[16px] font-semibold tracking-[-0.005em] text-ink">
            {t("app.title")}
          </span>
        )}
        <button
          type="button"
          onClick={toggle}
          title={collapsed ? t("nav.expand_sidebar") : t("nav.collapse_sidebar")}
          aria-label={collapsed ? t("nav.expand_sidebar") : t("nav.collapse_sidebar")}
          className="grid place-items-center w-[22px] h-[22px] text-ink-3 hover:text-ink cursor-pointer"
        >
          {collapsed ? <PanelLeftOpen size={16} /> : <PanelLeftClose size={16} />}
        </button>
      </div>

      <div className="px-2.5 pt-3.5 pb-1.5">
        <NavList items={items} collapsed={collapsed} />
      </div>

      {bookmarks.length > 0 && !collapsed ? (
        <div className="px-2.5 pt-3.5 pb-1.5">
          <div className="px-2.5 pb-1.5 text-[11px] font-medium text-ink-3 tracking-[0.04em]">
            {t("nav.bookmarks")}
          </div>
          <nav className="flex flex-col gap-px">
            {bookmarks.map((b) => (
              <NavLink
                key={b.id}
                to={`/bookmarks/${b.id}`}
                title={b.label}
                className={({ isActive }) =>
                  navItemClass(isActive, false, true)
                }
              >
                <Bookmark className="w-3.5 h-3.5 shrink-0" strokeWidth={1.5} />
                <span className="truncate">{b.label}</span>
              </NavLink>
            ))}
          </nav>
        </div>
      ) : null}

      <div
        className={[
          "mt-auto border-t border-line-soft",
          collapsed ? "p-2 flex justify-center" : "px-3 py-3 flex items-center gap-2.5",
        ].join(" ")}
      >
        <span className="grid place-items-center w-[26px] h-[26px] bg-accent-soft text-accent-ink text-[11px] font-semibold rounded-full shrink-0">
          CL
        </span>
        {collapsed ? null : (
          <div className="leading-tight min-w-0">
            <div className="text-ink font-medium text-[12px] truncate">Camille L.</div>
            <div className="text-[11px] text-ink-3 truncate">Cabinet Lemaire</div>
          </div>
        )}
      </div>
    </aside>
  );
}

function NavList({ items, collapsed }: { items: NavItem[]; collapsed: boolean }) {
  return (
    <nav className="flex flex-col gap-px">
      {items.map((item) => (
        <NavLink
          key={item.to}
          to={item.to}
          end={item.end}
          title={collapsed ? item.label : undefined}
          className={({ isActive }) => navItemClass(isActive, collapsed)}
        >
          <span className="truncate">{collapsed ? item.label.charAt(0) : item.label}</span>
        </NavLink>
      ))}
    </nav>
  );
}

function navItemClass(isActive: boolean, collapsed = false, withIcon = false): string {
  return [
    "flex items-center text-[13px] cursor-pointer transition-colors border-l-2 -ml-[2px]",
    collapsed
      ? "justify-center w-10 h-10 px-0 -ml-0 border-l-0"
      : withIcon
        ? "gap-2.5 pl-[14px] pr-2.5 py-[7px]"
        : "pl-[14px] pr-2.5 py-[7px]",
    isActive
      ? "bg-paper text-ink font-medium border-l-accent"
      : "text-ink-2 border-l-transparent hover:bg-paper-3 hover:text-ink",
  ].join(" ");
}
