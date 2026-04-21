import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { open, save } from "@tauri-apps/plugin-dialog";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { Money } from "../lib/money";
import { useCurrencyCatalogStore } from "../stores/currencyCatalogStore";
import { useEmailTemplateStore } from "../stores/emailTemplateStore";
import { useNotebookSectionStore } from "../stores/notebookSectionStore";
import { useSettingsStore } from "../stores/settingsStore";
import {
  ipc,
  type AppPreferencesDto,
  type CurrencyConfigDto,
  type EmailConfigDto,
  type EmailTemplateDto,
  type EmailTemplateTypeDto,
  type LanguageDto,
  type SellerProfileDto,
  type ThemeDto,
} from "../ipc";

const languageToI18n = (lang: LanguageDto): string =>
  lang === "Fr" ? "fr" : "en";

export function Settings() {
  const { t, i18n } = useTranslation();
  const {
    snapshot,
    load,
    loading,
    error,
    saveSeller,
    saveCurrency,
    savePreferences,
    saveEmailConfig,
    saveEmailPassword,
    testEmailConnection,
  } = useSettingsStore();

  useEffect(() => {
    void load();
  }, [load]);

  if (loading && !snapshot) {
    return <p className="text-sm text-fg-muted">{t("common.loading")}</p>;
  }
  if (!snapshot) {
    return error ? <p className="text-sm text-danger">{error}</p> : null;
  }

  return (
    <div className="max-w-3xl space-y-10">
      <h1 className="text-2xl font-bold text-fg">{t("settings.title")}</h1>

      <SellerSection seller={snapshot.seller} onSave={saveSeller} />
      <CurrencySection currency={snapshot.currency} onSave={saveCurrency} />
      <EmailSection
        config={snapshot.email}
        hasPassword={snapshot.has_email_password}
        onSaveConfig={saveEmailConfig}
        onSavePassword={saveEmailPassword}
        onTest={testEmailConnection}
      />
      <EmailTemplatesSection />
      <NotebookSectionsSection />
      <PreferencesSection
        prefs={snapshot.preferences}
        onSave={async (p) => {
          await savePreferences(p);
          await i18n.changeLanguage(languageToI18n(p.language));
        }}
      />
      <DataSection />
    </div>
  );
}

