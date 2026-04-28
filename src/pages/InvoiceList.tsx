import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate } from "react-router-dom";
import { Coins, Copy, Edit, Eye, Plus, Send, Trash2 } from "lucide-react";

import { Page } from "../components/layout/Page";
import { useWorkspaceName } from "../hooks/useWorkspaceName";
import { Avatar } from "../components/ui/Avatar";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { ConfirmModal } from "../components/ui/ConfirmModal";
import {
  DropdownMenu,
  type DropdownMenuItem,
} from "../components/ui/DropdownMenu";
import { EmptyState } from "../components/ui/EmptyState";
import { Pills } from "../components/ui/Pills";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { Pagination } from "../components/common/Pagination";
import { StatusBadge } from "../components/invoice/StatusBadge";
import { PaymentStatusBadge } from "../components/invoice/PaymentStatusBadge";
import { MarkPaidModal } from "../components/invoice/MarkPaidModal";
import { useMoneyFormat } from "../lib/money";
import { useInvoiceStore } from "../stores/invoiceStore";
import type {
  InvoiceDto,
  InvoicePaymentFilterDto,
  InvoiceStatusDto,
} from "../ipc";

type FilterValue = "all" | InvoiceStatusDto;
type PaymentFilterValue = "all" | InvoicePaymentFilterDto;

export function InvoiceList() {
  const { t } = useTranslation();
  const workspaceName = useWorkspaceName();
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
  const { format } = useMoneyFormat();
  const [payFor, setPayFor] = useState<InvoiceDto | null>(null);
  const [cancelTarget, setCancelTarget] = useState<InvoiceDto | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);
  const filterValue: FilterValue = query.status ?? "all";
  const paymentFilterValue: PaymentFilterValue = query.payment_filter ?? "all";

  return (
    <Page
      crumbs={[workspaceName, t("invoices.title")]}
      title={t("invoices.title")}
      subtitle={
        page
          ? `${t("invoices.summary_total", { count: page.total })} · ${t("invoices.summary_displayed", { count: invoices.length })}`
          : undefined
      }
      actions={
        <Button
          variant="primary"
          leadingIcon={<Plus size={13} strokeWidth={1.5} />}
          onClick={() => navigate("/invoices/create")}
        >
          {t("invoices.new")}
        </Button>
      }
    >
      <div className="mb-3.5 flex flex-wrap items-center justify-between gap-3">
        <div className="flex flex-wrap items-center gap-3">
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
          <Pills<PaymentFilterValue>
            value={paymentFilterValue}
            onChange={(id) =>
              setQuery({
                ...query,
                payment_filter: id === "all" ? null : id,
              })
            }
            options={[
              { id: "all", label: t("invoices.all") },
              { id: "Paid", label: t("invoices.payment_filter_paid") },
              { id: "Unpaid", label: t("invoices.payment_filter_unpaid") },
              { id: "Late", label: t("invoices.payment_filter_late") },
            ]}
          />
        </div>
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
                const name = inv.client_name ?? "—";
                const cancellable =
                  inv.status === "Finalized" || inv.status === "Sent";
                const sendable =
                  inv.status === "Finalized" || inv.status === "Sent";
                const unpaid =
                  (inv.status === "Finalized" || inv.status === "Sent") &&
                  inv.payment_status !== "Paid";

                // Build the overflow menu: every action valid in this
                // state that isn't the row's primary. Open-in-editor
                // first (mirrors the row click), then secondary workflow
                // actions, with destructive last.
                const editable = inv.status === "Draft";
                const menuItems: DropdownMenuItem[] = [
                  {
                    id: "open",
                    label: editable ? t("common.edit") : t("common.view"),
                    icon: editable ? (
                      <Edit size={13} strokeWidth={1.5} />
                    ) : (
                      <Eye size={13} strokeWidth={1.5} />
                    ),
                    onSelect: () => navigate(`/invoices/${inv.id}/edit`),
                  },
                ];
                if (sendable && unpaid && inv.email_sends.length > 0) {
                  menuItems.push({
                    id: "remind",
                    label: t("invoices.send_reminder"),
                    icon: <Send size={13} strokeWidth={1.5} />,
                    onSelect: () =>
                      void send(inv.id).catch((e) => toast.error(String(e))),
                  });
                }
                menuItems.push({
                  id: "duplicate",
                  label: t("invoices.duplicate"),
                  icon: <Copy size={13} strokeWidth={1.5} />,
                  onSelect: () => void duplicate(inv.id),
                });
                if (cancellable) {
                  menuItems.push({
                    id: "cancel",
                    label: t("invoices.cancel"),
                    icon: <Trash2 size={13} strokeWidth={1.5} />,
                    tone: "danger",
                    onSelect: () => setCancelTarget(inv),
                  });
                }

                return (
                  <Tr
                    key={inv.id}
                    className="cursor-pointer"
                    onClick={() => navigate(`/invoices/${inv.id}/edit`)}
                  >
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
                    <Td
                      className="text-right whitespace-nowrap"
                      onClick={(e) => e.stopPropagation()}
                    >
                      <div className="flex justify-end gap-1">
                        {/* Slot 1: state-aware primary action */}
                        {inv.status === "Draft" ? (
                          <Button
                            size="sm"
                            variant="primary"
                            onClick={() =>
                              void finalize(inv.id).catch((e) => toast.error(String(e)))
                            }
                          >
                            {t("invoices.finalize")}
                          </Button>
                        ) : null}
                        {inv.status === "Finalized" ? (
                          <>
                            <Button
                              size="sm"
                              variant="accent"
                              leadingIcon={<Send size={11} strokeWidth={1.5} />}
                              onClick={() =>
                                void send(inv.id).catch((e) => toast.error(String(e)))
                              }
                            >
                              {t("invoices.send")}
                            </Button>
                            {unpaid ? (
                              <Button
                                size="sm"
                                leadingIcon={<Coins size={11} strokeWidth={1.5} />}
                                onClick={() => setPayFor(inv)}
                              >
                                {t("invoices.mark_paid")}
                              </Button>
                            ) : null}
                          </>
                        ) : null}
                        {inv.status === "Sent" && unpaid ? (
                          <Button
                            size="sm"
                            variant="primary"
                            leadingIcon={<Coins size={11} strokeWidth={1.5} />}
                            onClick={() => setPayFor(inv)}
                          >
                            {t("invoices.mark_paid")}
                          </Button>
                        ) : null}

                        {/* Slot 2: overflow menu */}
                        <DropdownMenu
                          triggerLabel={t("invoices.more_actions")}
                          items={menuItems}
                        />
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

      <ConfirmModal
        open={cancelTarget !== null}
        title={t("invoices.cancel")}
        description={t("invoices.confirm_cancel")}
        confirmLabel={t("invoices.cancel")}
        tone="danger"
        onConfirm={async () => {
          if (cancelTarget) await cancel(cancelTarget.id);
        }}
        onClose={() => setCancelTarget(null)}
      />
    </Page>
  );
}
