import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";
import { Trash2 } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Field, Input, Select } from "../components/ui/Input";
import { Pills } from "../components/ui/Pills";
import { MoneyInput } from "../components/common/MoneyInput";
import { useCatalogStore } from "../stores/catalogStore";
import { useCurrencyCatalogStore } from "../stores/currencyCatalogStore";
import { useSettingsStore } from "../stores/settingsStore";
import type { CatalogItemKindDto, CurrencyConfigDto, MoneyDto } from "../ipc";

export function CatalogEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { items, refresh, create, update } = useCatalogStore();
  const { snapshot, load } = useSettingsStore();
  const { all: currencies, load: loadCurrencies, byCode } = useCurrencyCatalogStore();

  useEffect(() => {
    if (items.length === 0) void refresh();
    if (!snapshot) void load();
    void loadCurrencies();
  }, [items.length, refresh, load, snapshot, loadCurrencies]);

  const item = useMemo(() => items.find((i) => i.id === id), [items, id]);
  const orgCurrencyCode = snapshot?.currency.code ?? "EUR";

  const [name, setName] = useState("");
  const [kind, setKind] = useState<CatalogItemKindDto>("Service");
  const [prices, setPrices] = useState<MoneyDto[]>([]);
  const [unit, setUnit] = useState("");
  const [reference, setReference] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (item) {
      setName(item.name);
      setKind(item.kind);
      setPrices(item.prices);
      setUnit(item.unit ?? "");
      setReference(item.reference ?? "");
    } else {
      // New item: start with one row in the org's currency. The user can
      // remove it to create an unpriced item, or add more rows in other
      // currencies.
      const meta = byCode(orgCurrencyCode);
      if (meta) setPrices([{ amount: 0, currency: meta }]);
    }
  }, [item, orgCurrencyCode, byCode]);

  const usedCurrencies = useMemo(
    () => new Set(prices.map((p) => p.currency.code)),
    [prices],
  );
  const availableToAdd = currencies.filter((c) => !usedCurrencies.has(c.code));

  const setPriceAmount = (code: string, amount: number) => {
    setPrices((cur) =>
      cur.map((p) => (p.currency.code === code ? { ...p, amount } : p)),
    );
  };
  const removePrice = (code: string) => {
    setPrices((cur) => cur.filter((p) => p.currency.code !== code));
  };
  const addPrice = (currency: CurrencyConfigDto) => {
    setPrices((cur) => [...cur, { amount: 0, currency }]);
  };

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setSubmitting(true);
    try {
      if (editing && item) {
        await update({
          id: item.id,
          name,
          kind,
          prices,
          unit: unit.trim() || null,
          reference: reference.trim() || null,
        });
      } else {
        await create({
          name,
          kind,
          prices,
          unit: unit.trim() || null,
          reference: reference.trim() || null,
        });
      }
      navigate("/catalog");
    } catch (e) {
      toast.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Page
      crumbs={[
        { label: t("catalog.title"), to: "/catalog" },
        editing ? t("catalog.edit") : t("catalog.new"),
      ]}
      title={editing ? t("catalog.edit") : t("catalog.new")}
    >
      <form onSubmit={submit} className="max-w-lg">
        <Card>
          <CardHead
            title={editing ? `${t("catalog.edit")} — ${item?.name ?? ""}` : t("catalog.new")}
          />
          <CardBody>
            <div className="flex flex-col gap-3.5">
              <Field label={t("catalog.kind")}>
                <Pills<CatalogItemKindDto>
                  value={kind}
                  onChange={setKind}
                  options={[
                    { id: "Service", label: t("catalog.kind_service") },
                    { id: "Product", label: t("catalog.kind_product") },
                  ]}
                />
              </Field>
              <Field label={t("common.name")}>
                <Input value={name} onChange={(e) => setName(e.target.value)} required />
              </Field>
              <div className="grid grid-cols-2 gap-3.5">
                <Field label={t("catalog.reference")}>
                  <Input
                    mono
                    value={reference}
                    onChange={(e) => setReference(e.target.value)}
                    placeholder={t("catalog.reference_placeholder") ?? ""}
                  />
                </Field>
                <Field label={t("catalog.unit")}>
                  <Input
                    value={unit}
                    onChange={(e) => setUnit(e.target.value)}
                    placeholder={t("catalog.unit_placeholder") ?? ""}
                  />
                </Field>
              </div>

              <Field label={t("catalog.prices")}>
                <div className="flex flex-col gap-2">
                  {prices.length === 0 ? (
                    <p className="text-[12px] text-ink-3">
                      {t("catalog.no_prices")}
                    </p>
                  ) : (
                    prices.map((p) => (
                      <div key={p.currency.code} className="flex items-end gap-2">
                        <div className="flex-1">
                          <MoneyInput
                            valueMinor={p.amount}
                            currency={p.currency}
                            onChangeMinor={(m) => setPriceAmount(p.currency.code, m)}
                          />
                        </div>
                        <Button
                          type="button"
                          onClick={() => removePrice(p.currency.code)}
                          title={t("catalog.remove_price") ?? ""}
                        >
                          <Trash2 size={13} strokeWidth={1.5} />
                        </Button>
                      </div>
                    ))
                  )}
                  {availableToAdd.length > 0 ? (
                    <Select
                      value=""
                      onChange={(e) => {
                        const meta = byCode(e.target.value);
                        if (meta) addPrice(meta);
                      }}
                    >
                      <option value="">{t("catalog.add_price")}</option>
                      {availableToAdd.map((c) => (
                        <option key={c.code} value={c.code}>
                          {c.code} — {c.name}
                        </option>
                      ))}
                    </Select>
                  ) : null}
                </div>
              </Field>
            </div>
            {err ? <p className="mt-3 text-[13px] text-danger">{err}</p> : null}
          </CardBody>
        </Card>
        <div className="mt-4 flex justify-end gap-2">
          <Button type="button" onClick={() => navigate("/catalog")}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" variant="primary" disabled={submitting}>
            {t("common.save")}
          </Button>
        </div>
      </form>
    </Page>
  );
}