function DataSection() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<
    | { kind: "idle" }
    | { kind: "ok"; message: string }
    | { kind: "err"; message: string }
  >({ kind: "idle" });
  const [busy, setBusy] = useState<"export" | "backup" | "restore" | null>(null);

  const flash = (kind: "ok" | "err", message: string) => {
    setStatus({ kind, message });
    if (kind === "ok") {
      setTimeout(() => setStatus({ kind: "idle" }), 4000);
    }
  };

  const runExport = async () => {
    setBusy("export");
    try {
      const stamp = new Date().toISOString().slice(0, 10);
      const destination = await save({
        title: t("settings.data_export"),
        defaultPath: `terative-${stamp}.sqlite`,
        filters: [{ name: "SQLite", extensions: ["sqlite"] }],
      });
      if (!destination) {
        setBusy(null);
        return;
      }
      const written = await ipc.dataExport(destination);
      flash("ok", t("settings.data_exported_to", { path: written }));
    } catch (e) {
      flash("err", String(e));
    } finally {
      setBusy(null);
    }
  };

  const runBackup = async () => {
    setBusy("backup");
    try {
      const path = await ipc.dataBackup(null);
      flash("ok", t("settings.data_backed_up_to", { path }));
    } catch (e) {
      flash("err", String(e));
    } finally {
      setBusy(null);
    }
  };

  const runRestore = async () => {
    setBusy("restore");
    try {
      const source = await open({
        title: t("settings.data_restore"),
        multiple: false,
        directory: false,
        filters: [{ name: "SQLite", extensions: ["sqlite"] }],
      });
      if (!source || Array.isArray(source)) {
        setBusy(null);
        return;
      }
      if (!confirm(t("settings.data_restore_warning"))) {
        setBusy(null);
        return;
      }
      await ipc.dataRestore(source);
      flash("ok", t("settings.data_restored"));
    } catch (e) {
      flash("err", String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.data")}
      </h2>
      <p className="mb-3 text-sm text-fg-muted">{t("settings.data_help")}</p>
      <div className="flex flex-wrap gap-2">
        <Button onClick={runExport} disabled={busy !== null}>
          {t("settings.data_export")}
        </Button>
        <Button variant="secondary" onClick={runBackup} disabled={busy !== null}>
          {t("settings.data_backup")}
        </Button>
        <Button variant="secondary" onClick={runRestore} disabled={busy !== null}>
          {t("settings.data_restore")}
        </Button>
      </div>
      {status.kind === "ok" ? (
        <p className="mt-3 text-sm text-success break-all">{status.message}</p>
      ) : null}
      {status.kind === "err" ? (
        <p className="mt-3 text-sm text-danger break-all">{status.message}</p>
      ) : null}
    </section>
  );
}

interface SellerProps {
  seller: SellerProfileDto;
  onSave: (s: SellerProfileDto) => Promise<void>;
}

function SellerSection({ seller, onSave }: SellerProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<SellerProfileDto>(seller);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setForm(seller);
  }, [seller]);

  const update = <K extends keyof SellerProfileDto>(key: K, value: SellerProfileDto[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.seller")}
      </h2>
      <form
        className="grid grid-cols-1 gap-3 sm:grid-cols-2"
        onSubmit={async (e) => {
          e.preventDefault();
          await onSave(form);
          setSaved(true);
          setTimeout(() => setSaved(false), 1500);
        }}
      >
        <Input
          label={t("settings.seller_name") ?? ""}
          value={form.name}
          onChange={(e) => update("name", e.target.value)}
          required
        />
        <Input
          label={t("settings.seller_title") ?? ""}
          value={form.title ?? ""}
          onChange={(e) => update("title", e.target.value || null)}
        />
        <Input
          label={t("settings.seller_registration_id") ?? ""}
          value={form.registration_id ?? ""}
          onChange={(e) => update("registration_id", e.target.value || null)}
        />
        <Input
          label={t("common.email") ?? ""}
          type="email"
          value={form.email ?? ""}
          onChange={(e) => update("email", e.target.value || null)}
        />
        <Input
          label={t("common.phone") ?? ""}
          value={form.phone ?? ""}
          onChange={(e) => update("phone", e.target.value || null)}
        />
        <Input
          label={t("common.address") ?? ""}
          value={form.address ?? ""}
          onChange={(e) => update("address", e.target.value || null)}
        />
        <div className="sm:col-span-2 flex items-center gap-3">
          <Button type="submit">{t("common.save")}</Button>
          {saved ? (
            <span className="text-sm text-success">{t("settings.saved")}</span>
          ) : null}
        </div>
      </form>
    </section>
  );
}

interface CurrencyProps {
  currency: CurrencyConfigDto;
  onSave: (code: string) => Promise<void>;
}

function CurrencySection({ currency, onSave }: CurrencyProps) {
  const { t } = useTranslation();
  const { all, load: loadCatalog } = useCurrencyCatalogStore();
  const [code, setCode] = useState<string>(currency.code);
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    void loadCatalog();
  }, [loadCatalog]);

  useEffect(() => {
    setCode(currency.code);
  }, [currency.code]);

  const selected = all.find((c) => c.code === code) ?? currency;
  const sampleMinor =
    selected.fraction_digits === 0 ? BigInt(1000) : BigInt(123456);
  const sampleMoney = new Money(sampleMinor, selected.code);

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.currency")}
      </h2>
      <form
        className="flex flex-col gap-3"
        onSubmit={async (e) => {
          e.preventDefault();
          setErr(null);
          try {
            await onSave(code);
            setSaved(true);
            setTimeout(() => setSaved(false), 1500);
          } catch (e) {
            setErr(String(e));
          }
        }}
      >
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("settings.currency")}
          <select
            className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={code}
            onChange={(e) => setCode(e.target.value)}
          >
            {all.map((c) => (
              <option key={c.code} value={c.code}>
                {c.code} · {c.name}
                {c.fraction_digits === 0 ? ` (${t("settings.no_decimals")})` : ""}
              </option>
            ))}
          </select>
        </label>
        <p className="text-xs text-fg-subtle">
          {t("settings.currency_sample")}: {sampleMoney.format(selected)}
        </p>
        <div className="flex items-center gap-3">
          <Button type="submit">{t("common.save")}</Button>
          {saved ? (
            <span className="text-sm text-success">{t("settings.saved")}</span>
          ) : null}
          {err ? <span className="text-sm text-danger">{err}</span> : null}
        </div>
      </form>
    </section>
  );
}

