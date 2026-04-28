import { useTranslation } from "react-i18next";
import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "../ui/Button";

interface PaginationProps {
  first: number;
  last: number;
  previous: number | null;
  next: number | null;
  total: number;
  currentPage: number;
  perPage: number;
  onPageChange: (page: number) => void;
  onPerPageChange: (perPage: number) => void;
}

const PAGE_SIZE_OPTIONS = [10, 25, 50, 100];

export function Pagination({
  last,
  previous,
  next,
  total,
  currentPage,
  perPage,
  onPageChange,
  onPerPageChange,
}: PaginationProps) {
  const { t } = useTranslation();
  if (total === 0) return null;

  return (
    <div className="flex items-center justify-between px-3.5 py-2.5 border-t border-line-soft text-[12px] text-ink-3">
      <div className="flex items-center gap-3">
        <span>{t("pagination.total", { count: total })}</span>
        <label className="flex items-center gap-1.5">
          <span>{t("pagination.per_page")}</span>
          <select
            value={perPage}
            onChange={(e) => onPerPageChange(Number(e.target.value))}
            className="bg-paper border border-line text-ink rounded-sm px-1.5 py-[2px] text-[11px] tabular font-mono"
          >
            {PAGE_SIZE_OPTIONS.map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
      </div>
      {last > 1 ? (
        <div className="flex items-center gap-1">
          <Button
            size="sm"
            variant="default"
            disabled={previous == null}
            onClick={() => previous != null && onPageChange(previous)}
            iconOnly
            aria-label={t("pagination.previous")}
          >
            <ChevronLeft size={12} strokeWidth={1.5} />
          </Button>
          <span className="font-mono tabular px-2 text-[11px]">
            {currentPage} / {last}
          </span>
          <Button
            size="sm"
            variant="default"
            disabled={next == null}
            onClick={() => next != null && onPageChange(next)}
            iconOnly
            aria-label={t("pagination.next")}
          >
            <ChevronRight size={12} strokeWidth={1.5} />
          </Button>
        </div>
      ) : null}
    </div>
  );
}
