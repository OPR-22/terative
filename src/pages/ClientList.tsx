import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { useClientStore } from "../stores/clientStore";
import type { NewClient } from "../types/client";

type EditorState = { mode: "closed" } | { mode: "create" };

const emptyForm: NewClient = {
  name: "",
  email: "",
  address: "",
  phone: "",
  notes: "",
};

export function ClientList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    clients,
    loading,
    error,
    query,
    setQuery,
    refresh,
    create,
    remove,
  } = useClientStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("clients.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("clients.new")}
        </Button>
      </div>

      <div className="mb-4 flex items-center gap-3">
        <Input
          placeholder={t("common.search")}
          value={query.search ?? ""}
          onChange={(e) => setQuery({ ...query, search: e.target.value })}
        />
        <label className="flex items-center gap-2 text-sm text-fg-muted">
          <input
            type="checkbox"
            checked={query.include_inactive ?? false}
            onChange={(e) =>
              setQuery({ ...query, include_inactive: e.target.checked })
            }
          />
          {t("common.include_inactive")}
        </label>
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : clients.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("clients.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-border text-left text-fg-muted">
              <th className="py-2 pr-3 font-medium">{t("common.name")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.email")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.phone")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.active")}</th>
              <th className="py-2 pr-3 font-medium"></th>
            </tr>
          </thead>
          <tbody>
            {clients.map((c) => (
              <tr
                key={c.id}
                className="cursor-pointer border-b border-border transition-colors hover:bg-surface-muted"
                onClick={() => navigate(`/clients/${c.id}`)}
              >
                <td className="py-2 pr-3 font-medium text-fg">{c.name}</td>
                <td className="py-2 pr-3 text-fg-muted">{c.email ?? "—"}</td>
                <td className="py-2 pr-3 text-fg-muted">{c.phone ?? "—"}</td>
                <td className="py-2 pr-3 text-fg-muted">
                  {c.active ? "✓" : "—"}
                </td>
                <td
                  className="flex justify-end gap-2 py-2 pr-3"
                  onClick={(e) => e.stopPropagation()}
                >
                  <Button
                    variant="secondary"
                    onClick={() => navigate(`/clients/${c.id}`)}
                  >
                    {t("common.view")}
                  </Button>
                  <Button
                    variant="danger"
                    onClick={() => {
                      if (confirm(t("common.confirm_delete"))) {
                        void remove(c.id);
                      }
                    }}
                  >
                    {t("common.delete")}
                  </Button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      )}

      {editor.mode === "create" ? (
        <ClientEditor
          initial={emptyForm}
          onCancel={() => setEditor({ mode: "closed" })}
          onSubmit={async (form) => {
            await create(form);
            setEditor({ mode: "closed" });
          }}
        />
      ) : null}
    </div>
  );
}

interface EditorProps {
  initial: NewClient;
  onCancel: () => void;
  onSubmit: (form: NewClient) => void | Promise<void>;
}

function ClientEditor({ initial, onCancel, onSubmit }: EditorProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<NewClient>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  const update = <K extends keyof NewClient>(key: K, value: NewClient[K]) =>
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
        <h2 className="mb-4 text-lg font-bold text-fg">{t("clients.new")}</h2>
        <div className="flex flex-col gap-3">
          <Input
            label={t("common.name") ?? ""}
            value={form.name}
            onChange={(e) => update("name", e.target.value)}
            required
          />
          <Input
            label={t("common.email") ?? ""}
            type="email"
            value={form.email ?? ""}
            onChange={(e) => update("email", e.target.value)}
          />
          <Input
            label={t("common.phone") ?? ""}
            value={form.phone ?? ""}
            onChange={(e) => update("phone", e.target.value)}
          />
          <Input
            label={t("common.address") ?? ""}
            value={form.address ?? ""}
            onChange={(e) => update("address", e.target.value)}
          />
          <Input
            label={t("common.notes") ?? ""}
            value={form.notes ?? ""}
            onChange={(e) => update("notes", e.target.value)}
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