interface PreferencesProps {
  prefs: AppPreferencesDto;
  onSave: (p: AppPreferencesDto) => Promise<void>;
}

function PreferencesSection({ prefs, onSave }: PreferencesProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<AppPreferencesDto>(prefs);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setForm(prefs);
  }, [prefs]);

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.preferences")}
      </h2>
      <form
        className="grid grid-cols-1 gap-3 sm:grid-cols-2"
        onSubmit={async (e) => {
          e.preventDefault();
          await onSave(form);
          setSaved(true);
          setTimeout(() => setSaved(false), 1500);
        }}
      >
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("settings.theme")}
          <select
            className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={form.theme}
            onChange={(e) =>
              setForm({ ...form, theme: e.target.value as ThemeDto })
            }
          >
            <option value="Light">{t("settings.light")}</option>
            <option value="Dark">{t("settings.dark")}</option>
          </select>
        </label>
        <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
          {t("settings.language")}
          <select
            className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
            value={form.language}
            onChange={(e) =>
              setForm({ ...form, language: e.target.value as LanguageDto })
            }
          >
            <option value="Fr">Français</option>
            <option value="En">English</option>
          </select>
        </label>
        <Input
          label={t("settings.pdf_output_dir") ?? ""}
          value={form.pdf_output_dir}
          onChange={(e) => setForm({ ...form, pdf_output_dir: e.target.value })}
          className="sm:col-span-2"
        />
        <div className="sm:col-span-2 flex items-center gap-3">
          <Button type="submit">{t("common.save")}</Button>
          {saved ? (
            <span className="text-sm text-success">{t("settings.saved")}</span>
          ) : null}
        </div>
      </form>
    </section>
  );
}

interface EmailProps {
  config: EmailConfigDto;
  hasPassword: boolean;
  onSaveConfig: (c: EmailConfigDto) => Promise<void>;
  onSavePassword: (p: string) => Promise<void>;
  onTest: () => Promise<void>;
}

