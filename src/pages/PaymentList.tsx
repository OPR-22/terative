import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Calendar, Download, Edit, Plus, Search, Trash2 } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Input } from "../components/ui/Input";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { useMoneyFormat } from "../lib/money";
import { usePaymentStore } from "../stores/paymentStore";
import { useClientStore } from "../stores/clientStore";
import { useSettingsStore } from "../stores/settingsStore";
import type { PaymentMethodDto } from "../ipc";

function paymentMethodLabel(method: PaymentMethodDto, t: (k: string) => string): string {
  switch (method.kind) {
    case "BankTransfer":
      return t("payments.method_banktransfer");
    case "Cash":
      return t("payments.method_cash");
    case "Check":
      return t("payments.method_check");
    case "Card":
      return t("payments.method_card");
    case "Other":
      return method.detail || t("payments.method_other");
  }
}

export function PaymentList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { payments, loading, error, refresh, remove } = usePaymentStore();
  const ensureDirectory = useClientStore((s) => s.ensureDirectory);
  const clientName = useClientStore((s) => s.clientName);
  const { snapshot, load } = useSettingsStore();

  useEffect(() => {
    void refresh();
    void ensureDirectory();
    if (!snapshot) void load();
  }, [refresh, ensureDirectory, load, snapshot]);

  const { formatMinor } = useMoneyFormat();
  const currencyCode = snapshot?.currency.code ?? "EUR";

  const totalAmount = payments.reduce((sum, p) => sum + p.amount.amount_minor, 0);

  return (
    <Page
      crumbs={["Cabinet Lemaire", t("payments.title")]}
      title={t("payments.title")}
      subtitle={`${payments.length} paiements · ${formatMinor(totalAmount, currencyCode)} encaissés`}
      actions={
        <>
          <Button leadingIcon={<Download size={13} strokeWidth={1.5} />}>
            Exporter
          </Button>
          <Button
            variant="primary"
            leadingIcon={<Plus size={13} strokeWidth={1.5} />}
            onClick={() => navigate("/payments/create")}
          >
            {t("payments.new")}
          </Button>
        </>
      }
    >
      <div className="mb-3.5 flex flex-wrap items-center justify-between gap-3">
        <div className="relative max-w-md flex-1">
          <Search
            size={13}
            strokeWidth={1.5}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-3"
          />
          <Input className="pl-8" placeholder="Client, référence…" />
        </div>
        <Button leadingIcon={<Calendar size={13} strokeWidth={1.5} />}>
          Période
        </Button>
      </div>

      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      <Card className="overflow-hidden">
        {loading ? (
          <EmptyState description={t("common.loading")} />
        ) : payments.length === 0 ? (
          <EmptyState description={t("payments.none")} />
        ) : (
          <Table>
            <THead>
              <Tr>
                <Th>{t("common.date")}</Th>
                <Th>{t("payments.client")}</Th>
                <Th>{t("payments.method")}</Th>
                <Th>{t("payments.reference")}</Th>
                <Th numeric>{t("payments.amount")}</Th>
                <Th numeric>{t("payments.allocated")}</Th>
                <Th numeric>{t("payments.unallocated")}</Th>
                <Th />
              </Tr>
            </THead>
            <tbody>
              {payments.map((p) => {
                const allocated = p.allocations.reduce(
                  (sum, a) => sum + a.amount.amount_minor,
                  0,
                );
                const rest = p.amount.amount_minor - allocated;
                const name = clientName(p.client_id);
                return (
                  <Tr key={p.id}>
                    <Td muted mono>
                      {p.date}
                    </Td>
                    <Td>
                      <div className="flex items-center gap-2">
                        <Avatar name={name} size={22} />
                        <span>{name}</span>
                      </div>
                    </Td>
                    <Td>
                      <Badge kind="outline">{paymentMethodLabel(p.method, t)}</Badge>
                    </Td>
                    <Td muted mono>
                      {p.reference ?? "—"}
                    </Td>
                    <Td numeric>{formatMinor(p.amount.amount_minor, p.amount.currency)}</Td>
                    <Td numeric>{formatMinor(allocated, currencyCode)}</Td>
                    <Td numeric>
                      {rest > 0 ? (
                        <span className="text-warn">
                          {formatMinor(rest, currencyCode)}
                        </span>
                      ) : (
                        <span className="text-ink-4">—</span>
                      )}
                    </Td>
                    <Td className="text-right whitespace-nowrap">
                      <div className="flex justify-end gap-1">
                        <Button
                          size="sm"
                          iconOnly
                          aria-label={t("common.edit")}
                          onClick={() => navigate(`/payments/${p.id}/edit`)}
                        >
                          <Edit size={11} strokeWidth={1.5} />
                        </Button>
                        <Button
                          size="sm"
                          iconOnly
                          variant="danger"
                          aria-label={t("common.delete")}
                          onClick={() => {
                            if (confirm(t("common.confirm_delete"))) {
                              void remove(p.id);
                            }
                          }}
                        >
                          <Trash2 size={11} strokeWidth={1.5} />
                        </Button>
                      </div>
                    </Td>
                  </Tr>
                );
              })}
            </tbody>
          </Table>
        )}
      </Card>
    </Page>
  );
}
