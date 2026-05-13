import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";

import { open } from "@tauri-apps/plugin-dialog";

import { OrgPasswordSection } from "../components/org/OrgPasswordSection";
import { Page } from "../components/layout/Page";
import { Button } from "../components/common/Button";
import { ConfirmModal } from "../components/ui/ConfirmModal";
import { Modal } from "../components/ui/Modal";
import { ImageUploader } from "../components/common/ImageUploader";
import { Input } from "../components/common/Input";
import { Money } from "../lib/money";
import { useBookmarkStore } from "../stores/bookmarkStore";
import { useCurrencyCatalogStore } from "../stores/currencyCatalogStore";
import { useNotebookSectionStore } from "../stores/notebookSectionStore";
import { useSettingsStore } from "../stores/settingsStore";
import { errorCodeOf, translateError } from "../ipc/errorCatalog";
import {
  ipc,
  type AppPreferencesDto,
  type BackupDto,
  type BackupKindDto,
  type BackupScopeDto,
  type CurrencyConfigDto,
  type EmailConfigDto,
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
    return (
      <Page crumbs={[t("settings.title")]} title={t("settings.title")}>
        <p className="text-[13px] text-ink-3">{t("common.loading")}</p>
      </Page>
    );
  }
  if (!snapshot) {
    return (
      <Page crumbs={[t("settings.title")]} title={t("settings.title")}>
        {error ? <p className="text-[13px] text-danger">{error}</p> : null}
      </Page>
    );
  }

  return (
    <Page
      crumbs={[t("settings.title")]}
      title={t("settings.title")}
      subtitle={t("settings.subtitle")}
    >
      <div className="max-w-3xl space-y-10">
      <SellerSection seller={snapshot.seller} onSave={saveSeller} />
      <CurrencySection currency={snapshot.currency} onSave={saveCurrency} />
      <EmailSection
        config={snapshot.email}
        hasPassword={snapshot.has_email_password}
        onSaveConfig={saveEmailConfig}
        onSavePassword={saveEmailPassword}
        onTest={testEmailConnection}
      />
      <BookmarksSection />
      <NotebookSectionsSection />
      <PreferencesSection
        prefs={snapshot.preferences}
        onSave={async (p) => {
          await savePreferences(p);
          await i18n.changeLanguage(languageToI18n(p.language));
        }}
      />
      <OrgPasswordSection />
      <DataSection />
      {import.meta.env.DEV ? <DeveloperSection /> : null}
      </div>
    </Page>
  );
}

