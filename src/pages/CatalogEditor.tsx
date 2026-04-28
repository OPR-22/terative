import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Field, Input } from "../components/ui/Input";
import { Pills } from "../components/ui/Pills";
import { MoneyInput } from "../components/common/MoneyInput";
import { useCatalogStore } from "../stores/catalogStore";
import { useSettingsStore } from "../stores/settingsStore";
import type { CatalogItemKindDto, MoneyDto } from "../ipc";

export function CatalogEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { items, refresh, create, update } = useCatalogStore();
  const { snapshot, load } = useSettingsStore();

  useEffect(() => {
    if (items.length === 0) void refresh();
    if (!snapshot) void load();
  }, [items.length, refresh, load, snapshot]);

  const item = useMemo(() => items.find((i) => i.id === id), [items, id]);
  const currency = snapshot?.currency;
  const currencyCode = currency?.code ?? "EUR";

  const [name, setName] = useState("");
  const [kind, setKind] = useState<CatalogItemKindDto>("Service");
  const [price, setPrice] = useState<MoneyDto>({ amount_minor: 0, currency: currencyCode });
  const [unit, setUnit] = useState("");
  const [reference, setReference] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (item) {
      setName(item.name);
      setKind(item.kind);
      setPrice(item.default_price);
      setUnit(item.unit ?? "");
      setReference(item.reference ?? "");
    } else {
      setPrice({ amount_minor: 0, currency: currencyCode });
    }
  }, [item, currencyCode]);

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
          default_price: price,
          unit: unit.trim() || null,
          reference: reference.trim() || null,
        });
      } else {
        await create({
          name,
          kind,
          default_price: price,
          unit: unit.trim() || null,
          reference: reference.trim() || null,
        });
      }
      navigate("/catalog");
    } catch (e) {
      toast.error(String(e));
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
              {currency ? (
                <Field label={t("catalog.default_price")}>
                  <MoneyInput
                    valueMinor={price.amount_minor}
                    currency={currency}
                    onChangeMinor={(minor) =>
                      setPrice({ amount_minor: minor, currency: currencyCode })
                    }
                  />
                </Field>
              ) : null}
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
