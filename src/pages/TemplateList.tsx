import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { TemplateEditor } from "./TemplateEditor";
import { useTemplateStore } from "../stores/templateStore";
import type { InvoiceTemplateDto, NewInvoiceTemplateDto } from "../ipc";

type EditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; template: InvoiceTemplateDto };

function defaultNewTemplate(): NewInvoiceTemplateDto {
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

export function TemplateList() {
  const { t } = useTranslation();
  const {
    templates,
    loading,
    error,
    refresh,
    remove,
    duplicate,
    setDefault,
  } = useTemplateStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (editor.mode !== "closed") {
    return (
      <TemplateEditor
        initial={
          editor.mode === "edit"
            ? editor.template
            : { ...defaultNewTemplate(), id: null }
        }
        onClose={() => setEditor({ mode: "closed" })}
      />
    );
  }

  return (
    <div className="max-w-5xl">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-2xl font-bold text-fg">{t("templates.title")}</h1>
        <Button onClick={() => setEditor({ mode: "create" })}>
          {t("templates.new")}
        </Button>
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}
      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : templates.length === 0 ? (
        <p className="text-sm text-fg-muted">{t("templates.none")}</p>
      ) : (
        <ul className="grid grid-cols-1 gap-3 sm:grid-cols-2">
          {templates.map((tpl) => (
            <li
              key={tpl.id}
              className="rounded-card border border-border bg-surface p-4 shadow-card"
            >
              <div className="mb-3 flex items-start justify-between">
                <div>
                  <h2 className="font-semibold text-fg">{tpl.name}</h2>
                  <p className="text-xs text-fg-subtle">
                    {t(`templates.layout_${tpl.base_layout.toLowerCase()}`)} ·{" "}
                    {tpl.font_family}
                  </p>
                </div>
                {tpl.is_default ? (
                  <span className="rounded-pill bg-status-finalized-bg px-2 py-0.5 text-xs font-medium text-status-finalized-fg">
                    {t("templates.default")}
                  </span>
                ) : null}
              </div>
              <div className="flex flex-wrap gap-2">
                <Button
                  variant="secondary"
                  onClick={() => setEditor({ mode: "edit", template: tpl })}
                >
                  {t("common.edit")}
                </Button>
                <Button
                  variant="secondary"
                  onClick={() => void duplicate(tpl.id)}
                >
                  {t("templates.duplicate")}
                </Button>
                {!tpl.is_default ? (
                  <Button
                    variant="secondary"
                    onClick={() => void setDefault(tpl.id)}
                  >
                    {t("templates.make_default")}
                  </Button>
                ) : null}
                <Button
                  variant="danger"
                  onClick={() => {
                    if (confirm(t("common.confirm_delete"))) {
                      void remove(tpl.id).catch((e) => alert(String(e)));
                    }
                  }}
                >
                  {t("common.delete")}
                </Button>
              </div>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
