import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { Download } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Field, Input, Select } from "../components/ui/Input";
import { Pills } from "../components/ui/Pills";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import {
  ipc,
  type AgingBucketDto,
  type AgingRowDto,
  type ClientBalanceDto,
  type RevenueBucketDto,
  type RevenueByClientDto,
  type RevenueGroupingDto,
} from "../ipc";
import { useMoneyFormat } from "../lib/money";

type Tab = "revenue" | "aging" | "balances";

const THIS_YEAR_START = `${new Date().getFullYear()}-01-01`;
const THIS_YEAR_END = `${new Date().getFullYear()}-12-31`;

export function Accounting() {
  const { t } = useTranslation();
  const [tab, setTab] = useState<Tab>("revenue");

  return (
    <Page
      crumbs={[t("accounting.title")]}
      title={t("accounting.title")}
      subtitle={t("accounting.period_range", {
        start: THIS_YEAR_START,
        end: THIS_YEAR_END,
      })}
      actions={
        <Button leadingIcon={<Download size={13} strokeWidth={1.5} />}>
          {t("common.pdf")}
        </Button>
      }
    >
      <div className="mb-5 flex flex-wrap items-center justify-between gap-3">
        <Pills<Tab>
          value={tab}
          onChange={setTab}
          options={[
            { id: "revenue", label: t("accounting.tab_revenue") },
            { id: "aging", label: t("accounting.tab_aging") },
            { id: "balances", label: t("accounting.tab_balances") },
          ]}
        />
      </div>

      {tab === "revenue" ? <RevenueTab /> : null}
      {tab === "aging" ? <AgingTab /> : null}
      {tab === "balances" ? <BalancesTab /> : null}
    </Page>
  );
}

