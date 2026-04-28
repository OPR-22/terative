import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Archive, ArchiveRestore, Edit, Plus } from "lucide-react";

import { Page } from "../components/layout/Page";
import { useWorkspaceName } from "../hooks/useWorkspaceName";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { EmptyState } from "../components/ui/EmptyState";
import { StatusDot } from "../components/ui/StatusDot";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { useTaxStore } from "../stores/taxStore";

export function TaxList() {
  const { t } = useTranslation();
  const workspaceName = useWorkspaceName();
  const navigate = useNavigate();
  const {
    taxes,
    includeArchived,
    setIncludeArchived,
    loading,
    error,
    refresh,
    archive,
    unarchive,
  } = useTaxStore();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const activeCount = taxes.filter((t) => !t.archived_at).length;

  return (
    <Page
      crumbs={[workspaceName, t("taxes.title")]}
      title={t("taxes.title")}
      subtitle={t("taxes.summary_active", { count: activeCount })}
      actions={
        <Button
          variant="primary"
          leadingIcon={<Plus size={13} strokeWidth={1.5} />}
          onClick={() => navigate("/taxes/create")}
        >
          {t("taxes.new")}
        </Button>
      }
    >
      <div className="mb-3.5 flex items-center justify-between">
        <Checkbox checked={includeArchived} onChange={setIncludeArchived}>
          {t("common.include_archived")}
        </Checkbox>
      </div>

      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Card className="overflow-hidden max-w-3xl">
        {loading ? (
          <EmptyState description={t("common.loading")} />
        ) : taxes.length === 0 ? (
          <EmptyState description={t("taxes.none")} />
        ) : (
          <Table>
            <THead>
              <Tr>
                <Th>{t("common.name")}</Th>
                <Th numeric>{t("taxes.percentage")}</Th>
                <Th>{t("taxes.tax_id_number")}</Th>
                <Th>{t("common.status")}</Th>
                <Th />
              </Tr>
            </THead>
            <tbody>
              {taxes.map((tax) => (
                <Tr key={tax.id}>
                  <Td className="font-medium">{tax.name}</Td>
                  <Td numeric className="text-[14px]">
                    {tax.percentage}&nbsp;%
                  </Td>
                  <Td muted mono>
                    {tax.tax_id_number ?? "—"}
                  </Td>
                  <Td>
                    <span className="inline-flex items-center gap-1.5 text-[12px]">
                      {tax.archived_at ? (
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
                        onClick={() => navigate(`/taxes/${tax.id}/edit`)}
                      >
                        {t("common.edit")}
                      </Button>
                      {tax.archived_at ? (
                        <Button
                          size="sm"
                          iconOnly
                          aria-label={t("common.unarchive")}
                          onClick={() => void unarchive(tax.id)}
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
                              void archive(tax.id);
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
