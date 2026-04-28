import type { ReactNode } from "react";

import { usePageCrumbs } from "./PageMeta";

interface PageProps {
  crumbs?: ReactNode[];
  title?: ReactNode;
  subtitle?: ReactNode;
  actions?: ReactNode;
  children: ReactNode;
  /** Hide the page-head divider when the page has its own custom chrome. */
  noHeader?: boolean;
}

/**
 * Page wrapper. Sets breadcrumbs in the topbar and renders the page header
 * (title + subtitle + actions) above the content.
 */
export function Page({
  crumbs = [],
  title,
  subtitle,
  actions,
  children,
  noHeader = false,
}: PageProps) {
  usePageCrumbs(crumbs);

  if (noHeader) {
    return <div className="min-h-full">{children}</div>;
  }

  return (
    <div className="min-h-full">
      <header className="flex items-end justify-between gap-4 pb-[18px] mb-5 border-b border-line">
        <div className="min-w-0">
          {title ? (
            <h1 className="text-[22px] font-semibold tracking-[-0.005em] leading-none text-ink m-0">
              {title}
            </h1>
          ) : null}
          {subtitle ? (
            <p className="text-[13px] text-ink-3 mt-1.5">{subtitle}</p>
          ) : null}
        </div>
        {actions ? <div className="flex items-center gap-2 shrink-0">{actions}</div> : null}
      </header>
      {children}
    </div>
  );
}

export function SectionTitle({ children, action }: { children: ReactNode; action?: ReactNode }) {
  return (
    <div className="flex items-center justify-between mt-6 mb-2.5 pb-1.5 border-b border-line-soft text-[12px] font-medium text-ink-3">
      <span>{children}</span>
      {action}
    </div>
  );
}
