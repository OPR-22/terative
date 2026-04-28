import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router-dom";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { ClientAttributeDatalists } from "../components/client/ClientAttributeDatalists";
import { ContactListEditor } from "../components/client/ContactListEditor";
import { useClientStore } from "../stores/clientStore";
import {
  ipc,
  type ClientDto,
  type ClientJournalEntryDto,
  type ClientNotebookViewDto,
  type ContactEntryDto,
  type NewJournalEntryDto,
  type UpdateClientDto,
  type UpdateJournalEntryDto,
} from "../ipc";

type Tab = "info" | "notebook" | "journal";

export function ClientDetail() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const [tab, setTab] = useState<Tab>("info");
  const [client, setClient] = useState<ClientDto | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    ipc
      .clientGet(id)
      .then((c) => {
        if (!cancelled) setClient(c);
      })
      .catch((e) => {
        if (!cancelled) setError(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  if (!id) return null;
  if (error) {
    return (
      <div className="max-w-5xl">
        <p className="text-sm text-danger">{error}</p>
        <Button variant="secondary" onClick={() => navigate("/clients")}>
          {t("common.back")}
        </Button>
      </div>
    );
  }
  if (!client) {
    return (
      <p className="text-sm text-fg-muted">{t("common.loading")}</p>
    );
  }

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Button variant="secondary" onClick={() => navigate("/clients")}>
            ← {t("common.back")}
          </Button>
          <h1 className="text-2xl font-bold text-fg">{client.name}</h1>
        </div>
      </div>

      <div className="mb-4 flex gap-2 border-b border-border">
        <TabButton active={tab === "info"} onClick={() => setTab("info")}>
          {t("clients.tab_info")}
        </TabButton>
        <TabButton
          active={tab === "notebook"}
          onClick={() => setTab("notebook")}
        >
          {t("clients.tab_notebook")}
        </TabButton>
        <TabButton active={tab === "journal"} onClick={() => setTab("journal")}>
          {t("clients.tab_journal")}
        </TabButton>
      </div>

      {tab === "info" ? (
        <InfoTab client={client} onSaved={setClient} />
      ) : null}
      {tab === "notebook" ? <NotebookTab clientId={client.id} /> : null}
      {tab === "journal" ? <JournalTab clientId={client.id} /> : null}
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
        "border-b-2 px-4 py-2 text-sm font-medium transition-colors",
        active
          ? "border-brand text-fg"
          : "border-transparent text-fg-muted hover:text-fg",
      ].join(" ")}
    >
      {children}
    </button>
  );
}

/// Returns a localized "X years old" string for the given DOB, or null if
/// no DOB or the date doesn't parse. Calendar-correct (rolls back a year
/// if the birthday hasn't happened yet this year).
function computeAgeLabel(
  dob: string | null,
  t: (key: string, opts?: Record<string, unknown>) => string,
): string | null {
  if (!dob) return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dob);
  if (!m) return null;
  const birthY = Number(m[1]);
  const birthM = Number(m[2]);
  const birthD = Number(m[3]);
  const today = new Date();
  let age = today.getFullYear() - birthY;
  const monthDiff = today.getMonth() + 1 - birthM;
  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthD)) {
    age -= 1;
  }
  if (age < 0) return null;
  return t("clients.age_years", { count: age });
}

// ---- Info tab ----

