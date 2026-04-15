import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";

import {
  ipc,
  type AgingBucketDto,
  type AgingRowDto,
  type ClientBalanceDto,
  type RevenueBucketDto,
  type RevenueByClientDto,
  type RevenueGroupingDto,
} from "../ipc";
import { useSettingsStore } from "../stores/settingsStore";

type Tab = "revenue" | "aging" | "balances";

const THIS_YEAR_START = `${new Date().getFullYear()}-01-01`;
const THIS_YEAR_END = `${new Date().getFullYear()}-12-31`;

export function Accounting() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("revenue");

  return (
    <div className="max-w-6xl">
      <h1 className="mb-4 text-2xl font-bold text-fg">
        {t("accounting.title")}
      </h1>

      <div className="mb-4 flex gap-2">
        <TabButton active={tab === "revenue"} onClick={() => setTab("revenue")}>
          {t("accounting.tab_revenue")}
        </TabButton>
        <TabButton active={tab === "aging"} onClick={() => setTab("aging")}>
          {t("accounting.tab_aging")}
        </TabButton>
        <TabButton
          active={tab === "balances"}
          onClick={() => setTab("balances")}
        >
          {t("accounting.tab_balances")}
        </TabButton>
      </div>

      {tab === "revenue" ? <RevenueTab /> : null}
      {tab === "aging" ? <AgingTab /> : null}
      {tab === "balances" ? <BalancesTab /> : null}
    </div>
  );
}

function TabButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: React.ReactNode;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "rounded-pill px-4 py-1.5 text-sm font-medium transition-colors",
        active
          ? "bg-brand text-brand-fg"
          : "bg-surface-muted text-fg-muted hover:bg-border",
      ].join(" ")}
    >
      {children}
    </button>
  );
}

