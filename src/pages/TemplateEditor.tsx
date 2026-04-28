import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router-dom";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody } from "../components/ui/Card";
import { Field, Input } from "../components/ui/Input";
import { Pills } from "../components/ui/Pills";
import { Toggle } from "../components/ui/Toggle";
import { ImageUploader } from "../components/common/ImageUploader";
import { PdfPreview, useDebounced } from "../components/template/PdfPreview";
import { useTemplateStore } from "../stores/templateStore";
import {
  ipc,
  type FontChoiceDto,
  type NewInvoiceTemplateDto,
  type TemplateLayoutDto,
  type TemplateOverrideDto,
  type UpdateTemplateDto,
} from "../ipc";

const LAYOUTS: TemplateLayoutDto[] = ["Classic", "Modern", "Minimal"];
const FONTS: FontChoiceDto[] = ["SansSerif", "Serif", "Mono"];

function defaults(): NewInvoiceTemplateDto {
  return {
    name: "",
    base_layout: "Classic",
    logo_image: null,
    accent_color: null,
    font_family: "SansSerif",
    show_seller_phone: true,
    show_seller_email: true,
    show_registration_id: true,
    show_tax_id_numbers: true,
    show_signature: false,
    show_due_date: true,
    show_total_in_words: false,
    header_text: null,
    footer_text: null,
  };
}

