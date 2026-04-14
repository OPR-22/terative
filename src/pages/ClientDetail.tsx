import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router-dom";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { clientsApi } from "../api/clients";
import { notebookApi } from "../api/notebook";
import type { Client, UpdateClientInput } from "../types/client";
import type {
  ClientJournalEntry,
  ClientNotebookView,
  NewJournalEntry,
  UpdateJournalEntryInput,
} from "../types/notebook";

type Tab = "info" | "notebook" | "journal";

export function ClientDetail() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const [tab, setTab] = useState<Tab>("info");
  const [client, setClient] = useState<Client | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    clientsApi
      .get(id)
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

// ---- Info tab ----

function InfoTab({
  client,
  onSaved,
}: {
  client: Client;
  onSaved: (c: Client) => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState(client.name);
  const [email, setEmail] = useState(client.email ?? "");
  const [phone, setPhone] = useState(client.phone ?? "");
  const [address, setAddress] = useState(client.address ?? "");
  const [notes, setNotes] = useState(client.notes ?? "");
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    setName(client.name);
    setEmail(client.email ?? "");
    setPhone(client.phone ?? "");
    setAddress(client.address ?? "");
    setNotes(client.notes ?? "");
  }, [client]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setBusy(true);
    try {
      const payload: UpdateClientInput = {
        id: client.id,
        name: name.trim(),
        email: email.trim() || null,
        phone: phone.trim() || null,
        address: address.trim() || null,
        notes: notes.trim() || null,
      };
      const updated = await clientsApi.update(payload);
      onSaved(updated);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form
      onSubmit={onSubmit}
      className="grid grid-cols-1 gap-3 rounded-card border border-border bg-surface p-5 shadow-card sm:grid-cols-2"
    >
      <Input
        label={t("common.name") ?? ""}
        value={name}
        onChange={(e) => setName(e.target.value)}
        required
      />
      <Input
        label={t("common.email") ?? ""}
        type="email"
        value={email}
        onChange={(e) => setEmail(e.target.value)}
      />
      <Input
        label={t("common.phone") ?? ""}
        value={phone}
        onChange={(e) => setPhone(e.target.value)}
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
        className="sm:col-span-2"
      />
      {err ? (
        <p className="sm:col-span-2 text-sm text-danger">{err}</p>
      ) : null}
      <div className="sm:col-span-2 flex items-center gap-3">
        <Button type="submit" disabled={busy}>
          {t("common.save")}
        </Button>
        {saved ? (
          <span className="text-sm text-success">{t("settings.saved")}</span>
        ) : null}
      </div>
    </form>
  );
}

// ---- Notebook tab ----

function NotebookTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<ClientNotebookView | null>(null);
  const [contents, setContents] = useState<Record<string, string>>({});
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = () => {
    setErr(null);
    notebookApi
      .getClientNotebook(clientId)
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
      await notebookApi.saveClientNotebook({
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
  | { mode: "edit"; entry: ClientJournalEntry };

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function JournalTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<ClientJournalEntry[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [editor, setEditor] = useState<JournalEditorState>({ mode: "closed" });

  const refresh = () => {
    setErr(null);
    notebookApi
      .listJournal(clientId)
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
      await notebookApi.deleteJournalEntry(id);
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
  initial: ClientJournalEntry | null;
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
        const payload: UpdateJournalEntryInput = {
          id: initial.id,
          entry_date: date,
          content,
        };
        await notebookApi.updateJournalEntry(payload);
      } else {
        const payload: NewJournalEntry = {
          client_id: clientId,
          entry_date: date,
          content,
        };
        await notebookApi.createJournalEntry(payload);
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
