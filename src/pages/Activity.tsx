import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { ActivityList } from "../components/activity/ActivityList";
import { Page } from "../components/layout/Page";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { ipc, type AuditDto } from "../ipc";
import { toast } from "../stores/toastStore";

/// Full activity-log page — org-wide feed, newest first. The sidebar entry
/// for this page sits just above Settings.
export function Activity() {
  const { t } = useTranslation();
  const [items, setItems] = useState<AuditDto[] | null>(null);

  useEffect(() => {
    let cancelled = false;
    ipc
      .auditPaginateRecent({ page: 1, per_page: 200 })
      .then((page) => {
        if (!cancelled) setItems(page.data);
      })
      .catch((e) => {
        if (!cancelled) toast.error(e);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <Page title={t("activity.title")}>
      {items == null ? (
        <EmptyState description={t("common.loading")} />
      ) : (
        <Card>
          <ActivityList items={items} />
        </Card>
      )}
    </Page>
  );
}