export function TemplateEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { templates, refresh, create, update } = useTemplateStore();
  useEffect(() => {
    if (templates.length === 0) void refresh();
  }, [templates.length, refresh]);

  const existing = useMemo(() => templates.find((tpl) => tpl.id === id), [templates, id]);

  const [form, setForm] = useState<NewInvoiceTemplateDto>(defaults);
  const [submitting, setSubmitting] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const [previewBytes, setPreviewBytes] = useState<Uint8Array | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

  useEffect(() => {
    if (!existing) return;
    setForm({
      name: existing.name,
      base_layout: existing.base_layout,
      logo_image: existing.logo_image ?? null,
      accent_color: existing.accent_color,
      font_family: existing.font_family,
      show_seller_phone: existing.show_seller_phone,
      show_seller_email: existing.show_seller_email,
      show_registration_id: existing.show_registration_id,
      show_tax_id_numbers: existing.show_tax_id_numbers,
      show_signature: existing.show_signature,
      show_due_date: existing.show_due_date,
      show_total_in_words: existing.show_total_in_words,
      header_text: existing.header_text,
      footer_text: existing.footer_text,
    });
  }, [existing]);

  const debouncedForm = useDebounced(form, 300);

  useEffect(() => {
    let cancelled = false;
    const overrides: TemplateOverrideDto = {
      base_layout: debouncedForm.base_layout,
      accent_color: debouncedForm.accent_color,
      font_family: debouncedForm.font_family,
      logo_image: debouncedForm.logo_image ?? null,
      show_seller_phone: debouncedForm.show_seller_phone,
      show_seller_email: debouncedForm.show_seller_email,
      show_registration_id: debouncedForm.show_registration_id,
      show_tax_id_numbers: debouncedForm.show_tax_id_numbers,
      show_signature: debouncedForm.show_signature,
      show_due_date: debouncedForm.show_due_date,
      show_total_in_words: debouncedForm.show_total_in_words,
      header_text: debouncedForm.header_text,
      footer_text: debouncedForm.footer_text,
    };
    setPreviewLoading(true);
    setPreviewError(null);
    ipc
      .templatePreview({ template_id: existing?.id ?? null, overrides })
      .then((bytes) => {
        if (!cancelled) setPreviewBytes(new Uint8Array(bytes));
      })
      .catch((e) => {
        if (!cancelled) setPreviewError(String(e));
      })
      .finally(() => {
        if (!cancelled) setPreviewLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [debouncedForm, existing?.id]);

  const updateField = <K extends keyof NewInvoiceTemplateDto>(
    key: K,
    value: NewInvoiceTemplateDto[K],
  ) => setForm((f) => ({ ...f, [key]: value }));

  const toggle = (key: keyof NewInvoiceTemplateDto) =>
    setForm((f) => ({ ...f, [key]: !f[key] }));

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaveError(null);
    setSubmitting(true);
    try {
      if (editing && existing) {
        const payload: UpdateTemplateDto = { ...form, id: existing.id };
        await update(payload);
      } else {
        await create(form);
      }
      navigate("/templates");
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Page
      crumbs={[
        "Cabinet Lemaire",
        t("templates.title"),
        editing ? existing?.name ?? t("templates.edit") : t("templates.new"),
      ]}
      title={editing ? t("templates.edit") : t("templates.new")}
      subtitle="Aperçu mis à jour automatiquement"
      actions={
        <>
          <Button onClick={() => navigate("/templates")}>{t("common.cancel")}</Button>
          <Button onClick={submit} variant="primary" disabled={submitting}>
            {t("common.save")}
          </Button>
        </>
      }
    >
      {saveError ? <p className="mb-3 text-[13px] text-danger">{saveError}</p> : null}

      <Card className="overflow-hidden">
        <div className="grid grid-cols-1 lg:grid-cols-[360px_1fr]">
          <form onSubmit={submit} className="border-r border-line">
            <CardBody className="flex flex-col gap-4">
              <Field label={t("common.name")}>
                <Input
                  value={form.name}
                  onChange={(e) => updateField("name", e.target.value)}
                  required
                />
              </Field>

              <Field label={t("templates.layout")}>
                <Pills<TemplateLayoutDto>
                  value={form.base_layout}
                  onChange={(v) => updateField("base_layout", v)}
                  options={LAYOUTS.map((l) => ({
                    id: l,
                    label: t(`templates.layout_${l.toLowerCase()}`),
                  }))}
                />
              </Field>

              <Field label={t("templates.font")}>
                <Pills<FontChoiceDto>
                  value={form.font_family}
                  onChange={(v) => updateField("font_family", v)}
                  options={FONTS.map((f) => ({
                    id: f,
                    label: t(`templates.font_${f.toLowerCase()}`),
                  }))}
                />
              </Field>

              <ImageUploader
                label={t("templates.logo") ?? ""}
                value={form.logo_image}
                onChange={(bytes) => updateField("logo_image", bytes)}
              />

              <Field label={t("templates.accent_color")}>
                <div className="flex items-center gap-2">
                  <Input
                    mono
                    value={form.accent_color ?? ""}
                    onChange={(e) =>
                      updateField("accent_color", e.target.value || null)
                    }
                    placeholder="#2563EB"
                    className="w-32"
                  />
                  <input
                    type="color"
                    aria-label={t("templates.accent_color") ?? ""}
                    value={form.accent_color ?? "#2563EB"}
                    onChange={(e) =>
                      updateField("accent_color", e.target.value.toUpperCase())
                    }
                    className="h-8 w-8 cursor-pointer border border-line bg-paper rounded-sm"
                  />
                </div>
              </Field>

              <Field label={t("templates.header_text")}>
                <Input
                  value={form.header_text ?? ""}
                  onChange={(e) =>
                    updateField("header_text", e.target.value || null)
                  }
                />
              </Field>

              <Field label={t("templates.footer_text")}>
                <Input
                  value={form.footer_text ?? ""}
                  onChange={(e) =>
                    updateField("footer_text", e.target.value || null)
                  }
                />
              </Field>

              <div>
                <div className="text-[12px] font-medium text-ink-3 mb-1.5">
                  {t("templates.toggles")}
                </div>
                <div className="flex flex-col gap-2">
                  {(
                    [
                      "show_seller_phone",
                      "show_seller_email",
                      "show_registration_id",
                      "show_tax_id_numbers",
                      "show_signature",
                      "show_due_date",
                      "show_total_in_words",
                    ] as const
                  ).map((key) => (
                    <Toggle
                      key={key}
                      checked={form[key] as boolean}
                      onChange={() => toggle(key)}
                      label={t(`templates.${key}`)}
                    />
                  ))}
                </div>
              </div>
            </CardBody>
          </form>

          <div className="bg-paper-3 p-6 grid place-items-center min-h-[600px]">
            <PdfPreview
              bytes={previewBytes}
              loading={previewLoading}
              error={previewError}
            />
          </div>
        </div>
      </Card>
    </Page>
  );
}
