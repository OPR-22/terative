import { Fragment } from "react";
import { useTranslation } from "react-i18next";
import { ArrowLeft, ArrowRight, Search } from "lucide-react";
import { Link, useNavigate } from "react-router-dom";

import { useWorkspaceName } from "../../hooks/useWorkspaceName";
import { usePageMeta, type Crumb } from "./PageMeta";

export function Topbar() {
  const { t } = useTranslation();
  const { crumbs } = usePageMeta();
  const navigate = useNavigate();
  const workspaceName = useWorkspaceName();
  const allCrumbs: Crumb[] = [
    { label: workspaceName, to: "/dashboard" },
    ...crumbs,
  ];
  return (
    <div className="flex items-center justify-between px-7 py-3.5 border-b border-line bg-paper">
      <div className="flex items-center gap-3 min-w-0">
        <div className="flex items-center gap-1 shrink-0">
          <button
            type="button"
            aria-label={t("topbar.back")}
            onClick={() => navigate(-1)}
            className="p-1 rounded-sm text-ink-3 hover:text-ink hover:bg-paper-2 cursor-pointer"
          >
            <ArrowLeft size={18} strokeWidth={1.5} />
          </button>
          <button
            type="button"
            aria-label={t("topbar.forward")}
            onClick={() => navigate(1)}
            className="p-1 rounded-sm text-ink-3 hover:text-ink hover:bg-paper-2 cursor-pointer"
          >
            <ArrowRight size={18} strokeWidth={1.5} />
          </button>
        </div>
        <div className="flex items-center gap-2 text-[12px] text-ink-3 min-w-0">
          {allCrumbs.map((c, i) => {
            const isLast = i === allCrumbs.length - 1;
            const className = isLast
              ? "text-ink font-medium truncate"
              : "truncate";
            return (
              <Fragment key={i}>
                {i > 0 ? <span className="text-ink-4">/</span> : null}
                {c.to && !isLast ? (
                  <Link to={c.to} className={`${className} hover:text-ink`}>
                    {c.label}
                  </Link>
                ) : (
                  <span className={className}>{c.label}</span>
                )}
              </Fragment>
            );
          })}
        </div>
      </div>
      <div className="flex items-center gap-2">
        <button
          type="button"
          className="flex items-center gap-2 px-2.5 py-1.5 bg-paper-2 border border-line rounded-sm text-[12px] text-ink-3 min-w-[220px] hover:border-ink-4"
        >
          <Search size={13} strokeWidth={1.5} />
          <span>{t("topbar.search_placeholder")}</span>
          <kbd className="ml-auto font-mono text-[10px] px-1.5 py-px bg-paper-3 border border-line-soft text-ink-3 rounded-sm">
            ⌘K
          </kbd>
        </button>
      </div>
    </div>
  );
}
