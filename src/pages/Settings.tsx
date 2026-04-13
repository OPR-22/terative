import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { useSettingsStore } from "../stores/settingsStore";
import type {
  AppPreferences,
  CurrencyConfig,
  Language,
  SellerProfile,
  Theme,
} from "../types/settings";

export function Settings() {
  const { t, i18n } = useTranslation();
  const { snapshot, load, loading, error, saveSeller, saveCurrency, savePreferences } =
    useSettingsStore();

  useEffect(() => {
    void load();
  }, [load]);

  if (loading && !snapshot) {
    return <p className="text-sm text-zinc-600">{t("common.loading")}</p>;
  }
  if (!snapshot) {
    return error ? <p className="text-sm text-red-600">{error}</p> : null;
  }

  return (
    <div className="max-w-3xl space-y-10">
      <h1 className="text-2xl font-bold text-zinc-900">{t("settings.title")}</h1>

      <SellerSection seller={snapshot.seller} onSave={saveSeller} />
      <CurrencySection currency={snapshot.currency} onSave={saveCurrency} />
      <PreferencesSection
        prefs={snapshot.preferences}
        onSave={async (p) => {
          await savePreferences(p);
          await i18n.changeLanguage(p.language);
        }}
      />
    </div>
  );
}

interface SellerProps {
  seller: SellerProfile;
  onSave: (s: SellerProfile) => Promise<void>;
}

function SellerSection({ seller, onSave }: SellerProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<SellerProfile>(seller);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setForm(seller);
  }, [seller]);

  const update = <K extends keyof SellerProfile>(key: K, value: SellerProfile[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-zinc-900">
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
            <span className="text-sm text-green-600">{t("settings.saved")}</span>
          ) : null}
        </div>
      </form>
    </section>
  );
}

interface CurrencyProps {
  currency: CurrencyConfig;
  onSave: (c: CurrencyConfig) => Promise<void>;
}

function CurrencySection({ currency, onSave }: CurrencyProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<CurrencyConfig>(currency);
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    setForm(currency);
  }, [currency]);

  const update = <K extends keyof CurrencyConfig>(key: K, value: CurrencyConfig[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-zinc-900">
        {t("settings.currency")}
      </h2>
      <form
        className="grid grid-cols-1 gap-3 sm:grid-cols-2"
        onSubmit={async (e) => {
          e.preventDefault();
          setErr(null);
          try {
            await onSave(form);
            setSaved(true);
            setTimeout(() => setSaved(false), 1500);
          } catch (e) {
            setErr(String(e));
          }
        }}
      >
        <Input
          label={t("settings.currency_code") ?? ""}
          value={form.code}
          onChange={(e) => update("code", e.target.value.toUpperCase())}
          required
        />
        <Input
          label={t("settings.currency_symbol") ?? ""}
          value={form.symbol}
          onChange={(e) => update("symbol", e.target.value)}
          required
        />
        <Input
          label={t("settings.main_unit") ?? ""}
          value={form.main_unit_name}
          onChange={(e) => update("main_unit_name", e.target.value)}
        />
        <Input
          label={t("settings.sub_unit") ?? ""}
          value={form.sub_unit_name}
          onChange={(e) => update("sub_unit_name", e.target.value)}
        />
        <label className="flex items-center gap-2 text-sm text-zinc-700">
          <input
            type="checkbox"
            checked={form.symbol_before}
            onChange={(e) => update("symbol_before", e.target.checked)}
          />
          {t("settings.symbol_before")}
        </label>
        <div className="sm:col-span-2 flex items-center gap-3">
          <Button type="submit">{t("common.save")}</Button>
          {saved ? (
            <span className="text-sm text-green-600">{t("settings.saved")}</span>
          ) : null}
          {err ? <span className="text-sm text-red-600">{err}</span> : null}
        </div>
      </form>
    </section>
  );
}

interface PreferencesProps {
  prefs: AppPreferences;
  onSave: (p: AppPreferences) => Promise<void>;
}

function PreferencesSection({ prefs, onSave }: PreferencesProps) {
  const { t } = useTranslation();
  const [form, setForm] = useState<AppPreferences>(prefs);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setForm(prefs);
  }, [prefs]);

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-zinc-900">
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
        <label className="flex flex-col gap-1 text-sm font-medium text-zinc-700">
          {t("settings.theme")}
          <select
            className="block w-full rounded-md border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm"
            value={form.theme}
            onChange={(e) =>
              setForm({ ...form, theme: e.target.value as Theme })
            }
          >
            <option value="Light">{t("settings.light")}</option>
            <option value="Dark">{t("settings.dark")}</option>
          </select>
        </label>
        <label className="flex flex-col gap-1 text-sm font-medium text-zinc-700">
          {t("settings.language")}
          <select
            className="block w-full rounded-md border border-zinc-300 bg-white px-3 py-2 text-sm text-zinc-900 shadow-sm"
            value={form.language}
            onChange={(e) =>
              setForm({ ...form, language: e.target.value as Language })
            }
          >
            <option value="fr">Français</option>
            <option value="en">English</option>
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
            <span className="text-sm text-green-600">{t("settings.saved")}</span>
          ) : null}
        </div>
      </form>
    </section>
  );
}