function EmailSection({
  config,
  hasPassword,
  onSaveConfig,
  onSavePassword,
  onTest,
}: EmailProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<EmailConfigDto>(config);
  const [password, setPassword] = useState("");
  const [saved, setSaved] = useState(false);
  const [pwSaved, setPwSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [testState, setTestState] = useState<"idle" | "running" | "ok" | "err">(
    "idle",
  );

  useEffect(() => {
    setForm(config);
  }, [config]);

  const update = <K extends keyof EmailConfigDto>(key: K, value: EmailConfigDto[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  const saveConfig = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    try {
      await onSaveConfig(form);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      setErr(String(e));
    }
  };

  const savePassword = async () => {
    setErr(null);
    try {
      await onSavePassword(password);
      setPassword("");
      setPwSaved(true);
      setTimeout(() => setPwSaved(false), 1500);
    } catch (e) {
      setErr(String(e));
    }
  };

  const runTest = async () => {
    setTestState("running");
    setErr(null);
    try {
      await onTest();
      setTestState("ok");
      setTimeout(() => setTestState("idle"), 2500);
    } catch (e) {
      setTestState("err");
      setErr(String(e));
    }
  };

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.email")}
      </h2>
      <form
        className="grid grid-cols-1 gap-3 sm:grid-cols-2"
        onSubmit={saveConfig}
      >
        <Input
          label={t("settings.smtp_host") ?? ""}
          value={form.smtp_host}
          onChange={(e) => update("smtp_host", e.target.value)}
          placeholder="smtp.example.com"
        />
        <Input
          label={t("settings.smtp_port") ?? ""}
          type="number"
          min="1"
          max="65535"
          value={form.smtp_port}
          onChange={(e) => update("smtp_port", parseInt(e.target.value) || 0)}
        />
        <Input
          label={t("settings.sender_address") ?? ""}
          type="email"
          value={form.sender_address}
          onChange={(e) => update("sender_address", e.target.value)}
          className="sm:col-span-2"
          placeholder="you@example.com"
        />

        <div className="sm:col-span-2 flex items-center gap-3">
          <Button type="submit">{t("common.save")}</Button>
          {saved ? (
            <span className="text-sm text-success">{t("settings.saved")}</span>
          ) : null}
        </div>
      </form>

      <div className="mt-4 grid grid-cols-1 gap-3 border-t border-border pt-4 sm:grid-cols-2">
        <Input
          label={t("settings.smtp_password") ?? ""}
          type="password"
          value={password}
          placeholder={
            hasPassword
              ? (t("settings.password_stored") ?? "••••••••")
              : (t("settings.password_unset") ?? "")
          }
          onChange={(e) => setPassword(e.target.value)}
        />
        <div className="flex items-end gap-2">
          <Button
            type="button"
            onClick={savePassword}
            disabled={password.length === 0}
          >
            {hasPassword
              ? t("settings.update_password")
              : t("settings.save_password")}
          </Button>
          {hasPassword ? (
            <Button
              variant="secondary"
              type="button"
              onClick={() => {
                void onSavePassword("");
              }}
            >
              {t("common.delete")}
            </Button>
          ) : null}
          {pwSaved ? (
            <span className="text-sm text-success">{t("settings.saved")}</span>
          ) : null}
        </div>
        <div className="sm:col-span-2 flex items-center gap-3">
          <Button
            variant="secondary"
            type="button"
            onClick={runTest}
            disabled={testState === "running"}
          >
            {testState === "running"
              ? t("common.loading")
              : t("settings.test_connection")}
          </Button>
          {testState === "ok" ? (
            <span className="text-sm text-success">
              {t("settings.test_ok")}
            </span>
          ) : null}
        </div>
        {err ? <p className="sm:col-span-2 text-sm text-danger">{err}</p> : null}
      </div>
    </section>
  );
}

const PLACEHOLDER_KEYS = [
  "number",
  "client_name",
  "date",
  "due_date",
  "total",
  "subtotal",
  "seller_name",
  "currency_code",
];

function EmailTemplatesSection() {
  const { t } = useTranslation();
  const { templates, loading, refresh, create, update, remove, setDefault } =
    useEmailTemplateStore();
  const [editing, setEditing] = useState<EmailTemplateDto | null>(null);
  const [creating, setCreating] = useState<EmailTemplateTypeDto | null>(null);
  const [form, setForm] = useState({ name: "", subject_template: "", body_template: "" });
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const startEdit = (tmpl: EmailTemplateDto) => {
    setEditing(tmpl);
    setCreating(null);
    setForm({
      name: tmpl.name,
      subject_template: tmpl.subject_template,
      body_template: tmpl.body_template,
    });
    setErr(null);
  };

  const startCreate = (templateType: EmailTemplateTypeDto) => {
    setCreating(templateType);
    setEditing(null);
    setForm({ name: "", subject_template: "", body_template: "" });
    setErr(null);
  };

  const cancelEdit = () => {
    setEditing(null);
    setCreating(null);
    setErr(null);
  };

  const save = async () => {
    setErr(null);
    try {
      if (editing) {
        await update({
          id: editing.id,
          name: form.name,
          subject_template: form.subject_template,
          body_template: form.body_template,
        });
      } else if (creating) {
        await create({
          name: form.name,
          template_type: creating,
          subject_template: form.subject_template,
          body_template: form.body_template,
        });
      }
      cancelEdit();
    } catch (e) {
      setErr(String(e));
    }
  };

  const typeLabel = (t_type: EmailTemplateTypeDto) =>
    t_type === "InitialContact"
      ? t("email_templates.initial_contact")
      : t("email_templates.follow_up");

  const renderGroup = (templateType: EmailTemplateTypeDto) => {
    const group = templates.filter((tmpl) => tmpl.template_type === templateType);
    return (
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <h3 className="text-sm font-semibold text-fg">{typeLabel(templateType)}</h3>
          <Button variant="secondary" onClick={() => startCreate(templateType)}>
            {t("common.add")}
          </Button>
        </div>
        {group.length === 0 ? (
          <p className="text-sm text-fg-subtle">{t("email_templates.none")}</p>
        ) : (
          <ul className="space-y-1">
            {group.map((tmpl) => (
              <li
                key={tmpl.id}
                className="flex items-center justify-between rounded-field border border-border bg-surface px-3 py-2"
              >
                <div className="flex items-center gap-2">
                  <span className="text-sm text-fg">{tmpl.name}</span>
                  {tmpl.is_default ? (
                    <span className="rounded-field bg-accent/10 px-1.5 py-0.5 text-xs font-medium text-accent">
                      {t("email_templates.default_badge")}
                    </span>
                  ) : null}
                </div>
                <div className="flex items-center gap-1">
                  {!tmpl.is_default ? (
                    <Button
                      variant="secondary"
                      onClick={() => void setDefault(tmpl.id).catch((e) => alert(String(e)))}
                    >
                      {t("email_templates.set_default")}
                    </Button>
                  ) : null}
                  <Button variant="secondary" onClick={() => startEdit(tmpl)}>
                    {t("common.edit")}
                  </Button>
                  {!tmpl.is_default ? (
                    <Button
                      variant="danger"
                      onClick={() => void remove(tmpl.id).catch((e) => alert(String(e)))}
                    >
                      {t("common.delete")}
                    </Button>
                  ) : null}
                </div>
              </li>
            ))}
          </ul>
        )}
      </div>
    );
  };

  if (loading) return <p>{t("common.loading")}</p>;

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("email_templates.title")}
      </h2>
      <div className="space-y-4">
        {renderGroup("InitialContact")}
        {renderGroup("FollowUp")}
      </div>

      {(editing || creating) ? (
        <div className="mt-4 space-y-3 rounded-field border border-border bg-surface-muted p-4">
          <h3 className="text-sm font-semibold text-fg">
            {editing ? t("common.edit") : t("common.add")}
          </h3>
          <Input
            label={t("email_templates.name") ?? ""}
            value={form.name}
            onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
          />
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("email_templates.subject")}
            <input
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={form.subject_template}
              onChange={(e) => setForm((f) => ({ ...f, subject_template: e.target.value }))}
              placeholder="Invoice {{number}} from {{seller_name}}"
            />
          </label>
          <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
            {t("email_templates.body")}
            <textarea
              className="block min-h-32 w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
              value={form.body_template}
              onChange={(e) => setForm((f) => ({ ...f, body_template: e.target.value }))}
            />
          </label>
          <p className="text-xs text-fg-subtle">
            {t("email_templates.placeholders_help")}{" "}
            {PLACEHOLDER_KEYS.map((k) => (
              <code
                key={k}
                className="mx-0.5 rounded-field bg-surface-muted px-1 py-0.5 text-fg-muted"
              >{`{{${k}}}`}</code>
            ))}
          </p>
          <div className="flex items-center gap-2">
            <Button onClick={save}>{t("common.save")}</Button>
            <Button variant="secondary" onClick={cancelEdit}>
              {t("common.cancel")}
            </Button>
          </div>
          {err ? <p className="text-sm text-danger">{err}</p> : null}
        </div>
      ) : null}
    </section>
  );
}