function DataSection() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<
    | { kind: "idle" }
    | { kind: "ok"; message: string }
    | { kind: "err"; message: string }
  >({ kind: "idle" });
  const [busy, setBusy] = useState<"backup" | "restore" | "delete" | null>(null);
  const [backups, setBackups] = useState<BackupDto[]>([]);
  const [restoreTarget, setRestoreTarget] = useState<string | null>(null);
  /** Path of the encrypted backup awaiting a password in the prompt modal. */
  const [pendingPasswordFor, setPendingPasswordFor] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  const flash = (kind: "ok" | "err", message: string) => {
    setStatus({ kind, message });
    if (kind === "ok") {
      setTimeout(() => setStatus({ kind: "idle" }), 4000);
    }
  };

  const loadBackups = async () => {
    try {
      setBackups(await ipc.dataListBackups());
    } catch (e) {
      flash("err", String(e));
    }
  };

  useEffect(() => {
    void loadBackups();
  }, []);

  const runBackup = async () => {
    setBusy("backup");
    try {
      const path = await ipc.dataBackup();
      flash("ok", t("settings.data_backed_up_to", { path }));
      await loadBackups();
    } catch (e) {
      flash("err", String(e));
    } finally {
      setBusy(null);
    }
  };

  const requestRestore = (source: string) => {
    setRestoreTarget(source);
  };

  /** Called after the user has confirmed and (if needed) supplied a
   *  password. The Tauri app restarts on success. */
  const performRestore = async (source: string, sourcePassword: string | null) => {
    setBusy("restore");
    try {
      await ipc.dataRestore(source, sourcePassword);
      // App restarts on success, so nothing else to do here.
    } catch (e) {
      flash("err", String(e));
      setBusy(null);
      throw e;
    }
  };

  /** Branches on whether the source is SQLCipher-encrypted: plaintext goes
   *  straight to `performRestore`; encrypted opens a password modal first. */
  const startRestore = async (source: string) => {
    let encrypted = false;
    try {
      encrypted = await ipc.dataSourceAppearsEncrypted(source);
    } catch {
      // Fall through with `encrypted = false`; the restore adapter will
      // surface a clean error if the file is unreadable.
    }
    if (encrypted) {
      setPendingPasswordFor(source);
      return;
    }
    await performRestore(source, null);
  };

  const runRestoreFromPicker = async () => {
    const source = await open({
      title: t("settings.data_restore"),
      multiple: false,
      directory: false,
      filters: [{ name: "SQLite", extensions: ["sqlite"] }],
    });
    if (!source || Array.isArray(source)) return;
    requestRestore(source);
  };

  const requestDelete = (path: string) => {
    setDeleteTarget(path);
  };

  const performDelete = async (path: string) => {
    setBusy("delete");
    try {
      await ipc.dataDeleteBackup(path);
      flash("ok", t("settings.backup_deleted"));
      await loadBackups();
    } catch (e) {
      flash("err", String(e));
      throw e;
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
        <Button onClick={runBackup} disabled={busy !== null}>
          {t("settings.data_backup")}
        </Button>
        <Button
          variant="secondary"
          onClick={runRestoreFromPicker}
          disabled={busy !== null}
        >
          {t("settings.data_restore")}
        </Button>
      </div>
      {status.kind === "ok" ? (
        <p className="mt-3 text-sm text-success break-all">{status.message}</p>
      ) : null}
      {status.kind === "err" ? (
        <p className="mt-3 text-sm text-danger break-all">{status.message}</p>
      ) : null}

      <BackupHistory
        backups={backups}
        busy={busy !== null}
        onRestore={requestRestore}
        onDelete={requestDelete}
      />

      <ConfirmModal
        open={restoreTarget !== null}
        title={t("settings.data_restore")}
        description={t("settings.data_restore_warning")}
        confirmLabel={t("settings.data_restore")}
        tone="danger"
        requireText={t("settings.confirm_restore_phrase")}
        onConfirm={async () => {
          if (restoreTarget) await startRestore(restoreTarget);
        }}
        onClose={() => setRestoreTarget(null)}
      />

      <BackupPasswordModal
        source={pendingPasswordFor}
        onClose={() => setPendingPasswordFor(null)}
        onSubmit={async (password) => {
          if (!pendingPasswordFor) return;
          setBusy("restore");
          try {
            await ipc.dataRestore(pendingPasswordFor, password);
            // App restarts on success — nothing more to do.
          } catch (e) {
            toast.error(translateError(e, t));
            setBusy(null);
            if (errorCodeOf(e) === "restore_wrong_password") {
              // Re-throw so the modal stays open and clears the input.
              throw e;
            }
            // Any other failure is not retry-able from this modal.
            setPendingPasswordFor(null);
          }
        }}
      />

      <ConfirmModal
        open={deleteTarget !== null}
        title={t("common.delete")}
        description={t("settings.backup_delete_confirm")}
        confirmLabel={t("common.delete")}
        tone="danger"
        requireText={t("settings.confirm_delete_phrase")}
        onConfirm={async () => {
          if (deleteTarget) await performDelete(deleteTarget);
        }}
        onClose={() => setDeleteTarget(null)}
      />
    </section>
  );
}

interface BackupPasswordModalProps {
  /** Path of the encrypted backup to unlock; modal is hidden when `null`. */
  source: string | null;
  onClose: () => void;
  onSubmit: (password: string) => void | Promise<void>;
}

