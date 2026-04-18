import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { ipc, type DashboardSummaryDto, type InvoicePaymentRowDto } from "../ipc";
import { useMoneyFormat } from "../lib/money";
import { useSettingsStore } from "../stores/settingsStore";

export function Dashboard() {
  const { t } = useTranslation();
  const { snapshot, load } = useSettingsStore();
  const [summary, setSummary] = useState<DashboardSummaryDto | null>(null);
  const [overdue, setOverdue] = useState<InvoicePaymentRowDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!snapshot) void load();
    let cancelled = false;
    Promise.all([ipc.accountingDashboardSummary(), ipc.accountingListOverdue()])
      .then(([s, o]) => {
        if (cancelled) return;
        setSummary(s);
        setOverdue(o);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [snapshot, load]);

  const { format } = useMoneyFormat();

  return (
    <div className="max-w-6xl">
      <h1 className="mb-6 text-2xl font-bold text-fg">{t("dashboard.title")}</h1>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}

      {summary ? (
        <>
          <div className="mb-6 grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Card
              label={t("dashboard.revenue_this_year")}
              value={format(summary.revenue_this_year)}
              accent="brand"
            />
            <Card
              label={t("dashboard.outstanding")}
              value={format(summary.outstanding_total)}
              accent="warning"
            />
            <Card
              label={t("dashboard.overdue_count")}
              value={String(summary.overdue_count)}
              accent="danger"
            />
            <Card
              label={t("dashboard.draft_count")}
              value={String(summary.draft_count)}
              accent="muted"
            />
          </div>

          <section className="rounded-card border border-border bg-surface p-5 shadow-card">
            <h2 className="mb-3 text-sm font-semibold text-fg">
              {t("dashboard.overdue_invoices")}
            </h2>
            {overdue.length === 0 ? (
              <p className="text-sm text-fg-muted">{t("dashboard.no_overdue")}</p>
            ) : (
              <table className="w-full border-collapse text-sm">
                <thead>
                  <tr className="border-b border-border text-left text-fg-muted">
                    <th className="py-2 pr-3 font-medium">
                      {t("invoices.number")}
                    </th>
                    <th className="py-2 pr-3 font-medium">
                      {t("invoices.client")}
                    </th>
                    <th className="py-2 pr-3 font-medium">
                      {t("invoices.due_date")}
                    </th>
                    <th className="py-2 pr-3 text-right font-medium">
                      {t("accounting.amount_due")}
                    </th>
                  </tr>
                </thead>
                <tbody>
                  {overdue.map((row) => (
                    <tr key={row.invoice_id} className="border-b border-border">
                      <td className="py-2 pr-3 font-medium text-fg">
                        {row.number ?? "—"}
                      </td>
                      <td className="py-2 pr-3 text-fg-muted">
                        {row.client_name}
                      </td>
                      <td className="py-2 pr-3 text-fg-muted">
                        {row.due_date ?? "—"}
                      </td>
                      <td className="py-2 pr-3 text-right text-fg">
                        {format(row.amount_due)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </section>
        </>
      ) : (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      )}
    </div>
  );
}

interface CardProps {
  label: string;
  value: string;
  accent: "brand" | "warning" | "danger" | "muted";
}

function Card({ label, value, accent }: CardProps) {
  const accentClass =
    accent === "brand"
      ? "text-brand"
      : accent === "warning"
        ? "text-warning"
        : accent === "danger"
          ? "text-danger"
          : "text-fg-muted";
  return (
    <div className="rounded-card border border-border bg-surface p-4 shadow-card">
      <p className="text-xs font-medium uppercase tracking-wide text-fg-subtle">
        {label}
      </p>
      <p className={`mt-2 text-2xl font-bold ${accentClass}`}>{value}</p>
    </div>
  );
}