function NotebookSectionsSection() {
  const { t } = useTranslation();
  const { sections, loading, error, refresh, create, rename, remove, reorder } =
    useNotebookSectionStore();
  const [newName, setNewName] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editingName, setEditingName] = useState("");
  const [busy, setBusy] = useState(false);
  const [localErr, setLocalErr] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const onCreate = async () => {
    const trimmed = newName.trim();
    if (!trimmed) return;
    setBusy(true);
    setLocalErr(null);
    try {
      await create(trimmed);
      setNewName("");
    } catch (e) {
      setLocalErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onStartEdit = (id: string, name: string) => {
    setEditingId(id);
    setEditingName(name);
  };

  const onCommitEdit = async () => {
    if (!editingId) return;
    const trimmed = editingName.trim();
    if (!trimmed) {
      setEditingId(null);
      return;
    }
    setBusy(true);
    setLocalErr(null);
    try {
      await rename(editingId, trimmed);
      setEditingId(null);
    } catch (e) {
      setLocalErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const onDelete = async (id: string, name: string) => {
    let count = 0;
    try {
      count = await ipc.notebookSectionCountEntries(id);
    } catch {
      // Fall back to a generic warning if the count fails.
    }
    const msg =
      count > 0
        ? t("settings.notebook_delete_warning_with_count", {
            name,
            count,
          })
        : t("settings.notebook_delete_warning", { name });
    if (!confirm(msg)) return;
    setBusy(true);
    setLocalErr(null);
    try {
      await remove(id);
    } catch (e) {
      setLocalErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  const move = async (index: number, delta: -1 | 1) => {
    const target = index + delta;
    if (target < 0 || target >= sections.length) return;
    const ordered = sections.map((s) => s.id);
    [ordered[index], ordered[target]] = [ordered[target], ordered[index]];
    setBusy(true);
    setLocalErr(null);
    try {
      await reorder(ordered);
    } catch (e) {
      setLocalErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.notebook_sections")}
      </h2>
      <p className="mb-3 text-sm text-fg-muted">
        {t("settings.notebook_sections_help")}
      </p>

      {error ? (
        <p className="mb-3 text-sm text-danger">{error}</p>
      ) : null}
      {localErr ? (
        <p className="mb-3 text-sm text-danger">{localErr}</p>
      ) : null}

      {loading && sections.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : (
        <ul className="mb-3 flex flex-col gap-2">
          {sections.map((s, i) => (
            <li
              key={s.id}
              className="flex items-center gap-2 rounded-field border border-border bg-surface p-2"
            >
              <div className="flex flex-col">
                <button
                  type="button"
                  onClick={() => move(i, -1)}
                  disabled={busy || i === 0}
                  className="text-xs text-fg-muted hover:text-fg disabled:opacity-30"
                  aria-label={t("settings.move_up") ?? ""}
                >
                  ▲
                </button>
                <button
                  type="button"
                  onClick={() => move(i, 1)}
                  disabled={busy || i === sections.length - 1}
                  className="text-xs text-fg-muted hover:text-fg disabled:opacity-30"
                  aria-label={t("settings.move_down") ?? ""}
                >
                  ▼
                </button>
              </div>
              {editingId === s.id ? (
                <input
                  className="flex-1 rounded-field border border-border bg-surface px-2 py-1 text-sm text-fg"
                  value={editingName}
                  onChange={(e) => setEditingName(e.target.value)}
                  onBlur={onCommitEdit}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") void onCommitEdit();
                    if (e.key === "Escape") setEditingId(null);
                  }}
                  autoFocus
                />
              ) : (
                <button
                  type="button"
                  onClick={() => onStartEdit(s.id, s.name)}
                  className="flex-1 text-left text-sm font-medium text-fg"
                >
                  {s.name}
                </button>
              )}
              <Button
                variant="danger"
                onClick={() => void onDelete(s.id, s.name)}
                disabled={busy}
              >
                {t("common.delete")}
              </Button>
            </li>
          ))}
        </ul>
      )}

      <div className="flex items-end gap-2">
        <Input
          label={t("settings.new_section") ?? ""}
          value={newName}
          onChange={(e) => setNewName(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") void onCreate();
          }}
          placeholder={t("settings.new_section_placeholder") ?? ""}
        />
        <Button onClick={onCreate} disabled={busy || newName.trim() === ""}>
          {t("common.create")}
        </Button>
      </div>
    </section>
  );
}
