import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Pagination } from "../components/common/Pagination";
import { StatusBadge } from "../components/invoice/StatusBadge";
import { PaymentStatusBadge } from "../components/invoice/PaymentStatusBadge";
import { MarkPaidModal } from "../components/invoice/MarkPaidModal";
import { useMoneyFormat } from "../lib/money";
import { InvoiceEditor } from "./InvoiceEditor";
import { useInvoiceStore } from "../stores/invoiceStore";
import { useClientStore } from "../stores/clientStore";
import type { InvoiceDto, InvoiceStatusDto } from "../ipc";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; invoice: InvoiceDto };

const STATUSES: InvoiceStatusDto[] = ["Draft", "Finalized", "Sent", "Cancelled"];

export function InvoiceList() {
  const { t } = useTranslation();
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
  const { clients, refresh: refreshClients } = useClientStore();
  const { format } = useMoneyFormat();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });
  const [payFor, setPayFor] = useState<InvoiceDto | null>(null);

  useEffect(() => {
    void refresh();
    void refreshClients();
  }, [refresh, refreshClients]);

  const clientName = (id: string) =>
    clients.find((c) => c.id === id)?.name ?? id;

  if (editor.mode !== "closed") {
    return (
      <InvoiceEditor
        invoice={editor.mode === "edit" ? editor.invoice : null}
        onClose={() => {
          setEditor({ mode: "closed" });
          void refresh();
        }}
      />
    );
  }

  return (
    <div className="max-w-6xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("invoices.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("invoices.new")}
        </Button>
      </div>

      <div className="mb-4 flex flex-wrap items-center gap-2">
        <button
          type="button"
          onClick={() => setQuery({ ...query, status: null })}
          className={filterClass(!query.status)}
        >
          {t("invoices.all")}
        </button>
        {STATUSES.map((s) => (
          <button
            key={s}
            type="button"
            onClick={() => setQuery({ ...query, status: s })}
            className={filterClass(query.status === s)}
          >
            {t(`invoices.status_${s.toLowerCase()}`)}
          </button>
        ))}
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : invoices.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("invoices.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("invoices.number")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.date")}</th>
              <th className="py-2 pr-3 font-medium">{t("invoices.client")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.status")}</th>
              <th className="py-2 pr-3 font-medium">
                {t("invoices.payment")}
              </th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("invoices.total")}
              </th>
              <th className="py-2 pr-3"></th>
            </tr>
          </thead>
          <tbody>
            {invoices.map((inv) => (
              <tr key={inv.id} className="border-b border-border">
                <td className="py-2 pr-3 font-medium text-fg">
                  {inv.number ?? "—"}
                </td>
                <td className="py-2 pr-3 text-fg-muted">{inv.date}</td>
                <td className="py-2 pr-3 text-fg-muted">
                  {clientName(inv.client_id)}
                </td>
                <td className="py-2 pr-3">
                  <StatusBadge status={inv.status} />
                </td>
                <td className="py-2 pr-3">
                  <PaymentStatusBadge
                    paymentStatus={inv.payment_status}
                    rawStatus={inv.status}
                  />
                </td>
                <td className="py-2 pr-3 text-right font-medium text-fg">
                  {format(inv.total)}
                </td>
                <td className="flex flex-wrap justify-end gap-2 py-2 pr-3">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", invoice: inv })}
                  >
                    {t(inv.status === "Draft" ? "common.edit" : "common.view")}
                  </Button>
                  {inv.status === "Draft" ? (
                    <Button
                      onClick={() =>
                        void finalize(inv.id).catch((e) => alert(String(e)))
                      }
                    >
                      {t("invoices.finalize")}
                    </Button>
                  ) : null}
                  {inv.status === "Finalized" || inv.status === "Sent" ? (
                    <Button
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
                    <Button
                      variant="secondary"
                      onClick={() => setPayFor(inv)}
                    >
                      {t("invoices.mark_paid")}
                    </Button>
                  ) : null}
                  <Button
                    variant="secondary"
                    onClick={() => void duplicate(inv.id)}
                  >
                    {t("invoices.duplicate")}
                  </Button>
                  {inv.status === "Finalized" || inv.status === "Sent" ? (
                    <Button
                      variant="danger"
                      onClick={() => {
                        if (confirm(t("invoices.confirm_cancel"))) {
                          void cancel(inv.id).catch((e) => alert(String(e)));
                        }
                      }}
                    >
                      {t("invoices.cancel")}
                    </Button>
                  ) : null}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
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

      {payFor ? (
        <MarkPaidModal
          invoice={payFor}
          onClose={() => setPayFor(null)}
          onPaid={() => void refresh()}
        />
      ) : null}
    </div>
  );
}

function filterClass(active: boolean) {
  return [
    "rounded-pill px-3 py-1 text-xs font-medium transition-colors",
    active
      ? "bg-brand text-brand-fg"
      : "bg-surface-muted text-fg-muted hover:bg-border",
  ].join(" ");
}