function BackupPasswordModal({ source, onClose, onSubmit }: BackupPasswordModalProps) {
  const { t } = useTranslation();
  const [password, setPassword] = useState("");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (source === null) {
      setPassword("");
      setSubmitting(false);
    }
  }, [source]);

  if (source === null) return null;

  return (
    <Modal
      open
      onClose={onClose}
      title={t("settings.restore_password_title", {
        defaultValue: "Unlock backup",
      })}
    >
      <form
        onSubmit={async (e) => {
          e.preventDefault();
          if (!password) return;
          setSubmitting(true);
          try {
            await onSubmit(password);
          } catch {
            // Parent already toasted; we just clear the field so the user
            // can retry without re-opening the modal.
            setPassword("");
          } finally {
            setSubmitting(false);
          }
        }}
        className="space-y-4"
      >
        <p className="text-sm text-fg-muted">
          {t("settings.restore_password_help", {
            defaultValue:
              "This backup is encrypted. Enter the password it was created with.",
          })}
        </p>
        <div className="space-y-1.5">
          <label htmlFor="backup-password" className="text-sm font-medium">
            {t("org_unlock.password_label", { defaultValue: "Password" })}
          </label>
          <Input
            id="backup-password"
            type="password"
            autoFocus
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.currentTarget.value)}
            required
          />
        </div>
        <div className="flex justify-end gap-2">
          <Button type="button" variant="secondary" onClick={onClose}>
            {t("common.cancel", { defaultValue: "Cancel" })}
          </Button>
          <Button type="submit" disabled={submitting || !password}>
            {submitting
              ? t("org_unlock.submitting", { defaultValue: "Unlocking…" })
              : t("settings.data_restore", { defaultValue: "Restore" })}
          </Button>
        </div>
      </form>
    </Modal>
  );
}

interface BackupHistoryProps {
  backups: BackupDto[];
  busy: boolean;
  onRestore: (path: string) => void | Promise<void>;
  onDelete: (path: string) => void | Promise<void>;
}

