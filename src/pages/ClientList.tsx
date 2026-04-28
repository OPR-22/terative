import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Archive, ArchiveRestore, Eye, Plus, Search, Upload } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { Checkbox } from "../components/ui/Checkbox";
import { EmptyState } from "../components/ui/EmptyState";
import { Input } from "../components/ui/Input";
import { Pagination } from "../components/common/Pagination";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { useClientStore } from "../stores/clientStore";
import type { ContactEntryDto } from "../ipc";

const defaultContact = (entries: ContactEntryDto[]): string =>
  entries.find((e) => e.is_default)?.value ?? entries[0]?.value ?? "—";

export function ClientList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    clients,
    page,
    currentPage,
    perPage,
    loading,
    error,
    query,
    setQuery,
    setPage,
    setPerPage,
    refresh,
    archive,
    unarchive,
  } = useClientStore();

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const activeCount = clients.filter((c) => !c.archived_at).length;
  const archivedCount = clients.filter((c) => c.archived_at).length;

  return (
    <Page
      crumbs={[t("clients.title")]}
      title={t("clients.title")}
      subtitle={`${t("clients.summary_active", { count: activeCount })} · ${t("clients.summary_archived", { count: archivedCount })}`}
      actions={
        <>
          <Button leadingIcon={<Upload size={13} strokeWidth={1.5} />}>
            {t("common.import")}
          </Button>
          <Button
            variant="primary"
            leadingIcon={<Plus size={13} strokeWidth={1.5} />}
            onClick={() => navigate("/clients/create")}
          >
            {t("clients.new")}
          </Button>
        </>
      }
    >
      <div className="mb-3.5 flex flex-wrap items-center justify-between gap-3">
        <div className="flex items-center gap-2 flex-1 max-w-md">
          <div className="relative flex-1">
            <Search
              size={13}
              strokeWidth={1.5}
              className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-3"
            />
            <Input
              className="pl-8"
              placeholder={t("common.search") ?? ""}
              value={query.search ?? ""}
              onChange={(e) => setQuery({ ...query, search: e.target.value })}
            />
          </div>
          <Checkbox
            checked={query.include_archived ?? false}
            onChange={(v) => setQuery({ ...query, include_archived: v })}
          >
            {t("common.include_archived")}
          </Checkbox>
        </div>
      </div>

      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Card className="overflow-hidden">
        {loading ? (
          <EmptyState description={t("common.loading")} />
        ) : clients.length === 0 ? (
          <EmptyState description={t("clients.none")} />
        ) : (
          <Table>
            <THead>
              <Tr>
                <Th className="w-8" />
                <Th>{t("common.name")}</Th>
                <Th>{t("common.email")}</Th>
                <Th>{t("common.phone")}</Th>
                <Th>{t("clients.language")}</Th>
                <Th />
              </Tr>
            </THead>
            <tbody>
              {clients.map((c) => (
                <Tr
                  key={c.id}
                  className="cursor-pointer"
                  onClick={() => navigate(`/clients/${c.id}`)}
                >
                  <Td>
                    <Avatar name={c.name} size={26} />
                  </Td>
                  <Td>
                    <div>
                      <div className="font-medium">{c.name}</div>
                      {c.archived_at ? (
                        <Badge kind="draft" className="mt-0.5">
                          {t("clients.archived")}
                        </Badge>
                      ) : null}
                    </div>
                  </Td>
                  <Td muted>{defaultContact(c.emails)}</Td>
                  <Td muted mono>
                    {defaultContact(c.phones)}
                  </Td>
                  <Td>
                    {c.language ? (
                      <Badge kind="outline">{c.language.toUpperCase()}</Badge>
                    ) : (
                      <span className="text-ink-4">—</span>
                    )}
                  </Td>
                  <Td
                    className="text-right whitespace-nowrap"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <div className="flex justify-end gap-1">
                      <Button
                        size="sm"
                        leadingIcon={<Eye size={11} strokeWidth={1.5} />}
                        onClick={() => navigate(`/clients/${c.id}`)}
                      >
                        {t("common.view")}
                      </Button>
                      {c.archived_at ? (
                        <Button
                          size="sm"
                          iconOnly
                          aria-label={t("common.unarchive")}
                          onClick={() => void unarchive(c.id)}
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
                              void archive(c.id);
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
        {page ? (
          <Pagination
            first={page.first}
            last={page.last}
            previous={page.previous}
            next={page.next}
            total={page.total}
            currentPage={currentPage}
            perPage={perPage}
            onPageChange={setPage}
            onPerPageChange={setPerPage}
          />
        ) : null}
      </Card>
    </Page>
  );
}