function InfoTab({
  client,
  onSaved,
}: {
  client: ClientDto;
  onSaved: (c: ClientDto) => void;
}) {
  const { t } = useTranslation();
  const {
    clients,
    refresh: refreshClients,
    attributeValues,
    refreshAttributeValues,
  } = useClientStore();
  const [name, setName] = useState(client.name);
  const [emails, setEmails] = useState<ContactEntryDto[]>(client.emails);
  const [phones, setPhones] = useState<ContactEntryDto[]>(client.phones);
  const [address, setAddress] = useState(client.address ?? "");
  const [notes, setNotes] = useState(client.notes ?? "");
  const [referredBy, setReferredBy] = useState<string | null>(
    client.referred_by,
  );
  const [dateOfBirth, setDateOfBirth] = useState(client.date_of_birth ?? "");
  const [sex, setSex] = useState(client.sex ?? "");
  const [gender, setGender] = useState(client.gender ?? "");
  const [pronouns, setPronouns] = useState(client.pronouns ?? "");
  const [occupation, setOccupation] = useState(client.occupation ?? "");
  const [language, setLanguage] = useState(client.language ?? "");
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (clients.length === 0) void refreshClients();
  }, [clients.length, refreshClients]);

  useEffect(() => {
    void refreshAttributeValues();
  }, [refreshAttributeValues]);

  useEffect(() => {
    setName(client.name);
    setEmails(client.emails);
    setPhones(client.phones);
    setAddress(client.address ?? "");
    setNotes(client.notes ?? "");
    setReferredBy(client.referred_by);
    setDateOfBirth(client.date_of_birth ?? "");
    setSex(client.sex ?? "");
    setGender(client.gender ?? "");
    setPronouns(client.pronouns ?? "");
    setOccupation(client.occupation ?? "");
    setLanguage(client.language ?? "");
  }, [client]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setBusy(true);
    try {
      const payload: UpdateClientDto = {
        id: client.id,
        name: name.trim(),
        emails,
        phones,
        address: address.trim() || null,
        notes: notes.trim() || null,
        referred_by: referredBy,
        date_of_birth: dateOfBirth || null,
        sex: sex || null,
        gender: gender.trim() || null,
        pronouns: pronouns.trim() || null,
        occupation: occupation.trim() || null,
        language: language || null,
      };
      const updated = await ipc.clientUpdate(payload);
      onSaved(updated);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const ageLabel = computeAgeLabel(client.date_of_birth, t);

  const referrerOptions = clients.filter((c) => c.id !== client.id);

  return (
    <form
      onSubmit={onSubmit}
      className="flex flex-col gap-4 rounded-card border border-border bg-surface p-5 shadow-card"
    >
      <Input
        label={t("common.name") ?? ""}
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
      />

      <ContactListEditor
        title={t("clients.emails")}
        value={emails}
        onChange={setEmails}
        type="email"
        addLabel={t("clients.add_email")}
        emptyLabel={t("clients.no_emails")}
      />

      <ContactListEditor
        title={t("clients.phones")}
        value={phones}
        onChange={setPhones}
        type="tel"
        addLabel={t("clients.add_phone")}
        emptyLabel={t("clients.no_phones")}
      />

      <Input
        label={t("common.address") ?? ""}
        value={address}
        onChange={(e) => setAddress(e.target.value)}
      />
      <Input
        label={t("common.notes") ?? ""}
        value={notes}
        onChange={(e) => setNotes(e.target.value)}
      />

      <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
        <div className="flex flex-col gap-1">
          <Input
            type="date"
            label={t("clients.date_of_birth") ?? ""}
            value={dateOfBirth}
            onChange={(e) => setDateOfBirth(e.target.value)}
          />
          {ageLabel ? (
            <span className="text-xs text-fg-subtle">{ageLabel}</span>
          ) : null}
        </div>
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("clients.sex")}
          <select
            className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={sex}
            onChange={(e) => setSex(e.target.value)}
          >
            <option value="">{t("clients.no_sex")}</option>
            <option value="female">{t("clients.sex_female")}</option>
            <option value="male">{t("clients.sex_male")}</option>
            <option value="intersex">{t("clients.sex_intersex")}</option>
          </select>
        </label>
        <Input
          label={t("clients.gender") ?? ""}
          value={gender}
          onChange={(e) => setGender(e.target.value)}
          placeholder={t("clients.gender_placeholder") ?? ""}
          list="gender-suggestions"
        />
        <Input
          label={t("clients.pronouns") ?? ""}
          value={pronouns}
          onChange={(e) => setPronouns(e.target.value)}
          placeholder={t("clients.pronouns_placeholder") ?? ""}
          list="pronouns-suggestions"
        />
        <Input
          label={t("clients.occupation") ?? ""}
          value={occupation}
          onChange={(e) => setOccupation(e.target.value)}
          list="occupation-suggestions"
        />
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("clients.language")}
          <select
            className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={language}
            onChange={(e) => setLanguage(e.target.value)}
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
          value={referredBy ?? ""}
          onChange={(e) => setReferredBy(e.target.value || null)}
        >
          <option value="">{t("clients.no_referrer")}</option>
          {referrerOptions.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
              {c.archived_at ? ` (${t("clients.archived")})` : ""}
            </option>
          ))}
        </select>
      </label>

      {err ? <p className="text-sm text-danger">{err}</p> : null}
      <div className="flex items-center gap-3">
        <Button type="submit" disabled={busy}>
          {t("common.save")}
        </Button>
        {saved ? (
          <span className="text-sm text-success">{t("settings.saved")}</span>
        ) : null}
      </div>
      <ClientAttributeDatalists values={attributeValues} />
    </form>
  );
}

// ---- Notebook tab ----

function NotebookTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<ClientNotebookViewDto | null>(null);
  const [contents, setContents] = useState<Record<string, string>>({});
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = () => {
    setErr(null);
    ipc
      .clientNotebookGet(clientId)
      .then((v) => {
        setView(v);
        const map: Record<string, string> = {};
        for (const s of v.sections) map[s.section.id] = s.content;
        setContents(map);
        setDirty(false);
      })
      .catch((e) => setErr(String(e)));
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clientId]);

  const updateContent = (sectionId: string, content: string) => {
    setContents((prev) => ({ ...prev, [sectionId]: content }));
    setDirty(true);
  };

  const onSave = async () => {
    if (!view) return;
    setBusy(true);
    setErr(null);
    try {
      await ipc.clientNotebookSave({
        client_id: clientId,
        entries: view.sections.map((s) => ({
          section_id: s.section.id,
          content: contents[s.section.id] ?? "",
        })),
      });
      setDirty(false);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!view) {
    return err ? (
      <p className="text-sm text-danger">{err}</p>
    ) : (
      <p className="text-sm text-fg-muted">{t("common.loading")}</p>
    );
  }

  if (view.sections.length === 0) {
    return (
      <div className="rounded-card border border-border bg-surface p-5 text-sm text-fg-muted shadow-card">
        {t("clients.notebook_no_sections")}
      </div>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-end gap-3">
        {err ? <p className="text-sm text-danger">{err}</p> : null}
        {saved ? (
          <span className="text-sm text-success">{t("settings.saved")}</span>
        ) : null}
        <Button onClick={onSave} disabled={!dirty || busy}>
          {t("common.save")}
        </Button>
      </div>

      {view.sections.map((s) => (
        <section
          key={s.section.id}
          className="rounded-card border border-border bg-surface p-5 shadow-card"
        >
          <h2 className="mb-2 text-sm font-semibold text-fg-muted">
            {s.section.name}
          </h2>
          <textarea
            className="block min-h-32 w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={contents[s.section.id] ?? ""}
            onChange={(e) => updateContent(s.section.id, e.target.value)}
            placeholder={t("clients.notebook_placeholder") ?? ""}
          />
        </section>
      ))}
    </div>
  );
}

// ---- Journal tab ----

type JournalEditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; entry: ClientJournalEntryDto };

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function JournalTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<ClientJournalEntryDto[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [editor, setEditor] = useState<JournalEditorState>({ mode: "closed" });

  const refresh = () => {
    setErr(null);
    ipc
      .journalListForClient(clientId)
      .then(setEntries)
      .catch((e) => setErr(String(e)));
  };

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clientId]);

  const onDelete = async (id: string) => {
    if (!confirm(t("common.confirm_delete"))) return;
    try {
      await ipc.journalEntryDelete(id);
      refresh();
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-end">
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("clients.journal_new")}
        </Button>
      </div>

      {err ? <p className="text-sm text-danger">{err}</p> : null}

      {entries.length === 0 ? (
        <div className="rounded-card border border-border bg-surface p-5 text-sm text-fg-muted shadow-card">
          {t("clients.journal_none")}
        </div>
      ) : (
        <ul className="flex flex-col gap-3">
          {entries.map((entry) => (
            <li
              key={entry.id}
              className="rounded-card border border-border bg-surface p-5 shadow-card"
            >
              <div className="mb-2 flex items-start justify-between">
                <span className="text-sm font-semibold text-fg">
                  {entry.entry_date}
                </span>
                <div className="flex gap-2">
                  <Button
                    variant="secondary"
                    onClick={() => setEditor({ mode: "edit", entry })}
                  >
                    {t("common.edit")}
                  </Button>
                  <Button
                    variant="danger"
                    onClick={() => void onDelete(entry.id)}
                  >
                    {t("common.delete")}
                  </Button>
                </div>
              </div>
              <p className="whitespace-pre-wrap text-sm text-fg">
                {entry.content}
              </p>
            </li>
          ))}
        </ul>
      )}

      {editor.mode !== "closed" ? (
        <JournalEntryModal
          clientId={clientId}
          initial={editor.mode === "edit" ? editor.entry : null}
          onClose={() => {
            setEditor({ mode: "closed" });
            refresh();
          }}
        />
      ) : null}
    </div>
  );
}

function JournalEntryModal({
  clientId,
  initial,
  onClose,
}: {
  clientId: string;
  initial: ClientJournalEntryDto | null;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const [date, setDate] = useState(initial?.entry_date ?? todayIso());
  const [content, setContent] = useState(initial?.content ?? "");
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (content.trim() === "") {
      setErr(t("clients.journal_err_empty"));
      return;
    }
    setBusy(true);
    setErr(null);
    try {
      if (initial) {
        const payload: UpdateJournalEntryDto = {
          id: initial.id,
          entry_date: date,
          content,
        };
        await ipc.journalEntryUpdate(payload);
      } else {
        const payload: NewJournalEntryDto = {
          client_id: clientId,
          entry_date: date,
          content,
        };
        await ipc.journalEntryCreate(payload);
      }
      onClose();
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-10 flex items-start justify-center overflow-y-auto bg-overlay p-4">
      <form
        className="my-8 w-full max-w-lg rounded-card bg-surface p-6 shadow-card"
        onSubmit={submit}
      >
        <h2 className="mb-4 text-lg font-bold text-fg">
          {initial ? t("clients.journal_edit") : t("clients.journal_new")}
        </h2>
        <div className="flex flex-col gap-3">
          <Input
            type="date"
            label={t("common.date") ?? ""}
            value={date}
            onChange={(e) => setDate(e.target.value)}
            required
          />
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("clients.journal_content")}
            <textarea
              className="block min-h-40 w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={content}
              onChange={(e) => setContent(e.target.value)}
              autoFocus
            />
          </label>
        </div>
        {err ? <p className="mt-3 text-sm text-danger">{err}</p> : null}
        <div className="mt-5 flex justify-end gap-2">
          <Button variant="secondary" type="button" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" disabled={busy}>
            {t("common.save")}
          </Button>
        </div>
      </form>
    </div>
  );
}
