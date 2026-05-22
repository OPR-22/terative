import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { ActivityList } from "../components/activity/ActivityList";
import { AuditCleanup } from "../components/activity/AuditCleanup";
import { Page } from "../components/layout/Page";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { ipc, type AuditDto } from "../ipc";
import { toast } from "../stores/toastStore";

const PER_PAGE = 50;

/// Full activity-log page — org-wide feed, newest first, paginated.
/// "Load more" appends older entries beneath the current ones.
export function Activity() {
  const { t } = useTranslation();
  const [items, setItems] = useState<AuditDto[] | null>(null);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);
  const [hasMore, setHasMore] = useState(false);
  const [loadingMore, setLoadingMore] = useState(false);

  // Initial / reload: fetch page 1 fresh.
  const reload = useCallback(() => {
    let cancelled = false;
    ipc
      .auditPaginateRecent({ page: 1, per_page: PER_PAGE })
      .then((p) => {
        if (cancelled) return;
        setItems(p.data);
        setPage(1);
        setTotal(Number(p.total));
        setHasMore(p.next != null);
      })
      .catch((e) => {
        if (!cancelled) toast.error(e);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => reload(), [reload]);

  const loadMore = useCallback(() => {
    if (loadingMore || !hasMore) return;
    setLoadingMore(true);
    const next = page + 1;
    ipc
      .auditPaginateRecent({ page: next, per_page: PER_PAGE })
      .then((p) => {
        setItems((prev) => (prev ? [...prev, ...p.data] : p.data));
        setPage(next);
        setTotal(Number(p.total));
        setHasMore(p.next != null);
      })
      .catch((e) => toast.error(e))
      .finally(() => setLoadingMore(false));
  }, [hasMore, loadingMore, page]);

  return (
    <Page title={t("activity.title")}>
      {items == null ? (
        <EmptyState description={t("common.loading")} />
      ) : (
        <>
          <Card>
            <ActivityList
              items={items}
              header={{ title: t("activity.title"), count: total }}
              loadMore={hasMore ? loadMore : undefined}
              loadingMore={loadingMore}
            />
          </Card>
          <div className="mt-4 flex justify-end">
            <AuditCleanup onCleaned={reload} />
          </div>
        </>
      )}
    </Page>
  );
}
