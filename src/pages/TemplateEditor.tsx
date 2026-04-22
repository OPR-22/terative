import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { ImageUploader } from "../components/common/ImageUploader";
import { Input } from "../components/common/Input";
import { PdfPreview, useDebounced } from "../components/template/PdfPreview";
import { useTemplateStore } from "../stores/templateStore";
import {
  ipc,
  type FontChoiceDto,
  type InvoiceTemplateDto,
  type NewInvoiceTemplateDto,
  type TemplateLayoutDto,
  type TemplateOverrideDto,
  type UpdateTemplateDto,
} from "../ipc";

type EditorInitial =
  | (InvoiceTemplateDto & { id: string })
  | (NewInvoiceTemplateDto & { id: null });

interface Props {
  initial: EditorInitial;
  onClose: () => void;
}

const LAYOUTS: TemplateLayoutDto[] = ["Classic", "Modern", "Minimal"];
const FONTS: FontChoiceDto[] = ["SansSerif", "Serif", "Mono"];

type Form = NewInvoiceTemplateDto;

export function TemplateEditor({ initial, onClose }: Props) {
  const { t } = useTranslation();
  const { create, update } = useTemplateStore();
  const [form, setForm] = useState<Form>(stripId(initial));
  const [submitting, setSubmitting] = useState(false);
  const [saveError, setSaveError] = useState<string | null>(null);

  const [previewBytes, setPreviewBytes] = useState<Uint8Array | null>(null);
  const [previewLoading, setPreviewLoading] = useState(false);
  const [previewError, setPreviewError] = useState<string | null>(null);

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
      .templatePreview({
        template_id: initial.id ?? null,
        overrides,
      })
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
  }, [debouncedForm, initial.id]);

  const updateField = <K extends keyof Form>(key: K, value: Form[K]) =>
    setForm((f) => ({ ...f, [key]: value }));

  const toggle = (key: keyof Form) =>
    setForm((f) => ({ ...f, [key]: !f[key] }));

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaveError(null);
    setSubmitting(true);
    try {
      if (initial.id) {
        const payload: UpdateTemplateDto = { ...form, id: initial.id };
        await update(payload);
      } else {
        await create(form);
      }
      onClose();
    } catch (e) {
      setSaveError(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="flex h-[calc(100vh-3rem)] flex-col">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">
          {initial.id ? t("templates.edit") : t("templates.new")}
        </h1>
        <div className="flex gap-2">
          <Button variant="secondary" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={submit} disabled={submitting}>
            {t("common.save")}
          </Button>
        </div>
      </div>

      {saveError ? (
        <p className="mb-3 text-sm text-danger">{saveError}</p>
      ) : null}

      <div className="grid flex-1 grid-cols-1 gap-4 overflow-hidden lg:grid-cols-[minmax(0,26rem)_1fr]">
        <form
          onSubmit={submit}
          className="overflow-y-auto rounded-card border border-border bg-surface p-5 shadow-card"
        >
          <div className="flex flex-col gap-5">
            <Input
              label={t("common.name") ?? ""}
              value={form.name}
              onChange={(e) => updateField("name", e.target.value)}
              required
            />

            <fieldset className="flex flex-col gap-2">
              <legend className="text-sm font-medium text-fg-muted">
                {t("templates.layout")}
              </legend>
              <div className="flex gap-2">
                {LAYOUTS.map((layout) => (
                  <button
                    key={layout}
                    type="button"
                    onClick={() => updateField("base_layout", layout)}
                    className={[
                      "flex-1 rounded-field border px-3 py-2 text-sm font-medium transition-colors",
                      form.base_layout === layout
                        ? "border-brand bg-brand text-brand-fg"
                        : "border-border bg-surface text-fg-muted hover:bg-surface-muted",
                    ].join(" ")}
                  >
                    {t(`templates.layout_${layout.toLowerCase()}`)}
                  </button>
                ))}
              </div>
            </fieldset>

            <fieldset className="flex flex-col gap-2">
              <legend className="text-sm font-medium text-fg-muted">
                {t("templates.font")}
              </legend>
              <div className="flex gap-2">
                {FONTS.map((font) => (
                  <button
                    key={font}
                    type="button"
                    onClick={() => updateField("font_family", font)}
                    className={[
                      "flex-1 rounded-field border px-3 py-2 text-sm font-medium transition-colors",
                      form.font_family === font
                        ? "border-brand bg-brand text-brand-fg"
                        : "border-border bg-surface text-fg-muted hover:bg-surface-muted",
                    ].join(" ")}
                  >
                    {t(`templates.font_${font.toLowerCase()}`)}
                  </button>
                ))}
              </div>
            </fieldset>

            <ImageUploader
              label={t("templates.logo") ?? ""}
              value={form.logo_image}
              onChange={(bytes) => updateField("logo_image", bytes)}
            />

            <div className="flex items-end gap-3">
              <Input
                label={t("templates.accent_color") ?? ""}
                value={form.accent_color ?? ""}
                onChange={(e) =>
                  updateField("accent_color", e.target.value || null)
                }
                placeholder="#2563EB"
              />
              <input
                type="color"
                aria-label={t("templates.accent_color") ?? ""}
                value={form.accent_color ?? "#2563EB"}
                onChange={(e) =>
                  updateField("accent_color", e.target.value.toUpperCase())
                }
                className="h-10 w-12 cursor-pointer rounded-field border border-border bg-surface"
              />
            </div>

            <Input
              label={t("templates.header_text") ?? ""}
              value={form.header_text ?? ""}
              onChange={(e) =>
                updateField("header_text", e.target.value || null)
              }
            />
            <Input
              label={t("templates.footer_text") ?? ""}
              value={form.footer_text ?? ""}
              onChange={(e) =>
                updateField("footer_text", e.target.value || null)
              }
            />

            <fieldset className="flex flex-col gap-2">
              <legend className="text-sm font-medium text-fg-muted">
                {t("templates.toggles")}
              </legend>
              <div className="grid grid-cols-1 gap-2">
                <Toggle
                  label={t("templates.show_seller_phone")}
                  checked={form.show_seller_phone}
                  onChange={() => toggle("show_seller_phone")}
                />
                <Toggle
                  label={t("templates.show_seller_email")}
                  checked={form.show_seller_email}
                  onChange={() => toggle("show_seller_email")}
                />
                <Toggle
                  label={t("templates.show_registration_id")}
                  checked={form.show_registration_id}
                  onChange={() => toggle("show_registration_id")}
                />
                <Toggle
                  label={t("templates.show_tax_id_numbers")}
                  checked={form.show_tax_id_numbers}
                  onChange={() => toggle("show_tax_id_numbers")}
                />
                <Toggle
                  label={t("templates.show_signature")}
                  checked={form.show_signature}
                  onChange={() => toggle("show_signature")}
                />
                <Toggle
                  label={t("templates.show_due_date")}
                  checked={form.show_due_date}
                  onChange={() => toggle("show_due_date")}
                />
                <Toggle
                  label={t("templates.show_total_in_words")}
                  checked={form.show_total_in_words}
                  onChange={() => toggle("show_total_in_words")}
                />
              </div>
            </fieldset>
          </div>
        </form>

        <div className="overflow-hidden">
          <PdfPreview
            bytes={previewBytes}
            loading={previewLoading}
            error={previewError}
          />
        </div>
      </div>
    </div>
  );
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string;
  checked: boolean;
  onChange: () => void;
}) {
  return (
    <label className="flex items-center gap-2 text-sm text-fg-muted">
      <input type="checkbox" checked={checked} onChange={onChange} />
      {label}
    </label>
  );
}

function stripId(initial: EditorInitial): Form {
  return {
    name: initial.name,
    base_layout: initial.base_layout,
    logo_image: initial.logo_image ?? null,
    accent_color: initial.accent_color,
    font_family: initial.font_family,
    show_seller_phone: initial.show_seller_phone,
    show_seller_email: initial.show_seller_email,
    show_registration_id: initial.show_registration_id,
    show_tax_id_numbers: initial.show_tax_id_numbers,
    show_signature: initial.show_signature,
    show_due_date: initial.show_due_date,
    show_total_in_words: initial.show_total_in_words,
    header_text: initial.header_text,
    footer_text: initial.footer_text,
  };
}
