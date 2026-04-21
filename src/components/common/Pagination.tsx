import { useTranslation } from "react-i18next";
import { Button } from "./Button";

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
    <div className="flex items-center justify-between pt-4">
      <div className="flex items-center gap-4">
        <p className="text-sm text-fg-muted">
          {t("pagination.total", { count: total })}
        </p>
        <label className="flex items-center gap-2 text-sm text-fg-muted">
          {t("pagination.per_page")}
          <select
            value={perPage}
            onChange={(e) => onPerPageChange(Number(e.target.value))}
            className="rounded-field border border-border bg-surface px-2 py-1 text-sm text-fg shadow-sm"
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
        <div className="flex items-center gap-2">
          <Button
            variant="secondary"
            disabled={previous == null}
            onClick={() => previous != null && onPageChange(previous)}
          >
            {t("pagination.previous")}
          </Button>
          <span className="text-sm text-fg-muted">
            {t("pagination.page_of", { page: currentPage, last })}
          </span>
          <Button
            variant="secondary"
            disabled={next == null}
            onClick={() => next != null && onPageChange(next)}
          >
            {t("pagination.next")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