function BackupHistory({ backups, busy, onRestore, onDelete }: BackupHistoryProps) {
  const { t, i18n } = useTranslation();
  if (backups.length === 0) {
    return (
      <p className="mt-6 text-sm text-fg-muted">{t("settings.backups_none")}</p>
    );
  }
  const dateFmt = new Intl.DateTimeFormat(i18n.language, {
    dateStyle: "medium",
    timeStyle: "short",
  });
  const kindLabel = (k: BackupKindDto): string =>
    t(`settings.backup_kind_${k.toLowerCase()}`);
  const scopeLabel = (s: BackupScopeDto): string =>
    t(`settings.backup_scope_${s.toLowerCase()}`);
  return (
    <div className="mt-6">
      <h3 className="mb-2 text-sm font-semibold text-fg">
        {t("settings.backup_history")}
      </h3>
      <table className="w-full border-collapse text-sm">
        <thead>
          <tr className="border-b border-border text-left text-fg-muted">
            <th className="py-2 pr-3 font-medium">{t("settings.backup_when")}</th>
            <th className="py-2 pr-3 font-medium">{t("settings.backup_kind")}</th>
            <th className="py-2 pr-3 font-medium">{t("settings.backup_scope")}</th>
            <th className="py-2 pr-3 font-medium">{t("settings.backup_size")}</th>
            <th className="py-2 pr-3 font-medium"></th>
          </tr>
        </thead>
        <tbody>
          {backups.map((b) => (
            <tr key={b.path} className="border-b border-border">
              <td className="py-2 pr-3 text-fg" title={b.path}>
                {dateFmt.format(new Date(b.timestamp))}
              </td>
              <td className="py-2 pr-3 text-fg-muted">{kindLabel(b.kind)}</td>
              <td className="py-2 pr-3 text-fg-muted">{scopeLabel(b.scope)}</td>
              <td className="py-2 pr-3 text-fg-muted">
                {formatBytes(b.size_bytes)}
              </td>
              <td className="flex justify-end gap-2 py-2 pr-3">
                <Button
                  variant="secondary"
                  disabled={busy}
                  onClick={() => void onRestore(b.path)}
                >
                  {t("settings.backup_restore_this")}
                </Button>
                {b.scope === "User" ? (
                  <Button
                    variant="danger"
                    disabled={busy}
                    onClick={() => void onDelete(b.path)}
                  >
                    {t("common.delete")}
                  </Button>
                ) : (
                  <span
                    className="text-xs text-fg-muted"
                    title={t("settings.backup_system_tooltip") ?? ""}
                  >
                    {t("settings.backup_system_locked")}
                  </span>
                )}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  if (n < 1024 * 1024 * 1024) return `${(n / (1024 * 1024)).toFixed(1)} MB`;
  return `${(n / (1024 * 1024 * 1024)).toFixed(2)} GB`;
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
        <div className="sm:col-span-2">
          <ImageUploader
            label={t("settings.seller_signature") ?? ""}
            value={form.signature_image}
            onChange={(bytes) => update("signature_image", bytes)}
          />
        </div>
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
  const { t, i18n } = useTranslation();
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
  const sampleMoney = new Money(sampleMinor, selected);

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
            toast.error(e);
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
          {t("settings.currency_sample")}:{" "}
          {sampleMoney.formatWithSymbol(i18n.language)}
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
        <Input
          label={t("settings.user_backup_dir") ?? ""}
          value={form.user_backup_dir}
          onChange={(e) => setForm({ ...form, user_backup_dir: e.target.value })}
          placeholder={t("settings.user_backup_dir_placeholder") ?? ""}
          className="sm:col-span-2"
        />
        <Input
          label={t("settings.default_invoice_due_days") ?? ""}
          type="number"
          min="0"
          max="365"
          value={form.default_invoice_due_days}
          onChange={(e) =>
            setForm({
              ...form,
              default_invoice_due_days: Math.max(
                0,
                parseInt(e.target.value, 10) || 0,
              ),
            })
          }
          placeholder="30"
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
      toast.error(e);
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
      toast.error(e);
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
      toast.error(e);
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
      toast.error(e);
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
      toast.error(e);
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
      toast.error(e);
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
      toast.error(e);
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

function BookmarksSection() {
  const { t } = useTranslation();
  const { bookmarks, loading, error, refresh, create, update, remove, reorder } =
    useBookmarkStore();
  const [newLabel, setNewLabel] = useState("");
  const [newUrl, setNewUrl] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editLabel, setEditLabel] = useState("");
  const [editUrl, setEditUrl] = useState("");
  const [busy, setBusy] = useState(false);
  const [localErr, setLocalErr] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const onCreate = async () => {
    const label = newLabel.trim();
    const url = newUrl.trim();
    if (!label || !url) return;
    setBusy(true);
    setLocalErr(null);
    try {
      await create({ label, url });
      setNewLabel("");
      setNewUrl("");
    } catch (e) {
      toast.error(e);
    } finally {
      setBusy(false);
    }
  };

  const onStartEdit = (id: string, label: string, url: string) => {
    setEditingId(id);
    setEditLabel(label);
    setEditUrl(url);
  };

  const onCommitEdit = async () => {
    if (!editingId) return;
    const label = editLabel.trim();
    const url = editUrl.trim();
    if (!label || !url) {
      setEditingId(null);
      return;
    }
    setBusy(true);
    setLocalErr(null);
    try {
      await update({ id: editingId, label, url });
      setEditingId(null);
    } catch (e) {
      toast.error(e);
    } finally {
      setBusy(false);
    }
  };

  const onDelete = async (id: string) => {
    if (!confirm(t("bookmarks.delete_confirm"))) return;
    setBusy(true);
    setLocalErr(null);
    try {
      await remove(id);
    } catch (e) {
      toast.error(e);
    } finally {
      setBusy(false);
    }
  };

  const move = async (index: number, delta: -1 | 1) => {
    const target = index + delta;
    if (target < 0 || target >= bookmarks.length) return;
    const ordered = bookmarks.map((b) => b.id);
    [ordered[index], ordered[target]] = [ordered[target], ordered[index]];
    setBusy(true);
    setLocalErr(null);
    try {
      await reorder(ordered);
    } catch (e) {
      toast.error(e);
    } finally {
      setBusy(false);
    }
  };

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.bookmarks")}
      </h2>
      <p className="mb-3 text-sm text-fg-muted">
        {t("settings.bookmarks_help")}
      </p>

      {error ? <p className="mb-3 text-sm text-danger">{error}</p> : null}
      {localErr ? <p className="mb-3 text-sm text-danger">{localErr}</p> : null}

      {loading && bookmarks.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : bookmarks.length === 0 ? (
        <p className="mb-3 text-sm text-fg-muted">{t("bookmarks.none")}</p>
      ) : (
        <ul className="mb-3 flex flex-col gap-2">
          {bookmarks.map((b, i) => (
            <li
              key={b.id}
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
                  disabled={busy || i === bookmarks.length - 1}
                  className="text-xs text-fg-muted hover:text-fg disabled:opacity-30"
                  aria-label={t("settings.move_down") ?? ""}
                >
                  ▼
                </button>
              </div>
              {editingId === b.id ? (
                <div className="flex flex-1 gap-2">
                  <input
                    className="w-1/3 rounded-field border border-border bg-surface px-2 py-1 text-sm text-fg"
                    value={editLabel}
                    onChange={(e) => setEditLabel(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void onCommitEdit();
                      if (e.key === "Escape") setEditingId(null);
                    }}
                    placeholder={t("bookmarks.label_placeholder") ?? ""}
                    autoFocus
                  />
                  <input
                    className="flex-1 rounded-field border border-border bg-surface px-2 py-1 text-sm text-fg"
                    value={editUrl}
                    onChange={(e) => setEditUrl(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") void onCommitEdit();
                      if (e.key === "Escape") setEditingId(null);
                    }}
                    placeholder={t("bookmarks.url_placeholder") ?? ""}
                  />
                  <Button
                    type="button"
                    onClick={() => void onCommitEdit()}
                    disabled={busy}
                  >
                    {t("common.save")}
                  </Button>
                  <Button
                    variant="secondary"
                    type="button"
                    onClick={() => setEditingId(null)}
                  >
                    {t("common.cancel")}
                  </Button>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => onStartEdit(b.id, b.label, b.url)}
                  className="flex flex-1 flex-col items-start gap-0.5 text-left"
                >
                  <span className="text-sm font-medium text-fg">{b.label}</span>
                  <span className="truncate text-xs text-fg-muted">
                    {b.url}
                  </span>
                </button>
              )}
              {editingId === b.id ? null : (
                <Button
                  variant="danger"
                  onClick={() => void onDelete(b.id)}
                  disabled={busy}
                >
                  {t("common.delete")}
                </Button>
              )}
            </li>
          ))}
        </ul>
      )}

      <div className="flex items-end gap-2">
        <Input
          label={t("bookmarks.label") ?? ""}
          value={newLabel}
          onChange={(e) => setNewLabel(e.target.value)}
          placeholder={t("bookmarks.label_placeholder") ?? ""}
          className="w-1/3"
        />
        <Input
          label={t("bookmarks.url") ?? ""}
          value={newUrl}
          onChange={(e) => setNewUrl(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") void onCreate();
          }}
          placeholder={t("bookmarks.url_placeholder") ?? ""}
          className="flex-1"
        />
        <Button
          onClick={onCreate}
          disabled={busy || newLabel.trim() === "" || newUrl.trim() === ""}
        >
          {t("bookmarks.add")}
        </Button>
      </div>
    </section>
  );
}

/// Development-only tools. Rendered only when Vite is in dev mode
/// (`import.meta.env.DEV`); the backend command is also gated on
/// `cfg(debug_assertions)` so it physically isn't in release binaries.
function DeveloperSection() {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<
    | { kind: "idle" }
    | { kind: "ok"; message: string }
    | { kind: "err"; message: string }
  >({ kind: "idle" });

  const onSeed = async () => {
    if (!confirm(t("settings.seed_confirm"))) return;
    setBusy(true);
    setStatus({ kind: "idle" });
    try {
      const report = await ipc.seedDatabase(null);
      setStatus({
        kind: "ok",
        message: t("settings.seed_done", {
          clients: report.clients_added,
          invoices_drafted: report.invoices_drafted,
          invoices_finalized: report.invoices_finalized,
          invoices_cancelled: report.invoices_cancelled,
          payments: report.payments_added,
          journal: report.journal_entries_added,
        }),
      });
    } catch (e) {
      setStatus({ kind: "err", message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg">
        {t("settings.developer")}
      </h2>
      <p className="mb-3 text-sm text-fg-muted">
        {t("settings.developer_help")}
      </p>
      <Button onClick={onSeed} disabled={busy}>
        {busy ? t("settings.seed_running") : t("settings.seed_database")}
      </Button>
      {status.kind === "ok" ? (
        <p className="mt-3 text-sm text-success">{status.message}</p>
      ) : null}
      {status.kind === "err" ? (
        <p className="mt-3 text-sm text-danger break-all">{status.message}</p>
      ) : null}
    </section>
  );
}
