import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { ArrowLeft, Edit, Mail, Plus, Star, Trash2 } from "lucide-react";

import { Page, SectionTitle } from "../components/layout/Page";
import { useWorkspaceName } from "../hooks/useWorkspaceName";
import { Badge } from "../components/ui/Badge";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Field, Input, Textarea } from "../components/ui/Input";
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
  const workspaceName = useWorkspaceName();
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
      toast.error(String(e));
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

  if (editor.mode !== "closed") {
    const typeForLabel =
      editor.mode === "create" ? editor.templateType : editor.template.template_type;
    return (
      <Page
        crumbs={[
          workspaceName,
          t("email_templates.title"),
          editor.mode === "edit" ? editor.template.name : t("email_templates.new_template"),
        ]}
        title={
          editor.mode === "edit"
            ? `${t("common.edit")} — ${editor.template.name}`
            : `${t("email_templates.new_template")} — ${typeLabel(typeForLabel)}`
        }
        actions={
          <>
            <Button
              leadingIcon={<ArrowLeft size={13} strokeWidth={1.5} />}
              onClick={closeEditor}
            >
              {t("common.back")}
            </Button>
            <Button variant="primary" onClick={handleSave}>
              {t("common.save")}
            </Button>
          </>
        }
      >
        <Card>
          <div className="grid grid-cols-1 lg:grid-cols-[1fr_280px]">
            <CardBody className="border-r border-line">
              <div className="flex flex-col gap-3.5">
                <Field label={t("email_templates.name")}>
                  <Input
                    value={form.name}
                    onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                    placeholder={t("email_templates.name_placeholder") ?? ""}
                  />
                </Field>
                <Field label={t("email_templates.subject")}>
                  <Input
                    mono
                    value={form.subject_template}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, subject_template: e.target.value }))
                    }
                    placeholder="Invoice {{number}} from {{seller_name}}"
                  />
                </Field>
                <Field label={t("email_templates.body")}>
                  <Textarea
                    rows={10}
                    value={form.body_template}
                    onChange={(e) =>
                      setForm((f) => ({ ...f, body_template: e.target.value }))
                    }
                    placeholder={`Hi {{client_name}},\n\nPlease find invoice {{number}} attached.\nTotal: {{total}}.\n\n— {{seller_name}}`}
                  />
                </Field>
              </div>
              {saveErr ? (
                <p className="mt-3 text-[13px] text-danger">{saveErr}</p>
              ) : null}
            </CardBody>
            <div className="bg-paper-2 p-4">
              <div className="text-[12px] font-medium text-ink-3 mb-2">
                {t("email_templates.placeholders_help")}
              </div>
              <div className="flex flex-col gap-1">
                {PLACEHOLDER_KEYS.map((k) => (
                  <code
                    key={k}
                    className="bg-paper border border-line-soft rounded-sm px-1.5 py-0.5 font-mono text-[11px] text-ink-2 self-start"
                  >{`{{${k}}}`}</code>
                ))}
              </div>
            </div>
          </div>
        </Card>
      </Page>
    );
  }

  return (
    <Page
      crumbs={[workspaceName, t("email_templates.title")]}
      title={t("email_templates.title")}
      subtitle={t("email_templates.page_desc")}
    >
      {error ? <p className="mb-3 text-[13px] text-danger">{error}</p> : null}

      {loading ? (
        <Card>
          <EmptyState description={t("common.loading")} />
        </Card>
      ) : (
        (["InitialContact", "FollowUp"] as const).map((tt) => {
          const group = templates.filter((tmpl) => tmpl.template_type === tt);
          return (
            <div key={tt} className="mb-7">
              <SectionTitle
                action={
                  <Button
                    size="sm"
                    leadingIcon={<Plus size={11} strokeWidth={1.5} />}
                    onClick={() => openCreate(tt)}
                  >
                    {t("common.add")}
                  </Button>
                }
              >
                <span className="text-ink font-medium text-[14px]">
                  {typeLabel(tt)}
                </span>
                <span className="ml-2 text-ink-3 font-normal text-[12px]">
                  {typeDescription(tt)}
                </span>
              </SectionTitle>

              {group.length === 0 ? (
                <Card>
                  <EmptyState description={t("email_templates.none")} />
                </Card>
              ) : (
                <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                  {group.map((tmpl) => (
                    <Card key={tmpl.id}>
                      <CardHead>
                        <div className="flex items-center gap-2.5 min-w-0">
                          <Mail size={14} strokeWidth={1.5} className="text-ink-3 shrink-0" />
                          <div className="min-w-0">
                            <div className="text-[13px] font-medium truncate">
                              {tmpl.name}
                            </div>
                            <div className="text-[11px] text-ink-3 truncate font-mono">
                              {tmpl.subject_template}
                            </div>
                          </div>
                        </div>
                        {tmpl.is_default ? (
                          <Badge kind="final">
                            {t("email_templates.default_badge")}
                          </Badge>
                        ) : null}
                      </CardHead>
                      <CardBody>
                        <div className="text-[12px] text-ink-2 leading-[1.55] font-mono bg-paper-2 border border-line-soft rounded-sm p-2.5 whitespace-pre-wrap line-clamp-4">
                          {tmpl.body_template}
                        </div>
                        <div className="flex flex-wrap gap-1.5 mt-3">
                          <Button
                            size="sm"
                            leadingIcon={<Edit size={11} strokeWidth={1.5} />}
                            onClick={() => openEdit(tmpl)}
                          >
                            {t("common.edit")}
                          </Button>
                          {!tmpl.is_default ? (
                            <>
                              <Button
                                size="sm"
                                leadingIcon={<Star size={11} strokeWidth={1.5} />}
                                onClick={() =>
                                  void setDefault(tmpl.id).catch((e) =>
                                    toast.error(String(e)),
                                  )
                                }
                              >
                                {t("email_templates.set_default")}
                              </Button>
                              <Button
                                size="sm"
                                iconOnly
                                variant="danger"
                                aria-label={t("common.delete")}
                                onClick={() => {
                                  if (confirm(t("common.confirm_delete"))) {
                                    void remove(tmpl.id).catch((e) =>
                                      toast.error(String(e)),
                                    );
                                  }
                                }}
                              >
                                <Trash2 size={11} strokeWidth={1.5} />
                              </Button>
                            </>
                          ) : null}
                        </div>
                      </CardBody>
                    </Card>
                  ))}
                </div>
              )}
            </div>
          );
        })
      )}
    </Page>
  );
}
