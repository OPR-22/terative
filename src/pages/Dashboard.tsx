import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate } from "react-router-dom";
import {
  AlertCircle,
  ArrowRight,
  FileText,
  Plus,
  Send,
  User,
  Wallet,
} from "lucide-react";

import { Page, SectionTitle } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card, CardHead } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import {
  ipc,
  type DashboardOutstandingRowDto,
  type DashboardRevenueRowDto,
  type DashboardSummaryDto,
  type InvoicePaymentRowDto,
  type MoneyDto,
} from "../ipc";
import { useMoneyFormat } from "../lib/money";
import { useSettingsStore } from "../stores/settingsStore";

type FormatAmount = (dto: MoneyDto) => string;

function daysOverdue(dueDate: string | null): number | null {
  if (!dueDate) return null;
  const due = new Date(dueDate);
  const now = new Date();
  const diff = Math.round((now.getTime() - due.getTime()) / 86_400_000);
  return diff > 0 ? diff : null;
}

export function Dashboard() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { snapshot, load } = useSettingsStore();
  const [summary, setSummary] = useState<DashboardSummaryDto | null>(null);
  const [overdue, setOverdue] = useState<InvoicePaymentRowDto[]>([]);

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
        if (!cancelled) toast.error(e);
      });
    return () => {
      cancelled = true;
    };
  }, [snapshot, load]);

  const { formatAmount } = useMoneyFormat();
  const year = new Date().getFullYear();

  return (
    <Page
      title={t("dashboard.title")}
      subtitle={t("dashboard.subtitle")}
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
      {summary ? (
        <>
          <div
            className="grid grid-cols-1 gap-3.5 lg:grid-cols-[7fr_7fr_6fr]"
          >
            <RevenueCard
              summary={summary}
              year={year}
              formatAmount={formatAmount}
            />
            <OutstandingCard
              summary={summary}
              formatAmount={formatAmount}
              onFollowUp={() => navigate("/invoices")}
            />
            <ActivityCard
              summary={summary}
              year={year}
              onSeeMore={() => navigate("/invoices")}
            />
          </div>

          <SectionTitle>{t("dashboard.action_required")}</SectionTitle>

          <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1.6fr_1fr]">
            <Card>
              <CardHead
                title={t("dashboard.overdue_invoices")}
                subtitle={
                  overdue.length === 0
                    ? t("dashboard.no_overdue")
                    : t("dashboard.overdue_to_follow_up_count", {
                        count: overdue.length,
                      })
                }
                actions={
                  <Button size="sm" onClick={() => navigate("/invoices")}>
                    {t("common.see_all")}
                  </Button>
                }
              />
              {overdue.length === 0 ? (
                <EmptyState description={t("dashboard.no_overdue")} />
              ) : (
                <Table>
                  <THead>
                    <Tr>
                      <Th>N°</Th>
                      <Th>{t("invoices.client")}</Th>
                      <Th>{t("invoices.due_date")}</Th>
                      <Th>{t("dashboard.overdue_days_label")}</Th>
                      <Th>{t("accounting.currency")}</Th>
                      <Th numeric>{t("accounting.amount_due")}</Th>
                      <Th />
                    </Tr>
                  </THead>
                  <tbody>
                    {overdue.slice(0, 6).map((row) => {
                      const days = daysOverdue(row.due_date);
                      const name = row.client_name;
                      return (
                        <Tr key={row.invoice_id}>
                          <Td muted mono>
                            #{row.number ?? "—"}
                          </Td>
                          <Td>
                            <div className="flex items-center gap-2">
                              <Avatar name={name} size={22} />
                              <span>{name}</span>
                            </div>
                          </Td>
                          <Td muted>{row.due_date ?? "—"}</Td>
                          <Td>
                            {days != null ? (
                              <Badge dot kind="overdue">
                                {days}&nbsp;j
                              </Badge>
                            ) : (
                              "—"
                            )}
                          </Td>
                          <Td muted mono>{row.amount_due.currency.code}</Td>
                          <Td numeric>{formatAmount(row.amount_due)}</Td>
                          <Td className="text-right">
                            <Button
                              size="sm"
                              leadingIcon={<Send size={11} strokeWidth={1.5} />}
                              onClick={() =>
                                navigate(`/invoices/${row.invoice_id}/edit`)
                              }
                            >
                              {t("dashboard.follow_up_action")}
                            </Button>
                          </Td>
                        </Tr>
                      );
                    })}
                  </tbody>
                </Table>
              )}
            </Card>

            <Card>
              <CardHead title={t("dashboard.recent_activity")} />
              <div className="py-1.5">
                {[
                  {
                    Ic: Wallet,
                    title: t("dashboard.activity_payment_received"),
                    detail: t("dashboard.activity_payment_received_detail"),
                    when: t("dashboard.activity_when_recent_minutes"),
                  },
                  {
                    Ic: Send,
                    title: t("dashboard.activity_invoice_sent"),
                    detail: t("dashboard.activity_invoice_sent_detail"),
                    when: t("dashboard.activity_when_today"),
                  },
                  {
                    Ic: FileText,
                    title: t("dashboard.activity_draft_modified"),
                    detail: t("dashboard.activity_draft_modified_detail"),
                    when: t("dashboard.activity_when_recently"),
                  },
                  {
                    Ic: User,
                    title: t("dashboard.activity_new_client"),
                    detail: t("dashboard.activity_new_client_detail"),
                    when: t("dashboard.activity_when_recently"),
                  },
                ].map((a, i, arr) => (
                  <div
                    key={i}
                    className={[
                      "flex items-start gap-3 px-5 py-2.5",
                      i < arr.length - 1 ? "border-b border-line-soft" : "",
                    ].join(" ")}
                  >
                    <span className="grid place-items-center w-6 h-6 rounded-sm bg-paper-2 text-ink-2 shrink-0">
                      <a.Ic size={13} strokeWidth={1.5} />
                    </span>
                    <div className="min-w-0 flex-1">
                      <div className="text-[12.5px] font-medium text-ink">{a.title}</div>
                      <div className="text-[11px] text-ink-3 mt-0.5 truncate">
                        {a.detail}
                      </div>
                    </div>
                    <div className="text-[11px] text-ink-3 whitespace-nowrap">
                      {a.when}
                    </div>
                  </div>
                ))}
              </div>
            </Card>
          </div>
        </>
      ) : (
        <EmptyState description={t("common.loading")} />
      )}
    </Page>
  );
}

