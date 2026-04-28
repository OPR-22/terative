import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Field, Input } from "../components/ui/Input";
import { useTaxStore } from "../stores/taxStore";
import type { NewTaxDefinitionDto, UpdateTaxDto } from "../ipc";

export function TaxEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { taxes, refresh, create, update } = useTaxStore();
  useEffect(() => {
    if (taxes.length === 0) void refresh();
  }, [taxes.length, refresh]);

  const tax = useMemo(() => taxes.find((t) => t.id === id), [taxes, id]);

  const [name, setName] = useState("");
  const [percentage, setPercentage] = useState("0");
  const [taxIdNumber, setTaxIdNumber] = useState("");
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (!tax) return;
    setName(tax.name);
    setPercentage(tax.percentage);
    setTaxIdNumber(tax.tax_id_number ?? "");
  }, [tax]);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setSubmitting(true);
    try {
      if (editing && tax) {
        const payload: UpdateTaxDto = {
          id: tax.id,
          name,
          percentage,
          tax_id_number: taxIdNumber || null,
        };
        await update(payload);
      } else {
        const payload: NewTaxDefinitionDto = {
          name,
          percentage,
          tax_id_number: taxIdNumber || null,
        };
        await create(payload);
      }
      navigate("/taxes");
    } catch (e) {
      toast.error(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Page
      crumbs={[
        { label: t("taxes.title"), to: "/taxes" },
        editing ? t("taxes.edit") : t("taxes.new"),
      ]}
      title={editing ? t("taxes.edit") : t("taxes.new")}
    >
      <form onSubmit={submit} className="max-w-lg">
        <Card>
          <CardHead title={editing ? `${t("taxes.edit")} — ${tax?.name ?? ""}` : t("taxes.new")} />
          <CardBody>
            <div className="flex flex-col gap-3.5">
              <Field label={t("common.name")}>
                <Input value={name} onChange={(e) => setName(e.target.value)} required />
              </Field>
              <Field label={t("taxes.percentage")}>
                <div className="flex items-center gap-2">
                  <Input
                    mono
                    type="number"
                    step="0.01"
                    min="0"
                    value={percentage}
                    onChange={(e) => setPercentage(e.target.value)}
                    required
                    className="text-right"
                  />
                  <span className="text-[12px] text-ink-3">%</span>
                </div>
              </Field>
              <Field label={t("taxes.tax_id_number")}>
                <Input
                  mono
                  value={taxIdNumber}
                  onChange={(e) => setTaxIdNumber(e.target.value)}
                  placeholder={t("common.optional")}
                />
              </Field>
            </div>
            {err ? <p className="mt-3 text-[13px] text-danger">{err}</p> : null}
          </CardBody>
        </Card>
        <div className="mt-4 flex justify-end gap-2">
          <Button type="button" onClick={() => navigate("/taxes")}>
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
