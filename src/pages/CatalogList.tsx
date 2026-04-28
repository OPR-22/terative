import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Archive, ArchiveRestore, Edit, Plus } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { EmptyState } from "../components/ui/EmptyState";
import { Pills } from "../components/ui/Pills";
import { StatusDot } from "../components/ui/StatusDot";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { useMoneyFormat } from "../lib/money";
import { useCatalogStore } from "../stores/catalogStore";
import type { CatalogItemKindDto } from "../ipc";

type KindFilter = "All" | CatalogItemKindDto;

export function CatalogList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    items,
    loading,
    error,
    includeArchived,
    setIncludeArchived,
    refresh,
    archive,
    unarchive,
  } = useCatalogStore();
  const [kindFilter, setKindFilter] = useState<KindFilter>("All");

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const { format } = useMoneyFormat();

  const visibleItems =
    kindFilter === "All" ? items : items.filter((i) => i.kind === kindFilter);

  const counts = {
    All: items.length,
    Service: items.filter((i) => i.kind === "Service").length,
    Product: items.filter((i) => i.kind === "Product").length,
  };

  return (
    <Page
      crumbs={[t("catalog.title")]}
      title={t("catalog.title")}
      subtitle={[
        t("catalog.summary_total", { count: counts.All }),
        t("catalog.summary_products", { count: counts.Product }),
        t("catalog.summary_services", { count: counts.Service }),
      ].join(" · ")}
      actions={
        <Button
          variant="primary"
          leadingIcon={<Plus size={13} strokeWidth={1.5} />}
          onClick={() => navigate("/catalog/create")}
        >
          {t("catalog.new")}
        </Button>
      }
    >
      <div className="mb-3.5 flex items-center justify-between gap-3">
        <Pills<KindFilter>
          value={kindFilter}
          onChange={setKindFilter}
          options={[
            { id: "All", label: t("catalog.filter_all"), count: counts.All },
            {
              id: "Service",
              label: t("catalog.kind_service_plural"),
              count: counts.Service,
            },
            {
              id: "Product",
              label: t("catalog.kind_product_plural"),
              count: counts.Product,
            },
          ]}
        />
        <Checkbox checked={includeArchived} onChange={setIncludeArchived}>
          {t("common.include_archived")}
        </Checkbox>
      </div>

      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Card className="overflow-hidden">
        {loading ? (
          <EmptyState description={t("common.loading")} />
        ) : visibleItems.length === 0 ? (
          <EmptyState description={t("catalog.none")} />
        ) : (
          <Table>
            <THead>
              <Tr>
                <Th>{t("catalog.kind")}</Th>
                <Th>{t("common.name")}</Th>
                <Th>{t("catalog.reference")}</Th>
                <Th numeric>{t("catalog.default_price")}</Th>
                <Th>{t("catalog.unit")}</Th>
                <Th>{t("common.status")}</Th>
                <Th />
              </Tr>
            </THead>
            <tbody>
              {visibleItems.map((s) => (
                <Tr key={s.id}>
                  <Td>
                    <Badge kind={s.kind === "Product" ? "outline" : "info"}>
                      {t(`catalog.kind_${s.kind.toLowerCase()}`)}
                    </Badge>
                  </Td>
                  <Td className="font-medium">{s.name}</Td>
                  <Td muted mono>
                    {s.reference ?? "—"}
                  </Td>
                  <Td numeric>{format(s.default_price)}</Td>
                  <Td muted>{s.unit ?? "—"}</Td>
                  <Td>
                    <span className="inline-flex items-center gap-1.5 text-[12px]">
                      {s.archived_at ? (
                        <>
                          <StatusDot status="idle" />
                          <span className="text-ink-3">
                            {t("common.archived_status")}
                          </span>
                        </>
                      ) : (
                        <>
                          <StatusDot status="ok" />
                          <span className="text-ok-ink">
                            {t("common.active_status")}
                          </span>
                        </>
                      )}
                    </span>
                  </Td>
                  <Td className="text-right whitespace-nowrap">
                    <div className="flex justify-end gap-1">
                      <Button
                        size="sm"
                        leadingIcon={<Edit size={11} strokeWidth={1.5} />}
                        onClick={() => navigate(`/catalog/${s.id}/edit`)}
                      >
                        {t("common.edit")}
                      </Button>
                      {s.archived_at ? (
                        <Button
                          size="sm"
                          iconOnly
                          aria-label={t("common.unarchive")}
                          onClick={() => void unarchive(s.id)}
                        >
                          <ArchiveRestore size={12} strokeWidth={1.5} />
                        </Button>
                      ) : (
                        <Button
                          size="sm"
                          iconOnly
                          variant="danger"
                          aria-label={t("common.archive")}
                          onClick={() => {
                            if (confirm(t("common.confirm_archive"))) {
                              void archive(s.id);
                            }
                          }}
                        >
                          <Archive size={12} strokeWidth={1.5} />
                        </Button>
                      )}
                    </div>
                  </Td>
                </Tr>
              ))}
            </tbody>
          </Table>
        )}
      </Card>
    </Page>
  );
}
