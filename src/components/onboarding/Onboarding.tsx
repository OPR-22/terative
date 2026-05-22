import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../common/Button";
import { Input } from "../common/Input";
import { useCurrencyCatalogStore } from "../../stores/currencyCatalogStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { ipc, type LanguageDto } from "../../ipc";

const languageToI18n = (lang: LanguageDto): string =>
  lang === "Fr" ? "fr" : "en";

type Step = "welcome" | "seller" | "currency" | "done";

/**
 * Shown at first launch when the seller profile has no name. Walks the user
 * through the minimum setup needed to create an invoice (name, currency).
 * Dismisses itself once the seller name is set; the user can also skip and
 * configure things later in Settings.
 */
export function Onboarding() {
  const { t, i18n } = useTranslation();
  const { snapshot, load, saveSeller, saveCurrency, savePreferences } =
    useSettingsStore();
  const { all: currencies, load: loadCatalog } = useCurrencyCatalogStore();
  const [step, setStep] = useState<Step>("welcome");
  const [visible, setVisible] = useState(false);
  const [name, setName] = useState("");
  const [email, setEmail] = useState("");
  const [currencyCode, setCurrencyCode] = useState("EUR");
  const [language, setLanguage] = useState<LanguageDto>("Fr");
  const [startNumber, setStartNumber] = useState(1);
  const [canEditNumber, setCanEditNumber] = useState(true);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void loadCatalog();
  }, [loadCatalog]);

  useEffect(() => {
    if (!snapshot) {
      void load();
      return;
    }
    // Decide once, on first load, whether to show. Don't re-open it if the
    // user clears the seller name later.
    const firstLaunch = snapshot.seller.name.trim() === "";
    if (firstLaunch) {
      setName(snapshot.seller.name);
      setEmail(snapshot.seller.email ?? "");
      setCurrencyCode(snapshot.currency.code);
      setLanguage(snapshot.preferences.language);
      setVisible(true);
      // The invoice-number sequence lives in its own table, not the settings
      // snapshot — load it directly. Non-critical: on failure the user can
      // still set it later in Settings.
      void ipc
        .invoiceNumberingGet()
        .then((n) => {
          setStartNumber(n.next_number);
          setCanEditNumber(n.can_edit);
        })
        .catch(() => {});
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [snapshot?.seller.name === ""]);

  if (!visible || !snapshot) return null;

  const close = () => setVisible(false);

  const saveAndClose = async () => {
    setBusy(true);
    try {
      await saveSeller({
        ...snapshot.seller,
        name: name.trim() || snapshot.seller.name,
        email: email.trim() || null,
      });
      if (currencyCode !== snapshot.currency.code) {
        await saveCurrency(currencyCode);
      }
      if (language !== snapshot.preferences.language) {
        await savePreferences({ ...snapshot.preferences, language });
        await i18n.changeLanguage(languageToI18n(language));
      }
      if (canEditNumber) {
        await ipc.invoiceNumberingSetStart(startNumber);
      }
      setStep("done");
      setTimeout(close, 900);
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="fixed inset-0 z-20 flex items-center justify-center bg-overlay p-4">
      <div className="w-full max-w-lg rounded-card bg-surface p-6 shadow-card">
        {step === "welcome" && (
          <div className="flex flex-col gap-4">
            <h2 className="text-xl font-bold text-fg">
              {t("onboarding.welcome_title")}
            </h2>
            <p className="text-sm text-fg-muted">
              {t("onboarding.welcome_body")}
            </p>
            <div className="flex justify-end gap-2">
              <Button variant="secondary" onClick={close}>
                {t("onboarding.skip")}
              </Button>
              <Button onClick={() => setStep("seller")}>
                {t("onboarding.get_started")}
              </Button>
            </div>
          </div>
        )}

        {step === "seller" && (
          <div className="flex flex-col gap-4">
            <h2 className="text-xl font-bold text-fg">
              {t("onboarding.seller_title")}
            </h2>
            <p className="text-sm text-fg-muted">
              {t("onboarding.seller_body")}
            </p>
            <Input
              label={t("settings.seller_name") ?? ""}
              value={name}
              onChange={(e) => setName(e.target.value)}
              autoFocus
              required
            />
            <Input
              label={t("common.email") ?? ""}
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <div className="flex justify-between">
              <Button variant="secondary" onClick={() => setStep("welcome")}>
                {t("onboarding.back")}
              </Button>
              <Button
                onClick={() => setStep("currency")}
                disabled={name.trim() === ""}
              >
                {t("onboarding.next")}
              </Button>
            </div>
          </div>
        )}

        {step === "currency" && (
          <div className="flex flex-col gap-4">
            <h2 className="text-xl font-bold text-fg">
              {t("onboarding.currency_title")}
            </h2>
            <p className="text-sm text-fg-muted">
              {t("onboarding.currency_body")}
            </p>
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("settings.currency")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
                value={currencyCode}
                onChange={(e) => setCurrencyCode(e.target.value)}
              >
                {currencies.map((c) => (
                  <option key={c.code} value={c.code}>
                    {c.code} · {c.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="flex flex-col gap-1 text-sm font-medium text-fg-muted">
              {t("settings.language")}
              <select
                className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm"
                value={language}
                onChange={(e) => setLanguage(e.target.value as LanguageDto)}
              >
                <option value="Fr">Français</option>
                <option value="En">English</option>
              </select>
            </label>
            <Input
              label={t("settings.next_invoice_number") ?? ""}
              type="number"
              min="1"
              value={startNumber}
              disabled={!canEditNumber}
              onChange={(e) =>
                setStartNumber(Math.max(1, parseInt(e.target.value, 10) || 1))
              }
            />
            <div className="flex justify-between">
              <Button variant="secondary" onClick={() => setStep("seller")}>
                {t("onboarding.back")}
              </Button>
              <Button onClick={saveAndClose} disabled={busy}>
                {t("onboarding.finish")}
              </Button>
            </div>
          </div>
        )}

        {step === "done" && (
          <div className="flex flex-col items-center gap-3 py-6">
            <span className="text-2xl">✓</span>
            <h2 className="text-xl font-bold text-fg">
              {t("onboarding.done_title")}
            </h2>
            <p className="text-sm text-fg-muted">{t("onboarding.done_body")}</p>
          </div>
        )}
      </div>
    </div>
  );
}
