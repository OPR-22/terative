import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { useTaxStore } from "../stores/taxStore";
import type {
  NewTaxDefinitionDto,
  TaxDefinitionDto,
  UpdateTaxDto,
} from "../ipc";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; tax: TaxDefinitionDto };

interface Form {
  name: string;
  percentage: string;
  tax_id_number: string;
}

const emptyForm: Form = { name: "", percentage: "0", tax_id_number: "" };

export function TaxList() {
  const { t } = useTranslation();
  const {
    taxes,
    includeArchived,
    setIncludeArchived,
    loading,
    error,
    refresh,
    create,
    update,
    archive,
    unarchive,
  } = useTaxStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="max-w-4xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("taxes.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("taxes.new")}
        </Button>
      </div>

      <div className="mb-4 flex items-center gap-3">
        <label className="flex items-center gap-2 text-sm text-fg-muted">
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
      ) : taxes.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("taxes.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("common.name")}</th>
              <th className="py-2 pr-3 font-medium">{t("taxes.percentage")}</th>
              <th className="py-2 pr-3 font-medium">{t("taxes.tax_id_number")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.active")}</th>
              <th className="py-2 pr-3"></th>
            </tr>
          </thead>
          <tbody>
            {taxes.map((tax) => (
              <tr key={tax.id} className="border-b border-border">
                <td className="py-2 pr-3 font-medium text-fg">{tax.name}</td>
                <td className="py-2 pr-3 text-fg-muted">{tax.percentage}%</td>
                <td className="py-2 pr-3 text-fg-muted">
                  {tax.tax_id_number ?? "—"}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {tax.archived_at ? "—" : "✓"}
                </td>
                <td className="flex justify-end gap-2 py-2 pr-3">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", tax })}
                  >
                    {t("common.edit")}
                  </Button>
                  {tax.archived_at ? (
                    <Button onClick={() => void unarchive(tax.id)}>
                      {t("common.unarchive")}
                    </Button>
                  ) : (
                    <Button
                      variant="danger"
                      onClick={() => {
                        if (confirm(t("common.confirm_archive"))) {
                          void archive(tax.id);
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
        <TaxEditor
          initial={
            editor.mode === "edit"
              ? {
                  name: editor.tax.name,
                  percentage: editor.tax.percentage,
                  tax_id_number: editor.tax.tax_id_number ?? "",
                }
              : emptyForm
          }
          onCancel={() => setEditor({ mode: "closed" })}
          onSubmit={async (form) => {
            if (editor.mode === "edit") {
              const payload: UpdateTaxDto = {
                id: editor.tax.id,
                name: form.name,
                percentage: form.percentage,
                tax_id_number: form.tax_id_number || null,
              };
              await update(payload);
            } else {
              const payload: NewTaxDefinitionDto = {
                name: form.name,
                percentage: form.percentage,
                tax_id_number: form.tax_id_number || null,
              };
              await create(payload);
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
  onCancel: () => void;
  onSubmit: (form: Form) => void | Promise<void>;
}

function TaxEditor({ initial, onCancel, onSubmit }: EditorProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<Form>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const updateField = <K extends keyof Form>(key: K, value: Form[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

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
          {t("taxes.edit")}
        </h2>
        <div className="flex flex-col gap-3">
          <Input
            label={t("common.name") ?? ""}
            value={form.name}
            onChange={(e) => updateField("name", e.target.value)}
            required
          />
          <Input
            label={t("taxes.percentage") ?? ""}
            type="number"
            step="0.01"
            min="0"
            value={form.percentage}
            onChange={(e) => updateField("percentage", e.target.value)}
            required
          />
          <Input
            label={t("taxes.tax_id_number") ?? ""}
            value={form.tax_id_number}
            onChange={(e) => updateField("tax_id_number", e.target.value)}
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