function RevenueTab() {
  const { t } = useTranslation();
  const [start, setStart] = useState(THIS_YEAR_START);
  const [end, setEnd] = useState(THIS_YEAR_END);
  const [grouping, setGrouping] = useState<RevenueGroupingDto>("Month");
  const [buckets, setBuckets] = useState<RevenueBucketDto[]>([]);
  const [byClient, setByClient] = useState<RevenueByClientDto[]>([]);

  const { formatAmount, formatMinor } = useMoneyFormat();

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
        if (!cancelled) toast.error(e);
      });
    return () => {
      cancelled = true;
    };
  }, [start, end, grouping]);

  // Strict silos: every (currency) gets its own chart and table. No
  // cross-currency total exists.
  const bucketsByCurrency = useMemo(
    () => groupBy(buckets, (b) => b.amount.currency.code),
    [buckets],
  );
  const byClientByCurrency = useMemo(
    () => groupBy(byClient, (r) => r.total_invoiced.currency.code),
    [byClient],
  );
  const currencies = useMemo(() => {
    const set = new Set<string>();
    for (const b of buckets) set.add(b.amount.currency.code);
    for (const r of byClient) set.add(r.total_invoiced.currency.code);
    return Array.from(set).sort();
  }, [buckets, byClient]);

  return (
    <div className="flex flex-col">
      <Card className="mb-5">
        <CardBody className="flex flex-wrap items-end gap-4">
          <Field label={t("accounting.period_start")}>
            <Input
              mono
              type="date"
              value={start}
              onChange={(e) => setStart(e.target.value)}
            />
          </Field>
          <Field label={t("accounting.period_end")}>
            <Input
              mono
              type="date"
              value={end}
              onChange={(e) => setEnd(e.target.value)}
            />
          </Field>
          <Field label={t("accounting.grouping")}>
            <Select
              value={grouping}
              onChange={(e) => setGrouping(e.target.value as RevenueGroupingDto)}
            >
              <option value="Day">{t("accounting.grouping_day")}</option>
              <option value="Month">{t("accounting.grouping_month")}</option>
              <option value="Year">{t("accounting.grouping_year")}</option>
            </Select>
          </Field>
        </CardBody>
      </Card>

      {currencies.length === 0 ? (
        <Card>
          <CardBody>
            <EmptyState description={t("common.empty")} />
          </CardBody>
        </Card>
      ) : (
        <div className="flex flex-col gap-5">
          {currencies.map((code) => {
            const buckets = bucketsByCurrency.get(code) ?? [];
            const byClient = byClientByCurrency.get(code) ?? [];
            const total = buckets.reduce((s, b) => s + b.amount.amount, 0);
            const maxBucket = Math.max(1, ...buckets.map((b) => b.amount.amount));
            const totalAll = byClient.reduce(
              (s, r) => s + r.total_invoiced.amount,
              0,
            );
            return (
              <div key={code} className="flex flex-col gap-3">
                <h3 className="text-[18px] font-semibold font-mono tabular leading-none">
                  {formatMinor(total, code)}
                </h3>
                <div className="grid grid-cols-1 lg:grid-cols-[1.4fr_1fr] gap-5">
                  <Card>
                    <CardHead title={t("accounting.revenue_by_period")} />
                    <CardBody>
                      {buckets.length === 0 ? (
                        <EmptyState description={t("common.empty")} />
                      ) : (
                        <div className="flex flex-col gap-2.5">
                          {buckets.map((bucket) => {
                            const pct =
                              (bucket.amount.amount / maxBucket) * 100;
                            return (
                              <div
                                key={bucket.bucket_start}
                                className="grid items-center gap-3.5"
                                style={{ gridTemplateColumns: "110px 1fr 110px" }}
                              >
                                <span className="text-[12px] text-ink-3 font-mono tabular">
                                  {bucket.bucket_start}
                                </span>
                                <div className="h-[18px] bg-paper-3 relative">
                                  <div
                                    className="absolute inset-y-0 left-0 bg-accent"
                                    style={{ width: `${pct}%` }}
                                  />
                                </div>
                                <span className="text-right font-mono tabular text-[13px]">
                                  {formatAmount(bucket.amount)}
                                </span>
                              </div>
                            );
                          })}
                        </div>
                      )}
                    </CardBody>
                  </Card>

                  <Card>
                    <CardHead title={t("accounting.revenue_by_client")} />
                    {byClient.length === 0 ? (
                      <EmptyState description={t("common.empty")} />
                    ) : (
                      <Table>
                        <THead>
                          <Tr>
                            <Th>{t("invoices.client")}</Th>
                            <Th numeric>{t("accounting.invoice_count")}</Th>
                            <Th numeric>{t("accounting.total")}</Th>
                            <Th numeric className="w-16">
                              %
                            </Th>
                          </Tr>
                        </THead>
                        <tbody>
                          {byClient.map((row) => {
                            const pct =
                              totalAll > 0
                                ? (row.total_invoiced.amount / totalAll) * 100
                                : 0;
                            return (
                              <Tr key={row.client_id}>
                                <Td>
                                  <div className="flex items-center gap-2">
                                    <Avatar name={row.client_name} size={22} />
                                    <span className="font-medium">
                                      {row.client_name}
                                    </span>
                                  </div>
                                </Td>
                                <Td numeric muted>
                                  {row.invoice_count}
                                </Td>
                                <Td numeric>{formatAmount(row.total_invoiced)}</Td>
                                <Td numeric className="text-[11px]">
                                  {pct.toFixed(1)} %
                                </Td>
                              </Tr>
                            );
                          })}
                        </tbody>
                      </Table>
                    )}
                  </Card>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

function groupBy<T, K>(items: T[], key: (item: T) => K): Map<K, T[]> {
  const out = new Map<K, T[]>();
  for (const item of items) {
    const k = key(item);
    const arr = out.get(k);
    if (arr) arr.push(item);
    else out.set(k, [item]);
  }
  return out;
}

function AgingTab() {
  const { t } = useTranslation();
  const [rows, setRows] = useState<AgingRowDto[]>([]);
  const { formatAmount, formatMinor } = useMoneyFormat();

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingAgingReport()
      .then((r) => !cancelled && setRows(r))
      .catch((e) => !cancelled && toast.error(e));
    return () => {
      cancelled = true;
    };
  }, []);

  const byCurrency = useMemo(
    () => groupBy(rows, (r) => r.amount_due.currency.code),
    [rows],
  );
  const currencies = useMemo(
    () => Array.from(byCurrency.keys()).sort(),
    [byCurrency],
  );

  const BUCKETS: { id: AgingBucketDto; tone: string }[] = [
    { id: "Current", tone: "var(--color-ok)" },
    { id: "Days1To30", tone: "var(--color-ink-2)" },
    { id: "Days31To60", tone: "var(--color-ink-3)" },
    { id: "Days61To90", tone: "var(--color-warn)" },
    { id: "Days91Plus", tone: "var(--color-danger)" },
  ];

  if (rows.length === 0) {
    return (
      <Card>
        <CardBody>
          <EmptyState description={t("common.empty")} />
        </CardBody>
      </Card>
    );
  }

  return (
    <div className="flex flex-col gap-6">
      {currencies.map((code) => {
        const rowsForCurrency = byCurrency.get(code) ?? [];
        const grouped: Record<AgingBucketDto, AgingRowDto[]> = {
          Current: [],
          Days1To30: [],
          Days31To60: [],
          Days61To90: [],
          Days91Plus: [],
        };
        const totals: Record<AgingBucketDto, number> = {
          Current: 0,
          Days1To30: 0,
          Days31To60: 0,
          Days61To90: 0,
          Days91Plus: 0,
        };
        for (const row of rowsForCurrency) {
          grouped[row.bucket].push(row);
          totals[row.bucket] += row.amount_due.amount;
        }
        return (
          <div key={code} className="flex flex-col gap-3">
            <h3 className="text-[14px] font-semibold tabular tracking-tight">
              {code}
            </h3>
            <div className="grid grid-cols-2 gap-3.5 lg:grid-cols-5">
              {BUCKETS.map((b) => (
                <Card key={b.id} className="p-4 flex flex-col gap-2">
                  <div className="flex items-center gap-2.5">
                    <span className="w-1.5 h-6" style={{ background: b.tone }} />
                    <div>
                      <p className="text-[12px] font-medium text-ink-3">
                        {t(`accounting.bucket_${b.id.toLowerCase()}`)}
                      </p>
                      <p className="text-[18px] font-semibold font-mono tabular leading-tight mt-0.5">
                        {formatMinor(totals[b.id], code)}
                      </p>
                      <p className="text-[11px] text-ink-3">
                        {grouped[b.id].length} {t("accounting.invoices")}
                      </p>
                    </div>
                  </div>
                </Card>
              ))}
            </div>
            <Card>
              <CardHead title={t("accounting.aging_detail")} />
              <Table>
                <THead>
                  <Tr>
                    <Th>{t("invoices.number")}</Th>
                    <Th>{t("invoices.client")}</Th>
                    <Th>{t("invoices.due_date")}</Th>
                    <Th>{t("accounting.bucket")}</Th>
                    <Th numeric>{t("accounting.amount_due")}</Th>
                  </Tr>
                </THead>
                <tbody>
                  {rowsForCurrency.map((row) => (
                    <Tr key={row.invoice_id}>
                      <Td mono>#{row.number ?? "—"}</Td>
                      <Td>{row.client_name}</Td>
                      <Td muted mono>
                        {row.due_date ?? "—"}
                      </Td>
                      <Td muted>
                        {t(`accounting.bucket_${row.bucket.toLowerCase()}`)}
                      </Td>
                      <Td numeric>{formatAmount(row.amount_due)}</Td>
                    </Tr>
                  ))}
                </tbody>
              </Table>
            </Card>
          </div>
        );
      })}
    </div>
  );
}

function BalancesTab() {
  const { t } = useTranslation();
  const [rows, setRows] = useState<ClientBalanceDto[]>([]);
  const { formatAmount } = useMoneyFormat();

  useEffect(() => {
    let cancelled = false;
    ipc
      .accountingClientBalances()
      .then((r) => !cancelled && setRows(r))
      .catch((e) => !cancelled && toast.error(e));
    return () => {
      cancelled = true;
    };
  }, []);

  const byCurrency = useMemo(
    () => groupBy(rows, (r) => r.outstanding.currency.code),
    [rows],
  );
  const currencies = useMemo(
    () => Array.from(byCurrency.keys()).sort(),
    [byCurrency],
  );

  if (rows.length === 0) {
    return (
      <Card>
        <CardBody>
          <EmptyState description={t("common.empty")} />
        </CardBody>
      </Card>
    );
  }

  return (
    <div className="flex flex-col gap-5">
      {currencies.map((code) => {
        const list = byCurrency.get(code) ?? [];
        return (
          <div key={code} className="flex flex-col gap-3">
            <h3 className="text-[14px] font-semibold tabular tracking-tight">
              {code}
            </h3>
            <Card>
              <Table>
                <THead>
                  <Tr>
                    <Th>{t("invoices.client")}</Th>
                    <Th numeric>{t("accounting.total_invoiced")}</Th>
                    <Th numeric>{t("accounting.total_paid")}</Th>
                    <Th numeric>{t("accounting.outstanding")}</Th>
                  </Tr>
                </THead>
                <tbody>
                  {list.map((row) => (
                    <Tr key={`${row.client_id}-${row.outstanding.currency}`}>
                      <Td>
                        <div className="flex items-center gap-2">
                          <Avatar name={row.client_name} size={22} />
                          <span className="font-medium">{row.client_name}</span>
                        </div>
                      </Td>
                      <Td numeric muted>
                        {formatAmount(row.total_invoiced)}
                      </Td>
                      <Td numeric muted>
                        {formatAmount(row.total_paid)}
                      </Td>
                      <Td
                        numeric
                        className={
                          row.outstanding.amount > 0
                            ? "text-warn font-medium"
                            : "font-medium"
                        }
                      >
                        {formatAmount(row.outstanding)}
                      </Td>
                    </Tr>
                  ))}
                </tbody>
              </Table>
            </Card>
          </div>
        );
      })}
    </div>
  );
}
