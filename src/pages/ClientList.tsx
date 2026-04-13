import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { useClientStore } from "../stores/clientStore";
import type { Client, NewClient, UpdateClientInput } from "../types/client";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; client: Client };

const emptyForm: NewClient = {
  name: "",
  email: "",
  address: "",
  phone: "",
  notes: "",
};

export function ClientList() {
  const { t } = useTranslation();
  const {
    clients,
    loading,
    error,
    query,
    setQuery,
    refresh,
    create,
    update,
    remove,
  } = useClientStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-zinc-900">{t("clients.title")}</h1>
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
        <label className="flex items-center gap-2 text-sm text-zinc-700">
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

      {error ? <p className="mb-4 text-sm text-red-600">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-zinc-600">{t("common.loading")}</p>
      ) : clients.length === 0 ? (
        <p className="text-sm text-zinc-600">{t("clients.none")}</p>
      ) : (
        <table className="w-full border-collapse text-sm">
          <thead>
            <tr className="border-b border-zinc-200 text-left text-zinc-600">
              <th className="py-2 pr-3 font-medium">{t("common.name")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.email")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.phone")}</th>
              <th className="py-2 pr-3 font-medium">{t("common.active")}</th>
              <th className="py-2 pr-3 font-medium"></th>
            </tr>
          </thead>
          <tbody>
            {clients.map((c) => (
              <tr key={c.id} className="border-b border-zinc-100">
                <td className="py-2 pr-3 font-medium text-zinc-900">{c.name}</td>
                <td className="py-2 pr-3 text-zinc-700">{c.email ?? "—"}</td>
                <td className="py-2 pr-3 text-zinc-700">{c.phone ?? "—"}</td>
                <td className="py-2 pr-3 text-zinc-700">
                  {c.active ? "✓" : "—"}
                </td>
                <td className="flex justify-end gap-2 py-2 pr-3">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", client: c })}
                  >
                    {t("common.edit")}
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

      {editor.mode !== "closed" ? (
        <ClientEditor
          initial={
            editor.mode === "edit"
              ? {
                  name: editor.client.name,
                  email: editor.client.email ?? "",
                  address: editor.client.address ?? "",
                  phone: editor.client.phone ?? "",
                  notes: editor.client.notes ?? "",
                }
              : emptyForm
          }
          onCancel={() => setEditor({ mode: "closed" })}
          onSubmit={async (form) => {
            if (editor.mode === "edit") {
              const payload: UpdateClientInput = {
                id: editor.client.id,
                name: form.name,
                email: form.email || null,
                address: form.address || null,
                phone: form.phone || null,
                notes: form.notes || null,
              };
              await update(payload);
            } else {
              await create(form);
            }
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
    <div className="fixed inset-0 z-10 flex items-center justify-center bg-black/40 p-4">
      <form
        className="w-full max-w-lg rounded-lg bg-white p-6 shadow-xl"
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
        <h2 className="mb-4 text-lg font-bold text-zinc-900">
          {t("clients.edit")}
        </h2>
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
        {err ? <p className="mt-3 text-sm text-red-600">{err}</p> : null}
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
