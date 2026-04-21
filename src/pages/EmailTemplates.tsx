import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../components/common/Button";
import { Input } from "../components/common/Input";
import { useEmailTemplateStore } from "../stores/emailTemplateStore";
import type { EmailTemplateDto, EmailTemplateTypeDto } from "../ipc";

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

type EditorState =
  | { mode: "closed" }
  | { mode: "create"; templateType: EmailTemplateTypeDto }
  | { mode: "edit"; template: EmailTemplateDto };

export function EmailTemplates() {
  const { t } = useTranslation();
  const { templates, loading, error, refresh, create, update, remove, setDefault } =
    useEmailTemplateStore();
  const [editor, setEditor] = useState<EditorState>({ mode: "closed" });
  const [form, setForm] = useState({
    name: "",
    subject_template: "",
    body_template: "",
  });
  const [saveErr, setSaveErr] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const openCreate = (templateType: EmailTemplateTypeDto) => {
    setEditor({ mode: "create", templateType });
    setForm({ name: "", subject_template: "", body_template: "" });
    setSaveErr(null);
  };

  const openEdit = (tmpl: EmailTemplateDto) => {
    setEditor({ mode: "edit", template: tmpl });
    setForm({
      name: tmpl.name,
      subject_template: tmpl.subject_template,
      body_template: tmpl.body_template,
    });
    setSaveErr(null);
  };

  const closeEditor = () => {
    setEditor({ mode: "closed" });
    setSaveErr(null);
  };

  const handleSave = async () => {
    setSaveErr(null);
    try {
      if (editor.mode === "edit") {
        await update({
          id: editor.template.id,
          name: form.name,
          subject_template: form.subject_template,
          body_template: form.body_template,
        });
      } else if (editor.mode === "create") {
        await create({
          name: form.name,
          template_type: editor.templateType,
          subject_template: form.subject_template,
          body_template: form.body_template,
        });
      }
      closeEditor();
    } catch (e) {
      setSaveErr(String(e));
    }
  };

  const typeLabel = (tt: EmailTemplateTypeDto) =>
    tt === "InitialContact"
      ? t("email_templates.initial_contact")
      : t("email_templates.follow_up");

  const typeDescription = (tt: EmailTemplateTypeDto) =>
    tt === "InitialContact"
      ? t("email_templates.initial_contact_desc")
      : t("email_templates.follow_up_desc");

  // -- Editor view --
  if (editor.mode !== "closed") {
    const heading =
      editor.mode === "edit"
        ? `${t("common.edit")} — ${editor.template.name}`
        : `${t("email_templates.new_template")} — ${typeLabel(editor.mode === "create" ? editor.templateType : "InitialContact")}`;

    return (
      <div className="max-w-3xl">
        <button
          className="mb-4 text-sm font-medium text-fg-muted hover:text-fg"
          onClick={closeEditor}
        >
          &larr; {t("common.back")}
        </button>

        <h1 className="mb-6 text-2xl font-bold text-fg">{heading}</h1>

        <div className="space-y-4">
          <Input
            label={t("email_templates.name") ?? ""}
            value={form.name}
            onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
            placeholder={t("email_templates.name_placeholder") ?? ""}
          />

          <div>
            <label className="mb-1 block text-sm font-medium text-fg-muted">
              {t("email_templates.subject")}
            </label>
            <input
              className="block w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand"
              value={form.subject_template}
              onChange={(e) =>
                setForm((f) => ({ ...f, subject_template: e.target.value }))
              }
              placeholder="Invoice {{number}} from {{seller_name}}"
            />
          </div>

          <div>
            <label className="mb-1 block text-sm font-medium text-fg-muted">
              {t("email_templates.body")}
            </label>
            <textarea
              className="block min-h-48 w-full rounded-field border border-border bg-surface px-3 py-2 text-sm text-fg shadow-sm focus:border-brand focus:outline-none focus:ring-1 focus:ring-brand"
              value={form.body_template}
              onChange={(e) =>
                setForm((f) => ({ ...f, body_template: e.target.value }))
              }
              placeholder={`Hi {{client_name}},\n\nPlease find invoice {{number}} attached.\nTotal: {{total}}.\n\n— {{seller_name}}`}
            />
          </div>

          <div className="rounded-field border border-border bg-surface-muted px-3 py-2">
            <p className="mb-1 text-xs font-medium text-fg-muted">
              {t("email_templates.placeholders_help")}
            </p>
            <div className="flex flex-wrap gap-1">
              {PLACEHOLDER_KEYS.map((k) => (
                <code
                  key={k}
                  className="rounded-field bg-surface px-1.5 py-0.5 text-xs text-fg-muted"
                >{`{{${k}}}`}</code>
              ))}
            </div>
          </div>

          {saveErr ? (
            <p className="text-sm text-danger">{saveErr}</p>
          ) : null}

          <div className="flex items-center gap-2 pt-2">
            <Button onClick={handleSave}>{t("common.save")}</Button>
            <Button variant="secondary" onClick={closeEditor}>
              {t("common.cancel")}
            </Button>
          </div>
        </div>
      </div>
    );
  }

  // -- List view --
  return (
    <div className="max-w-5xl">
      <div className="mb-6">
        <h1 className="text-2xl font-bold text-fg">
          {t("email_templates.title")}
        </h1>
        <p className="mt-1 text-sm text-fg-muted">
          {t("email_templates.page_desc")}
        </p>
      </div>

      {error ? <p className="mb-4 text-sm text-danger">{error}</p> : null}

      {loading ? (
        <p className="text-sm text-fg-muted">{t("common.loading")}</p>
      ) : (
        <div className="space-y-8">
          {(["InitialContact", "FollowUp"] as const).map((tt) => {
            const group = templates.filter((tmpl) => tmpl.template_type === tt);
            return (
              <section key={tt}>
                <div className="mb-3 flex items-center justify-between">
                  <div>
                    <h2 className="text-lg font-semibold text-fg">
                      {typeLabel(tt)}
                    </h2>
                    <p className="text-sm text-fg-muted">{typeDescription(tt)}</p>
                  </div>
                  <Button variant="secondary" onClick={() => openCreate(tt)}>
                    {t("common.add")}
                  </Button>
                </div>

                {group.length === 0 ? (
                  <p className="text-sm text-fg-subtle">
                    {t("email_templates.none")}
                  </p>
                ) : (
                  <ul className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                    {group.map((tmpl) => (
                      <li
                        key={tmpl.id}
                        className="rounded-card border border-border bg-surface p-4 shadow-card"
                      >
                        <div className="mb-2 flex items-start justify-between">
                          <div>
                            <h3 className="font-semibold text-fg">
                              {tmpl.name}
                            </h3>
                            <p className="mt-0.5 text-xs text-fg-subtle">
                              {tmpl.subject_template}
                            </p>
                          </div>
                          {tmpl.is_default ? (
                            <span className="rounded-pill bg-status-finalized-bg px-2 py-0.5 text-xs font-medium text-status-finalized-fg">
                              {t("email_templates.default_badge")}
                            </span>
                          ) : null}
                        </div>
                        <p className="mb-3 line-clamp-2 text-xs text-fg-muted">
                          {tmpl.body_template}
                        </p>
                        <div className="flex flex-wrap gap-2">
                          <Button
                            variant="secondary"
                            onClick={() => openEdit(tmpl)}
                          >
                            {t("common.edit")}
                          </Button>
                          {!tmpl.is_default ? (
                            <Button
                              variant="secondary"
                              onClick={() =>
                                void setDefault(tmpl.id).catch((e) =>
                                  alert(String(e)),
                                )
                              }
                            >
                              {t("email_templates.set_default")}
                            </Button>
                          ) : null}
                          {!tmpl.is_default ? (
                            <Button
                              variant="danger"
                              onClick={() => {
                                if (confirm(t("common.confirm_delete"))) {
                                  void remove(tmpl.id).catch((e) =>
                                    alert(String(e)),
                                  );
                                }
                              }}
                            >
                              {t("common.delete")}
                            </Button>
                          ) : null}
                        </div>
                      </li>
                    ))}
                  </ul>
                )}
              </section>
            );
          })}
        </div>
      )}
    </div>
  );
}
