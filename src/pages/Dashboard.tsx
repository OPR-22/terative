import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import {
  AlertCircle,
  ArrowUp,
  Download,
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
import { ipc, type DashboardSummaryDto, type InvoicePaymentRowDto } from "../ipc";
import { useMoneyFormat } from "../lib/money";
import { useSettingsStore } from "../stores/settingsStore";
import { useClientStore } from "../stores/clientStore";

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
  const ensureDirectory = useClientStore((s) => s.ensureDirectory);
  const clientName = useClientStore((s) => s.clientName);
  const [summary, setSummary] = useState<DashboardSummaryDto | null>(null);
  const [overdue, setOverdue] = useState<InvoicePaymentRowDto[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!snapshot) void load();
    void ensureDirectory();
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
  }, [snapshot, load, ensureDirectory]);

  const { format } = useMoneyFormat();

  return (
    <Page
      crumbs={["Cabinet Lemaire", t("dashboard.title")]}
      title={t("dashboard.title")}
      subtitle="Vue d'ensemble"
      actions={
        <>
          <Button leadingIcon={<Download size={13} strokeWidth={1.5} />}>Exporter</Button>
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
      {error ? <p className="mb-4 text-[13px] text-danger">{error}</p> : null}

      {summary ? (
        <>
          <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-2 lg:grid-cols-4">
            <Kpi
              label={t("dashboard.revenue_this_year")}
              value={format(summary.revenue_this_year)}
              meta={
                <span className="inline-flex items-center gap-1">
                  <ArrowUp size={11} strokeWidth={1.5} className="text-ok" />
                  Vs. année précédente
                </span>
              }
            />
            <Kpi
              label={t("dashboard.outstanding")}
              value={format(summary.outstanding_total)}
              tone="warn"
              meta="Factures envoyées"
            />
            <Kpi
              label={t("dashboard.overdue_count")}
              value={String(summary.overdue_count)}
              tone="danger"
              meta={
                summary.overdue_count > 0 ? (
                  <span className="inline-flex items-center gap-1">
                    <AlertCircle size={11} strokeWidth={1.5} />
                    À relancer
                  </span>
                ) : (
                  "Aucun retard"
                )
              }
            />
            <Kpi
              label={t("dashboard.draft_count")}
              value={String(summary.draft_count)}
              meta="En cours d'édition"
            />
          </div>

          <SectionTitle>À traiter</SectionTitle>

          <div className="grid grid-cols-1 gap-4 lg:grid-cols-[1.6fr_1fr]">
            <Card>
              <CardHead
                title={t("dashboard.overdue_invoices")}
                subtitle={
                  overdue.length === 0
                    ? t("dashboard.no_overdue")
                    : `${overdue.length} factures à relancer`
                }
                actions={
                  <Button size="sm" onClick={() => navigate("/invoices")}>
                    Voir tout
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
                      <Th>Retard</Th>
                      <Th numeric>{t("accounting.amount_due")}</Th>
                      <Th />
                    </Tr>
                  </THead>
                  <tbody>
                    {overdue.slice(0, 6).map((row) => {
                      const days = daysOverdue(row.due_date);
                      const name = clientName(row.client_id);
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
                          <Td numeric>{format(row.amount_due)}</Td>
                          <Td className="text-right">
                            <Button
                              size="sm"
                              leadingIcon={<Send size={11} strokeWidth={1.5} />}
                              onClick={() =>
                                navigate(`/invoices/${row.invoice_id}/edit`)
                              }
                            >
                              Relance
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
              <CardHead title="Activité récente" />
              <div className="py-1.5">
                {[
                  {
                    Ic: Wallet,
                    title: "Paiement reçu",
                    detail: "Voir Paiements pour le détail",
                    when: "il y a quelques min",
                  },
                  {
                    Ic: Send,
                    title: "Facture envoyée",
                    detail: "Suivi des envois dans Factures",
                    when: "aujourd'hui",
                  },
                  {
                    Ic: FileText,
                    title: "Brouillon modifié",
                    detail: "Voir Factures > brouillons",
                    when: "récemment",
                  },
                  {
                    Ic: User,
                    title: "Nouveau client",
                    detail: "Voir la liste des clients",
                    when: "récemment",
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

interface KpiProps {
  label: string;
  value: string;
  meta?: React.ReactNode;
  tone?: "neutral" | "warn" | "danger" | "ok";
}

function Kpi({ label, value, meta, tone = "neutral" }: KpiProps) {
  const valueColor = {
    neutral: "text-ink",
    warn: "text-warn",
    danger: "text-danger",
    ok: "text-ok",
  }[tone];
  return (
    <div className="rounded-card border border-line bg-paper p-4 flex flex-col gap-2 transition-colors hover:border-ink-4">
      <div className="text-[12px] font-medium text-ink-3">{label}</div>
      <div
        className={[
          "text-[22px] font-semibold leading-none tabular tracking-[-0.01em]",
          valueColor,
        ].join(" ")}
      >
        {value}
      </div>
      {meta ? (
        <div className="text-[11px] text-ink-3 font-mono tabular flex items-center gap-1.5">
          {meta}
        </div>
      ) : null}
    </div>
  );
}
