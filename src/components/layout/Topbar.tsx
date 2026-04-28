import { Fragment } from "react";
import { Search } from "lucide-react";

import { usePageMeta } from "./PageMeta";

export function Topbar() {
  const { crumbs } = usePageMeta();
  return (
    <div className="flex items-center justify-between px-7 py-3.5 border-b border-line bg-paper">
      <div className="flex items-center gap-2 text-[12px] text-ink-3 min-w-0">
        {crumbs.map((c, i) => (
          <Fragment key={i}>
            {i > 0 ? <span className="text-ink-4">/</span> : null}
            <span
              className={i === crumbs.length - 1 ? "text-ink font-medium truncate" : "truncate"}
            >
              {c}
            </span>
          </Fragment>
        ))}
      </div>
      <div className="flex items-center gap-2">
        <button
          type="button"
          className="flex items-center gap-2 px-2.5 py-1.5 bg-paper-2 border border-line rounded-sm text-[12px] text-ink-3 min-w-[220px] hover:border-ink-4"
        >
          <Search size={13} strokeWidth={1.5} />
          <span>Rechercher partout</span>
          <kbd className="ml-auto font-mono text-[10px] px-1.5 py-px bg-paper-3 border border-line-soft text-ink-3 rounded-sm">
            ⌘K
          </kbd>
        </button>
      </div>
    </div>
  );
}
