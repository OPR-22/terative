import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { MoneyInput } from "../components/common/MoneyInput";
import { useMoneyFormat } from "../lib/money";
import { useCatalogStore } from "../stores/catalogStore";
import { useSettingsStore } from "../stores/settingsStore";
import type {
  CatalogItemDto,
  CatalogItemKindDto,
  CurrencyConfigDto,
  MoneyDto,
} from "../ipc";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; item: CatalogItemDto };

interface Form {
  name: string;
  kind: CatalogItemKindDto;
  price: MoneyDto;
  unit: string;
  reference: string;
}

const KINDS: CatalogItemKindDto[] = ["Service", "Product"];

export function CatalogList() {
  const { t } = useTranslation();
  const {
    items,
    loading,
    error,
    includeArchived,
    setIncludeArchived,
    refresh,
    create,
    update,
    archive,
    unarchive,
  } = useCatalogStore();
  const { snapshot, load } = useSettingsStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });
  const [kindFilter, setKindFilter] = useState<"All" | CatalogItemKindDto>(
    "All",
  );

  useEffect(() => {
    void refresh();
    if (!snapshot) void load();
  }, [refresh, load, snapshot]);

  const { format: formatMoney } = useMoneyFormat();
  const currency = snapshot?.currency;
  const currencyCode = currency?.code ?? "EUR";

  const visibleItems =
    kindFilter === "All" ? items : items.filter((i) => i.kind === kindFilter);

  return (
    <div className="max-w-4xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("catalog.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("catalog.new")}
        </Button>
      </div>

      <div className="mb-4 flex flex-wrap items-center gap-3">
        <div className="flex gap-1">
          <FilterPill
            active={kindFilter === "All"}
            onClick={() => setKindFilter("All")}
            label={t("catalog.filter_all")}
          />
          <FilterPill
            active={kindFilter === "Service"}
            onClick={() => setKindFilter("Service")}
            label={t("catalog.kind_service_plural")}
          />
          <FilterPill
            active={kindFilter === "Product"}
            onClick={() => setKindFilter("Product")}
            label={t("catalog.kind_product_plural")}
          />
        </div>
        <label className="ml-auto flex items-center gap-2 text-sm text-fg-muted">
          <input
            type="checkbox"
            checked={includeArchived}
            onChange={(e) => setIncludeArchived(e.target.checked)}
          />
          {t("common.include_archived")}
        </label>
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : visibleItems.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("catalog.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("catalog.kind")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.name")}</th>
              <th className="py-2 pr-3 font-medium">{t("catalog.reference")}</th>
              <th className="py-2 pr-3 font-medium">
                {t("catalog.default_price")}
              </th>
              <th className="py-2 pr-3 font-medium">{t("catalog.unit")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.active")}</th>
              <th className="py-2 pr-3"></th>
            </tr>
          </thead>
          <tbody>
            {visibleItems.map((s) => (
              <tr key={s.id} className="border-b border-border">
                <td className="py-2 pr-3 text-fg-muted">
                  {t(`catalog.kind_${s.kind.toLowerCase()}`)}
                </td>
                <td className="py-2 pr-3 font-medium text-fg">{s.name}</td>
                <td className="py-2 pr-3 text-fg-muted">
                  {s.reference ?? "—"}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {formatMoney(s.default_price)}
                </td>
                <td className="py-2 pr-3 text-fg-muted">{s.unit ?? "—"}</td>
                <td className="py-2 pr-3 text-fg-muted">
                  {s.archived_at ? "—" : "✓"}
                </td>
                <td className="flex justify-end gap-2 py-2 pr-3">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", item: s })}
                  >
                    {t("common.edit")}
                  </Button>
                  {s.archived_at ? (
                    <Button onClick={() => void unarchive(s.id)}>
                      {t("common.unarchive")}
                    </Button>
                  ) : (
                    <Button
                      variant="danger"
                      onClick={() => {
                        if (confirm(t("common.confirm_archive"))) {
                          void archive(s.id);
                        }
                      }}
                    >
                      {t("common.archive")}
                    </Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editor.mode !== "closed" ? (
        <CatalogItemEditor
          initial={
            editor.mode === "edit"
              ? {
                  name: editor.item.name,
                  kind: editor.item.kind,
                  price: editor.item.default_price,
                  unit: editor.item.unit ?? "",
                  reference: editor.item.reference ?? "",
                }
              : {
                  name: "",
                  kind: "Service",
                  price: { amount_minor: 0, currency: currencyCode },
                  unit: "",
                  reference: "",
                }
          }
          currency={currency}
          onCancel={() => setEditor({ mode: "closed" })}
          onSubmit={async (form) => {
            if (editor.mode === "edit") {
              await update({
                id: editor.item.id,
                name: form.name,
                kind: form.kind,
                default_price: form.price,
                unit: form.unit.trim() || null,
                reference: form.reference.trim() || null,
              });
            } else {
              await create({
                name: form.name,
                kind: form.kind,
                default_price: form.price,
                unit: form.unit.trim() || null,
                reference: form.reference.trim() || null,
              });
            }
            setEditor({ mode: "closed" });
          }}
        />
      ) : null}
    </div>
  );
}

function FilterPill({
  active,
  onClick,
  label,
}: {
  active: boolean;
  onClick: () => void;
  label: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "rounded-pill px-3 py-1 text-xs font-medium transition-colors",
        active
          ? "bg-brand text-brand-fg"
          : "bg-surface-muted text-fg-muted hover:bg-border",
      ].join(" ")}
    >
      {label}
    </button>
  );
}

interface EditorProps {
  initial: Form;
  currency: CurrencyConfigDto | undefined;
  onCancel: () => void;
  onSubmit: (form: Form) => void | Promise<void>;
}

function CatalogItemEditor({
  initial,
  currency,
  onCancel,
  onSubmit,
}: EditorProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<Form>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  return (
    <div className="fixed inset-0 z-10 flex items-start justify-center overflow-y-auto bg-overlay p-4">
      <form
        className="my-8 w-full max-w-lg rounded-card bg-surface p-6 shadow-card"
        onSubmit={async (e) => {
          e.preventDefault();
          setErr(null);
          setSubmitting(true);
          try {
            await onSubmit(form);
          } catch (e) {
            setErr(String(e));
          } finally {
            setSubmitting(false);
          }
        }}
      >
        <h2 className="mb-4 text-lg font-bold text-fg">{t("catalog.edit")}</h2>
        <div className="flex flex-col gap-3">
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("catalog.kind")}
            <select
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={form.kind}
              onChange={(e) =>
                setForm({ ...form, kind: e.target.value as CatalogItemKindDto })
              }
            >
              {KINDS.map((k) => (
                <option key={k} value={k}>
                  {t(`catalog.kind_${k.toLowerCase()}`)}
                </option>
              ))}
            </select>
          </label>
          <Input
            label={t("common.name") ?? ""}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            required
          />
          <Input
            label={t("catalog.reference") ?? ""}
            value={form.reference}
            onChange={(e) => setForm({ ...form, reference: e.target.value })}
            placeholder={t("catalog.reference_placeholder") ?? ""}
          />
          {currency ? (
            <MoneyInput
              label={t("catalog.default_price") ?? ""}
              valueMinor={form.price.amount_minor}
              currency={currency}
              onChangeMinor={(minor) =>
                setForm({
                  ...form,
                  price: { ...form.price, amount_minor: minor },
                })
              }
            />
          ) : null}
          <Input
            label={t("catalog.unit") ?? ""}
            value={form.unit}
            onChange={(e) => setForm({ ...form, unit: e.target.value })}
            placeholder={t("catalog.unit_placeholder") ?? ""}
          />
        </div>
        {err ? <p className="mt-3 text-sm text-danger">{err}</p> : null}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" type="button" onClick={onCancel}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={submitting}>
            {t("common.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
