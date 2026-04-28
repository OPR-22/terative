import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  Copy,
  Download,
  Edit,
  Eye,
  MoreHorizontal,
  Plus,
  Send,
  Trash2,
} from "lucide-react";

import { Page } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Pills } from "../components/ui/Pills";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { Pagination } from "../components/common/Pagination";
import { StatusBadge } from "../components/invoice/StatusBadge";
import { PaymentStatusBadge } from "../components/invoice/PaymentStatusBadge";
import { MarkPaidModal } from "../components/invoice/MarkPaidModal";
import { useMoneyFormat } from "../lib/money";
import { useInvoiceStore } from "../stores/invoiceStore";
import { useClientStore } from "../stores/clientStore";
import type { InvoiceDto, InvoiceStatusDto } from "../ipc";

type FilterValue = "all" | InvoiceStatusDto;

export function InvoiceList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    invoices,
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
    finalize,
    duplicate,
    cancel,
    send,
  } = useInvoiceStore();
  const ensureDirectory = useClientStore((s) => s.ensureDirectory);
  const clientName = useClientStore((s) => s.clientName);
  const { format } = useMoneyFormat();
  const [payFor, setPayFor] = useState<InvoiceDto | null>(null);

  useEffect(() => {
    void refresh();
    void ensureDirectory();
  }, [refresh, ensureDirectory]);
  const filterValue: FilterValue = query.status ?? "all";

  return (
    <Page
      crumbs={["Cabinet Lemaire", t("invoices.title")]}
      title={t("invoices.title")}
      subtitle={
        page ? `${page.total} factures au total · ${invoices.length} affichées` : undefined
      }
      actions={
        <>
          <Button leadingIcon={<Download size={13} strokeWidth={1.5} />}>
            Exporter
          </Button>
          <Button
            variant="primary"
            leadingIcon={<Plus size={13} strokeWidth={1.5} />}
            onClick={() => navigate("/invoices/create")}
          >
            {t("invoices.new")}
          </Button>
        </>
      }
    >
      <div className="mb-3.5 flex flex-wrap items-center justify-between gap-3">
        <Pills<FilterValue>
          value={filterValue}
          onChange={(id) =>
            setQuery({ ...query, status: id === "all" ? null : id })
          }
          options={[
            { id: "all", label: t("invoices.all") },
            { id: "Draft", label: t("invoices.status_draft") },
            { id: "Finalized", label: t("invoices.status_finalized") },
            { id: "Sent", label: t("invoices.status_sent") },
            { id: "Cancelled", label: t("invoices.status_cancelled") },
          ]}
        />
      </div>

      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Card className="overflow-hidden">
        {loading ? (
          <EmptyState description={t("common.loading")} />
        ) : invoices.length === 0 ? (
          <EmptyState description={t("invoices.none")} />
        ) : (
          <Table>
            <THead>
              <Tr>
                <Th>N°</Th>
                <Th>{t("common.date")}</Th>
                <Th>{t("invoices.client")}</Th>
                <Th>{t("common.status")}</Th>
                <Th>{t("invoices.payment")}</Th>
                <Th numeric>{t("invoices.total")}</Th>
                <Th />
              </Tr>
            </THead>
            <tbody>
              {invoices.map((inv) => {
                const name = clientName(inv.client_id);
                return (
                  <Tr key={inv.id}>
                    <Td mono className={inv.number == null ? "text-ink-4" : ""}>
                      {inv.number ?? "—"}
                    </Td>
                    <Td muted mono>
                      {inv.date}
                    </Td>
                    <Td>
                      <div className="flex items-center gap-2">
                        <Avatar name={name} size={22} />
                        <span>{name}</span>
                      </div>
                    </Td>
                    <Td>
                      <StatusBadge status={inv.status} />
                    </Td>
                    <Td>
                      <PaymentStatusBadge
                        paymentStatus={inv.payment_status}
                        rawStatus={inv.status}
                      />
                    </Td>
                    <Td numeric className="font-medium">
                      {format(inv.total)}
                    </Td>
                    <Td className="text-right whitespace-nowrap">
                      <div className="flex justify-end gap-1">
                        <Button
                          size="sm"
                          leadingIcon={
                            inv.status === "Draft" ? (
                              <Edit size={11} strokeWidth={1.5} />
                            ) : (
                              <Eye size={11} strokeWidth={1.5} />
                            )
                          }
                          onClick={() => navigate(`/invoices/${inv.id}/edit`)}
                        >
                          {t(inv.status === "Draft" ? "common.edit" : "common.view")}
                        </Button>
                        {inv.status === "Draft" ? (
                          <Button
                            size="sm"
                            variant="primary"
                            onClick={() =>
                              void finalize(inv.id).catch((e) => alert(String(e)))
                            }
                          >
                            {t("invoices.finalize")}
                          </Button>
                        ) : null}
                        {inv.status === "Finalized" || inv.status === "Sent" ? (
                          <Button
                            size="sm"
                            variant="accent"
                            leadingIcon={<Send size={11} strokeWidth={1.5} />}
                            onClick={() =>
                              void send(inv.id).catch((e) => alert(String(e)))
                            }
                          >
                            {inv.email_sends.length === 0
                              ? t("invoices.send")
                              : t("invoices.send_reminder")}
                          </Button>
                        ) : null}
                        {(inv.status === "Finalized" || inv.status === "Sent") &&
                        inv.payment_status !== "Paid" ? (
                          <Button size="sm" onClick={() => setPayFor(inv)}>
                            {t("invoices.mark_paid")}
                          </Button>
                        ) : null}
                        <Button
                          size="sm"
                          iconOnly
                          aria-label={t("invoices.duplicate")}
                          onClick={() => void duplicate(inv.id)}
                        >
                          <Copy size={12} strokeWidth={1.5} />
                        </Button>
                        {inv.status === "Finalized" || inv.status === "Sent" ? (
                          <Button
                            size="sm"
                            iconOnly
                            variant="danger"
                            aria-label={t("invoices.cancel")}
                            onClick={() => {
                              if (confirm(t("invoices.confirm_cancel"))) {
                                void cancel(inv.id).catch((e) => alert(String(e)));
                              }
                            }}
                          >
                            <Trash2 size={12} strokeWidth={1.5} />
                          </Button>
                        ) : (
                          <Button size="sm" iconOnly aria-label="Plus">
                            <MoreHorizontal size={12} strokeWidth={1.5} />
                          </Button>
                        )}
                      </div>
                    </Td>
                  </Tr>
                );
              })}
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

      {payFor ? (
        <MarkPaidModal
          invoice={payFor}
          onClose={() => setPayFor(null)}
          onPaid={() => void refresh()}
        />
      ) : null}
    </Page>
  );
}