function RevenueTab() {
  const { t } = useTranslation();
  const { snapshot } = useSettingsStore();
  const [start, setStart] = useState(THIS_YEAR_START);
  const [end, setEnd] = useState(THIS_YEAR_END);
  const [grouping, setGrouping] = useState<RevenueGroupingDto>("Month");
  const [buckets, setBuckets] = useState<RevenueBucketDto[]>([]);
  const [byClient, setByClient] = useState<RevenueByClientDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  const symbol = snapshot?.currency.symbol ?? "€";
  const fmt = (cents: number) => `${(cents / 100).toFixed(2)} ${symbol}`;

  useEffect(() => {
    let cancelled = false;
    Promise.all([
      ipc.accountingRevenueByPeriod({ start, end, grouping }),
      ipc.accountingRevenueByClient({ start, end }),
    ])
      .then(([b, c]) => {
        if (cancelled) return;
        setBuckets(b);
        setByClient(c);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [start, end, grouping]);

  const total = buckets.reduce((sum, b) => sum + b.amount.amount_cents, 0);
  const maxBucket = Math.max(1, ...buckets.map((b) => b.amount.amount_cents));

  return (
    <div className="flex flex-col gap-4">
      <div className="flex flex-wrap items-end gap-3 rounded-card border border-border bg-surface p-4">
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("accounting.period_start")}
          <input
            type="date"
            value={start}
            onChange={(e) => setStart(e.target.value)}
            className="rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("accounting.period_end")}
          <input
            type="date"
            value={end}
            onChange={(e) => setEnd(e.target.value)}
            className="rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
          />
        </label>
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("accounting.grouping")}
          <select
            value={grouping}
            onChange={(e) => setGrouping(e.target.value as RevenueGroupingDto)}
            className="rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
          >
            <option value="Day">{t("accounting.grouping_day")}</option>
            <option value="Month">{t("accounting.grouping_month")}</option>
            <option value="Year">{t("accounting.grouping_year")}</option>
          </select>
        </label>
        <div className="ml-auto text-right">
          <p className="text-xs uppercase tracking-wide text-fg-subtle">
            {t("accounting.total")}
          </p>
          <p className="text-xl font-bold text-fg">{fmt(total)}</p>
        </div>
      </div>

      {error ? <p className="text-sm text-danger">{error}</p> : null}

      <section className="rounded-card border border-border bg-surface p-5 shadow-card">
        <h2 className="mb-3 text-sm font-semibold text-fg">
          {t("accounting.revenue_by_period")}
        </h2>
        {buckets.length === 0 ? (
          <p className="text-sm text-fg-muted">{t("common.empty")}</p>
        ) : (
          <div className="flex flex-col gap-2">
            {buckets.map((bucket) => {
              const pct = (bucket.amount.amount_cents / maxBucket) * 100;
              return (
                <div
                  key={bucket.bucket_start}
                  className="grid grid-cols-12 items-center gap-2"
                >
                  <span className="col-span-2 text-xs text-fg-muted">
                    {bucket.bucket_start}
                  </span>
                  <div className="col-span-8 h-6 overflow-hidden rounded-field bg-surface-muted">
                    <div
                      className="h-full rounded-field bg-brand"
                      style={{ width: `${pct}%` }}
                    />
                  </div>
                  <span className="col-span-2 text-right text-sm font-medium text-fg">
                    {fmt(bucket.amount.amount_cents)}
                  </span>
                </div>
              );
            })}
          </div>
        )}
      </section>

      <section className="rounded-card border border-border bg-surface p-5 shadow-card">
        <h2 className="mb-3 text-sm font-semibold text-fg">
          {t("accounting.revenue_by_client")}
        </h2>
        {byClient.length === 0 ? (
          <p className="text-sm text-fg-muted">{t("common.empty")}</p>
        ) : (
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="border-b border-border text-left text-fg-muted">
                <th className="py-2 pr-3 font-medium">
                  {t("invoices.client")}
                </th>
                <th className="py-2 pr-3 font-medium">
                  {t("accounting.invoice_count")}
                </th>
                <th className="py-2 pr-3 text-right font-medium">
                  {t("accounting.total")}
                </th>
              </tr>
            </thead>
            <tbody>
              {byClient.map((row) => (
                <tr key={row.client_id} className="border-b border-border">
                  <td className="py-2 pr-3 font-medium text-fg">
                    {row.client_name}
                  </td>
                  <td className="py-2 pr-3 text-fg-muted">
                    {row.invoice_count}
                  </td>
                  <td className="py-2 pr-3 text-right text-fg">
                    {fmt(row.total_invoiced.amount_cents)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}

function AgingTab() {
  const { t } = useTranslation();
  const { snapshot } = useSettingsStore();
  const [rows, setRows] = useState<AgingRowDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const symbol = snapshot?.currency.symbol ?? "€";
  const fmt = (cents: number) => `${(cents / 100).toFixed(2)} ${symbol}`;

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingAgingReport()
      .then((r) => !cancelled && setRows(r))
      .catch((e) => !cancelled && setError(String(e)));
    return () => {
      cancelled = true;
    };
  }, []);

  const grouped = useMemo(() => {
    const buckets: Record<AgingBucketDto, AgingRowDto[]> = {
      Current: [],
      Days1To30: [],
      Days31To60: [],
      Days61To90: [],
      Days91Plus: [],
    };
    for (const row of rows) buckets[row.bucket].push(row);
    return buckets;
  }, [rows]);

  const totals = useMemo(() => {
    const t: Record<AgingBucketDto, number> = {
      Current: 0,
      Days1To30: 0,
      Days31To60: 0,
      Days61To90: 0,
      Days91Plus: 0,
    };
    for (const row of rows) t[row.bucket] += row.amount_due.amount_cents;
    return t;
  }, [rows]);

  const BUCKETS: AgingBucketDto[] = [
    "Current",
    "Days1To30",
    "Days31To60",
    "Days61To90",
    "Days91Plus",
  ];

  return (
    <div className="flex flex-col gap-4">
      {error ? <p className="text-sm text-danger">{error}</p> : null}

      <div className="grid grid-cols-2 gap-3 lg:grid-cols-5">
        {BUCKETS.map((b) => (
          <div
            key={b}
            className="rounded-card border border-border bg-surface p-4 shadow-card"
          >
            <p className="text-xs uppercase tracking-wide text-fg-subtle">
              {t(`accounting.bucket_${b.toLowerCase()}`)}
            </p>
            <p className="mt-2 text-lg font-bold text-fg">{fmt(totals[b])}</p>
            <p className="text-xs text-fg-muted">
              {grouped[b].length} {t("accounting.invoices")}
            </p>
          </div>
        ))}
      </div>

      <section className="rounded-card border border-border bg-surface p-5 shadow-card">
        <h2 className="mb-3 text-sm font-semibold text-fg">
          {t("accounting.aging_detail")}
        </h2>
        {rows.length === 0 ? (
          <p className="text-sm text-fg-muted">{t("common.empty")}</p>
        ) : (
          <table className="w-full border-collapse text-sm">
            <thead>
              <tr className="border-b border-border text-left text-fg-muted">
                <th className="py-2 pr-3 font-medium">{t("invoices.number")}</th>
                <th className="py-2 pr-3 font-medium">{t("invoices.client")}</th>
                <th className="py-2 pr-3 font-medium">
                  {t("invoices.due_date")}
                </th>
                <th className="py-2 pr-3 font-medium">
                  {t("accounting.bucket")}
                </th>
                <th className="py-2 pr-3 text-right font-medium">
                  {t("accounting.amount_due")}
                </th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.invoice_id} className="border-b border-border">
                  <td className="py-2 pr-3 font-medium text-fg">
                    {row.number ?? "—"}
                  </td>
                  <td className="py-2 pr-3 text-fg-muted">{row.client_name}</td>
                  <td className="py-2 pr-3 text-fg-muted">
                    {row.due_date ?? "—"}
                  </td>
                  <td className="py-2 pr-3 text-fg-muted">
                    {t(`accounting.bucket_${row.bucket.toLowerCase()}`)}
                  </td>
                  <td className="py-2 pr-3 text-right font-medium text-fg">
                    {fmt(row.amount_due.amount_cents)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
    </div>
  );
}

function BalancesTab() {
  const { t } = useTranslation();
  const { snapshot } = useSettingsStore();
  const [rows, setRows] = useState<ClientBalanceDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const symbol = snapshot?.currency.symbol ?? "€";
  const fmt = (cents: number) => `${(cents / 100).toFixed(2)} ${symbol}`;

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingClientBalances()
      .then((r) => !cancelled && setRows(r))
      .catch((e) => !cancelled && setError(String(e)));
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <section className="rounded-card border border-border bg-surface p-5 shadow-card">
      {error ? <p className="mb-3 text-sm text-danger">{error}</p> : null}
      {rows.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("common.empty")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("invoices.client")}</th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("accounting.total_invoiced")}
              </th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("accounting.total_paid")}
              </th>
              <th className="py-2 pr-3 text-right font-medium">
                {t("accounting.outstanding")}
              </th>
            </tr>
          </thead>
          <tbody>
            {rows.map((row) => (
              <tr key={row.client_id} className="border-b border-border">
                <td className="py-2 pr-3 font-medium text-fg">
                  {row.client_name}
                </td>
                <td className="py-2 pr-3 text-right text-fg-muted">
                  {fmt(row.total_invoiced.amount_cents)}
                </td>
                <td className="py-2 pr-3 text-right text-fg-muted">
                  {fmt(row.total_paid.amount_cents)}
                </td>
                <td
                  className={[
                    "py-2 pr-3 text-right font-semibold",
                    row.outstanding.amount_cents > 0
                      ? "text-warning"
                      : "text-fg",
                  ].join(" ")}
                >
                  {fmt(row.outstanding.amount_cents)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
    </section>
  );
}