// ─── Card 1: Revenue this year ───────────────────────────────────────────

function RevenueCard({
  summary,
  year,
  formatAmount,
}: {
  summary: DashboardSummaryDto;
  year: number;
  formatAmount: FormatAmount;
}) {
  const { t } = useTranslation();
  const totalInvoices = summary.revenue_this_year.reduce(
    (s, r) => s + r.invoice_count,
    0,
  );
  const totalCurrencies = summary.revenue_this_year.length;
  return (
    <DashboardCard
      title={t("dashboard.card_revenue_title")}
      meta={year}
      footer={
        summary.revenue_this_year.length === 0 ? null : (
          <span className="text-ink-3">
            {t("dashboard.card_revenue_footer_invoices", {
              count: totalInvoices,
            })}{" "}
            ·{" "}
            {t("dashboard.card_revenue_footer_currencies", {
              count: totalCurrencies,
            })}
          </span>
        )
      }
    >
      {summary.revenue_this_year.length === 0 ? (
        <EmptyRows label={t("dashboard.no_revenue_yet")} />
      ) : (
        <div className="max-h-48 overflow-y-auto overflow-x-hidden">
          <table className="w-full border-collapse">
            <tbody>
              {summary.revenue_this_year.map((r) => (
                <RevenueRow
                  key={r.amount.currency.code}
                  row={r}
                  formatAmount={formatAmount}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </DashboardCard>
  );
}

function RevenueRow({
  row,
  formatAmount,
}: {
  row: DashboardRevenueRowDto;
  formatAmount: FormatAmount;
}) {
  return (
    <tr className="border-t border-line-soft first:border-0 align-baseline">
      <td className="py-1.5 w-10 whitespace-nowrap text-[11px] font-medium text-ink-3 tabular">
        {row.amount.currency.code}
      </td>
      <td className="py-1.5 pl-5 whitespace-nowrap text-right font-medium text-[15px] font-mono tabular">
        {formatAmount(row.amount)}
      </td>
      <td className="w-full" aria-hidden="true" />
      <td className="py-1.5 pl-2 text-right whitespace-nowrap text-[12px] text-ink-3 font-mono tabular">
        {row.invoice_count}
      </td>
    </tr>
  );
}

// ─── Card 2: Outstanding ─────────────────────────────────────────────────

function OutstandingCard({
  summary,
  formatAmount,
  onFollowUp,
}: {
  summary: DashboardSummaryDto;
  formatAmount: FormatAmount;
  onFollowUp: () => void;
}) {
  const { t } = useTranslation();
  return (
    <DashboardCard
      title={
        <span className="inline-flex items-baseline gap-2">
          {t("dashboard.card_outstanding_title")}
          <span className="text-danger">
            [{t("dashboard.card_outstanding_meta")}]
          </span>
        </span>
      }
      footer={
        summary.overdue_count > 0 ? (
          <div className="flex items-center justify-between gap-3">
            <span className="inline-flex items-center gap-1.5 text-danger">
              <AlertCircle size={11} strokeWidth={1.5} />
              {t("dashboard.card_outstanding_footer", {
                count: summary.overdue_count,
                days: summary.overdue_max_days,
              })}
            </span>
            <button
              type="button"
              className="text-ink-2 hover:text-ink inline-flex items-center gap-1"
              onClick={onFollowUp}
            >
              {t("dashboard.follow_up_action")}
              <ArrowRight size={11} strokeWidth={1.5} />
            </button>
          </div>
        ) : (
          <span className="text-ink-3">{t("dashboard.no_overdue")}</span>
        )
      }
    >
      {summary.outstanding.length === 0 ? (
        <EmptyRows label={t("dashboard.no_outstanding")} />
      ) : (
        <div className="max-h-48 overflow-y-auto overflow-x-hidden">
          <table className="w-full border-collapse">
            <tbody>
              {summary.outstanding.map((r) => (
                <OutstandingRow
                  key={r.outstanding.currency.code}
                  row={r}
                  formatAmount={formatAmount}
                />
              ))}
            </tbody>
          </table>
        </div>
      )}
    </DashboardCard>
  );
}

function OutstandingRow({
  row,
  formatAmount,
}: {
  row: DashboardOutstandingRowDto;
  formatAmount: FormatAmount;
}) {
  const hasOverdue = row.overdue_count > 0;
  return (
    <tr className="border-t border-line-soft first:border-0 align-baseline">
      <td className="py-1.5 w-10 whitespace-nowrap text-[11px] font-medium text-ink-3 tabular">
        {row.outstanding.currency.code}
      </td>
      <td className="py-1.5 pl-5 whitespace-nowrap text-right font-medium text-[15px] font-mono tabular">
        {formatAmount(row.outstanding)}
      </td>
      <td className="py-1.5 pl-1 whitespace-nowrap text-left text-[12px] font-mono tabular text-danger">
        {hasOverdue ? `[${formatAmount(row.overdue)}]` : ""}
      </td>
      <td className="w-full" aria-hidden="true" />
      <td className="py-1.5 pl-2 text-right whitespace-nowrap text-[12px] text-ink-3 font-mono tabular">
        {row.open_count}
        {row.overdue_count > 0 ? (
          <span className="text-danger"> ({row.overdue_count})</span>
        ) : null}
      </td>
    </tr>
  );
}

// ─── Card 3: Activity ────────────────────────────────────────────────────

function ActivityCard({
  summary,
  year,
  onSeeMore,
}: {
  summary: DashboardSummaryDto;
  year: number;
  onSeeMore: () => void;
}) {
  const { t } = useTranslation();
  const delay =
    summary.avg_payment_delay_days == null
      ? null
      : Math.round(summary.avg_payment_delay_days);
  return (
    <DashboardCard
      title={t("dashboard.card_activity_title")}
      footer={
        <button
          type="button"
          className="text-ink-2 hover:text-ink inline-flex items-center gap-1"
          onClick={onSeeMore}
        >
          {t("dashboard.card_activity_footer")}
          <ArrowRight size={11} strokeWidth={1.5} />
        </button>
      }
    >
      <div className="grid grid-cols-2">
        <div className="pr-5 pb-4 border-r border-b border-line-soft">
          <Stat
            label={t("dashboard.stat_avg_payment_delay")}
            value={
              delay == null ? (
                "—"
              ) : (
                <span>
                  {delay}{" "}
                  <span className="text-[14px] text-ink-3 font-medium">
                    {t("dashboard.days_short")}
                  </span>
                </span>
              )
            }
            detail={t("dashboard.stat_avg_payment_delay_detail", {
              target: summary.avg_payment_delay_target_days,
            })}
          />
        </div>
        <div className="pl-5 pb-4 border-b border-line-soft">
          <Stat
            label={t("dashboard.stat_active_clients")}
            value={
              <span>
                {summary.active_clients_count}{" "}
                <span className="text-[14px] text-ink-3 font-medium">
                  {t("dashboard.stat_active_clients_unit")}
                </span>
              </span>
            }
            detail={t("dashboard.stat_active_clients_year", {
              count: summary.new_clients_this_year_count,
            })}
          />
        </div>
        <div className="pr-5 pt-4 border-r border-line-soft">
          <Stat
            label={t("dashboard.stat_finalized_invoices")}
            value={String(summary.finalized_this_year_count)}
            detail={t("dashboard.stat_finalized_invoices_detail", { year })}
          />
        </div>
        <div className="pl-5 pt-4">
          <Stat
            label={t("dashboard.stat_drafts")}
            value={String(summary.drafts_this_year_count)}
            detail={t("dashboard.stat_drafts_detail", { year })}
          />
        </div>
      </div>
    </DashboardCard>
  );
}

function Stat({
  label,
  value,
  detail,
}: {
  label: string;
  value: React.ReactNode;
  detail: string;
}) {
  return (
    <div className="flex flex-col gap-1">
      <div className="text-[12px] font-medium text-ink-3">{label}</div>
      <div className="text-[22px] font-semibold leading-none tabular tracking-[-0.01em]">
        {value}
      </div>
      <div className="text-[11px] text-ink-3 leading-snug">{detail}</div>
    </div>
  );
}

// ─── Shared layout primitives ────────────────────────────────────────────

function DashboardCard({
  title,
  meta,
  footer,
  children,
}: {
  title: React.ReactNode;
  meta?: React.ReactNode;
  footer?: React.ReactNode;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-card border border-line bg-paper p-4 flex flex-col gap-3 transition-colors hover:border-ink-4">
      <div className="flex items-baseline justify-between gap-2">
        <div className="text-[12px] font-medium text-ink-3">{title}</div>
        {meta ? <div className="text-[11px] text-ink-3">{meta}</div> : null}
      </div>
      <div className="flex-1 min-h-0">{children}</div>
      {footer ? (
        <div className="text-[11px] pt-2 border-t border-line-soft">
          {footer}
        </div>
      ) : null}
    </div>
  );
}

function EmptyRows({ label }: { label: string }) {
  return (
    <div className="flex items-center justify-center text-[12px] text-ink-3 py-6">
      {label}
    </div>
  );
}
