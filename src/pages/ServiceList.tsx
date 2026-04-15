import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { MoneyInput } from "../components/common/MoneyInput";
import { useServiceStore } from "../stores/serviceStore";
import { useSettingsStore } from "../stores/settingsStore";
import type { MoneyDto, ServiceDto } from "../ipc";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; service: ServiceDto };

interface Form {
  name: string;
  price: MoneyDto;
}

export function ServiceList() {
  const { t } = useTranslation();
  const {
    services,
    loading,
    error,
    includeInactive,
    setIncludeInactive,
    refresh,
    create,
    update,
    archive,
    unarchive,
  } = useServiceStore();
  const { snapshot, load } = useSettingsStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
    if (!snapshot) void load();
  }, [refresh, load, snapshot]);

  const currency = snapshot?.currency;
  const currencyCode = currency?.code ?? "EUR";
  const currencySymbol = currency?.symbol ?? "€";

  const formatMoney = (m: MoneyDto) =>
    `${(m.amount_cents / 100).toFixed(2)} ${currencySymbol}`;

  return (
    <div className="max-w-4xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">
          {t("services.title")}
        </h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("services.new")}
        </Button>
      </div>

      <label className="mb-4 flex items-center gap-2 text-sm text-fg-muted">
        <input
          type="checkbox"
          checked={includeInactive}
          onChange={(e) => setIncludeInactive(e.target.checked)}
        />
        {t("common.include_inactive")}
      </label>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : services.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("services.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("common.name")}</th>
              <th className="py-2 pr-3 font-medium">
                {t("services.default_price")}
              </th>
              <th className="py-2 pr-3 font-medium">{t("common.active")}</th>
              <th className="py-2 pr-3"></th>
            </tr>
          </thead>
          <tbody>
            {services.map((s) => (
              <tr key={s.id} className="border-b border-border">
                <td className="py-2 pr-3 font-medium text-fg">
                  {s.name}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {formatMoney(s.default_price)}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {s.active ? "✓" : "—"}
                </td>
                <td className="flex justify-end gap-2 py-2 pr-3">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", service: s })}
                  >
                    {t("common.edit")}
                  </Button>
                  {s.active ? (
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
                  ) : (
                    <Button onClick={() => void unarchive(s.id)}>
                      {t("common.unarchive")}
                    </Button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editor.mode !== "closed" ? (
        <ServiceEditor
          initial={
            editor.mode === "edit"
              ? {
                  name: editor.service.name,
                  price: editor.service.default_price,
                }
              : {
                  name: "",
                  price: { amount_cents: 0, currency: currencyCode },
                }
          }
          currencySymbol={currencySymbol}
          onCancel={() => setEditor({ mode: "closed" })}
          onSubmit={async (form) => {
            if (editor.mode === "edit") {
              await update({
                id: editor.service.id,
                name: form.name,
                default_price: form.price,
              });
            } else {
              await create({ name: form.name, default_price: form.price });
            }
            setEditor({ mode: "closed" });
          }}
        />
      ) : null}
    </div>
  );
}

interface EditorProps {
  initial: Form;
  currencySymbol: string;
  onCancel: () => void;
  onSubmit: (form: Form) => void | Promise<void>;
}

function ServiceEditor({
  initial,
  currencySymbol,
  onCancel,
  onSubmit,
}: EditorProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<Form>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  return (
    <div className="fixed inset-0 z-10 flex items-center justify-center bg-overlay p-4">
      <form
        className="w-full max-w-lg rounded-card bg-surface p-6 shadow-card"
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
        <h2 className="mb-4 text-lg font-bold text-fg">
          {t("services.edit")}
        </h2>
        <div className="flex flex-col gap-3">
          <Input
            label={t("common.name") ?? ""}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            required
          />
          <MoneyInput
            label={t("services.default_price") ?? ""}
            valueCents={form.price.amount_cents}
            currencySymbol={currencySymbol}
            onChangeCents={(cents) =>
              setForm({
                ...form,
                price: { ...form.price, amount_cents: cents },
              })
            }
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
