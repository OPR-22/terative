import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { Pagination } from "../components/common/Pagination";
import { ClientAttributeDatalists } from "../components/client/ClientAttributeDatalists";
import { ContactListEditor } from "../components/client/ContactListEditor";
import { useClientStore } from "../stores/clientStore";
import type { ClientDto, ContactEntryDto, NewClientDto } from "../ipc";

type EditorState = { mode: "closed" } | { mode: "create" };

const emptyForm: NewClientDto = {
  name: "",
  emails: [],
  phones: [],
  address: null,
  notes: null,
  referred_by: null,
  date_of_birth: null,
  sex: null,
  gender: null,
  pronouns: null,
  occupation: null,
  language: null,
};

const defaultContact = (entries: ContactEntryDto[]): string =>
  entries.find((e) => e.is_default)?.value ?? entries[0]?.value ?? "—";

export function ClientList() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const {
    clients,
    page,
    currentPage,
    perPage,
    loading,
    error,
    query,
    setQuery,
    setPage,
    setPerPage,
    refresh,
    create,
    archive,
    unarchive,
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
            checked={query.include_archived ?? false}
            onChange={(e) =>
              setQuery({ ...query, include_archived: e.target.checked })
            }
          />
          {t("common.include_archived")}
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
                <td className="py-2 pr-3 text-fg-muted">
                  {defaultContact(c.emails)}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {defaultContact(c.phones)}
                </td>
                <td className="py-2 pr-3 text-fg-muted">
                  {c.archived_at ? "—" : "✓"}
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
                  {c.archived_at ? (
                    <Button onClick={() => void unarchive(c.id)}>
                      {t("common.unarchive")}
                    </Button>
                  ) : (
                    <Button
                      variant="danger"
                      onClick={() => {
                        if (confirm(t("common.confirm_archive"))) {
                          void archive(c.id);
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

      {page ? (
        <Pagination
          first={page.first}
          last={page.last}
          previous={page.previous}
          next={page.next}
          total={page.total}
          currentPage={currentPage}
          perPage={perPage}
          onPageChange={setPage}
          onPerPageChange={setPerPage}
        />
      ) : null}

      {editor.mode === "create" ? (
        <ClientEditor
          initial={emptyForm}
          allClients={clients}
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
  initial: NewClientDto;
  allClients: ClientDto[];
  onCancel: () => void;
  onSubmit: (form: NewClientDto) => void | Promise<void>;
}

function ClientEditor({ initial, allClients, onCancel, onSubmit }: EditorProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<NewClientDto>(initial);
  const [submitting, setSubmitting] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const attributeValues = useClientStore((s) => s.attributeValues);
  const refreshAttrs = useClientStore((s) => s.refreshAttributeValues);

  useEffect(() => {
    void refreshAttrs();
  }, [refreshAttrs]);

  return (
    <div className="fixed inset-0 z-10 flex items-start justify-center overflow-y-auto bg-overlay p-4">
      <form
        className="my-8 w-full max-w-2xl rounded-card bg-surface p-6 shadow-card"
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
        <div className="flex flex-col gap-4">
          <Input
            label={t("common.name") ?? ""}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            required
          />

          <ContactListEditor
            title={t("clients.emails")}
            value={form.emails}
            onChange={(emails) => setForm({ ...form, emails })}
            type="email"
            addLabel={t("clients.add_email")}
            emptyLabel={t("clients.no_emails")}
          />

          <ContactListEditor
            title={t("clients.phones")}
            value={form.phones}
            onChange={(phones) => setForm({ ...form, phones })}
            type="tel"
            addLabel={t("clients.add_phone")}
            emptyLabel={t("clients.no_phones")}
          />

          <Input
            label={t("common.address") ?? ""}
            value={form.address ?? ""}
            onChange={(e) => setForm({ ...form, address: e.target.value || null })}
          />
          <Input
            label={t("common.notes") ?? ""}
            value={form.notes ?? ""}
            onChange={(e) => setForm({ ...form, notes: e.target.value || null })}
          />

          <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
            <Input
              label={t("clients.date_of_birth") ?? ""}
              type="date"
              value={form.date_of_birth ?? ""}
              onChange={(e) =>
                setForm({ ...form, date_of_birth: e.target.value || null })
              }
            />
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("clients.sex")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
                value={form.sex ?? ""}
                onChange={(e) =>
                  setForm({ ...form, sex: e.target.value || null })
                }
              >
                <option value="">{t("clients.no_sex")}</option>
                <option value="female">{t("clients.sex_female")}</option>
                <option value="male">{t("clients.sex_male")}</option>
                <option value="intersex">{t("clients.sex_intersex")}</option>
              </select>
            </label>
            <Input
              label={t("clients.gender") ?? ""}
              value={form.gender ?? ""}
              onChange={(e) =>
                setForm({ ...form, gender: e.target.value || null })
              }
              placeholder={t("clients.gender_placeholder") ?? ""}
              list="gender-suggestions"
            />
            <Input
              label={t("clients.pronouns") ?? ""}
              value={form.pronouns ?? ""}
              onChange={(e) =>
                setForm({ ...form, pronouns: e.target.value || null })
              }
              placeholder={t("clients.pronouns_placeholder") ?? ""}
              list="pronouns-suggestions"
            />
            <Input
              label={t("clients.occupation") ?? ""}
              value={form.occupation ?? ""}
              onChange={(e) =>
                setForm({ ...form, occupation: e.target.value || null })
              }
              list="occupation-suggestions"
            />
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("clients.language")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
                value={form.language ?? ""}
                onChange={(e) =>
                  setForm({ ...form, language: e.target.value || null })
                }
              >
                <option value="">{t("clients.no_language")}</option>
                <option value="fr">Français</option>
                <option value="en">English</option>
                <option value="nl">Nederlands</option>
                <option value="de">Deutsch</option>
              </select>
            </label>
          </div>

          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("clients.referred_by")}
            <select
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={form.referred_by ?? ""}
              onChange={(e) =>
                setForm({ ...form, referred_by: e.target.value || null })
              }
            >
              <option value="">{t("clients.no_referrer")}</option>
              {allClients.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.name}
                  {c.archived_at ? ` (${t("clients.archived")})` : ""}
                </option>
              ))}
            </select>
          </label>
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
        <ClientAttributeDatalists values={attributeValues} />
      </form>
    </div>
  );
}
