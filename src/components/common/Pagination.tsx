import { useTranslation } from "react-i18next";
import { Button } from "./Button";

interface PaginationProps {
  first: number;
  last: number;
  previous: number | null;
  next: number | null;
  total: number;
  currentPage: number;
  onPageChange: (page: number) => void;
}

export function Pagination({
  last,
  previous,
  next,
  total,
  currentPage,
  onPageChange,
}: PaginationProps) {
  const { t } = useTranslation();

  if (last <= 1) return null;

  return (
    <div className="flex items-center justify-between pt-4">
      <p className="text-sm text-fg-muted">
        {t("pagination.total", { count: total })}
      </p>
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
    </div>
  );
}
