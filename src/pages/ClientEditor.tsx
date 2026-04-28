import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate, useParams } from "react-router-dom";

import { Page } from "../components/layout/Page";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { Field, Input, Select, Textarea } from "../components/ui/Input";
import { ContactListEditor } from "../components/client/ContactListEditor";
import { ClientAttributeDatalists } from "../components/client/ClientAttributeDatalists";
import { useClientStore } from "../stores/clientStore";
import type { NewClientDto, UpdateClientDto } from "../ipc";

const empty: NewClientDto = {
  name: "",
  emails: [],
  phones: [],
  address: null,
  notes: null,
  referred_by: null,
  date_of_birth: null,
  sex: null,
  gender: null,
  pronouns: null,
  occupation: null,
  language: null,
};

export function ClientEditor() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id?: string }>();
  const editing = Boolean(id);

  const { clients, refresh, create, update, attributeValues, refreshAttributeValues } =
    useClientStore();

  useEffect(() => {
    if (clients.length === 0) void refresh();
    void refreshAttributeValues();
  }, [clients.length, refresh, refreshAttributeValues]);

  const existing = useMemo(() => clients.find((c) => c.id === id), [clients, id]);

  const [form, setForm] = useState<NewClientDto>(empty);
  const [err, setErr] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (existing) {
      setForm({
        name: existing.name,
        emails: existing.emails,
        phones: existing.phones,
        address: existing.address,
        notes: existing.notes,
        referred_by: existing.referred_by,
        date_of_birth: existing.date_of_birth,
        sex: existing.sex,
        gender: existing.gender,
        pronouns: existing.pronouns,
        occupation: existing.occupation,
        language: existing.language,
      });
    }
  }, [existing]);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setSubmitting(true);
    try {
      if (editing && existing) {
        const payload: UpdateClientDto = { id: existing.id, ...form };
        await update(payload);
        navigate(`/clients/${existing.id}`);
      } else {
        await create(form);
        navigate("/clients");
      }
    } catch (e) {
      setErr(String(e));
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Page
      crumbs={[
        "Cabinet Lemaire",
        t("clients.title"),
        editing ? existing?.name ?? "—" : t("clients.new"),
      ]}
      title={editing ? t("clients.edit") : t("clients.new")}
    >
      <form onSubmit={submit} className="max-w-3xl">
        <Card>
          <CardHead title={t("clients.title")} />
          <CardBody>
            <div className="flex flex-col gap-4">
              <Field label={t("common.name")}>
                <Input
                  value={form.name}
                  onChange={(e) => setForm({ ...form, name: e.target.value })}
                  required
                />
              </Field>

              <ContactListEditor
                title={t("clients.emails")}
                value={form.emails}
                onChange={(emails) => setForm({ ...form, emails })}
                type="email"
                addLabel={t("clients.add_email")}
                emptyLabel={t("clients.no_emails")}
              />

              <ContactListEditor
                title={t("clients.phones")}
                value={form.phones}
                onChange={(phones) => setForm({ ...form, phones })}
                type="tel"
                addLabel={t("clients.add_phone")}
                emptyLabel={t("clients.no_phones")}
              />

              <Field label={t("common.address")}>
                <Input
                  value={form.address ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, address: e.target.value || null })
                  }
                />
              </Field>

              <Field label={t("common.notes")}>
                <Textarea
                  rows={2}
                  value={form.notes ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, notes: e.target.value || null })
                  }
                />
              </Field>

              <div className="grid grid-cols-1 gap-3.5 sm:grid-cols-2">
                <Field label={t("clients.date_of_birth")}>
                  <Input
                    mono
                    type="date"
                    value={form.date_of_birth ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, date_of_birth: e.target.value || null })
                    }
                  />
                </Field>
                <Field label={t("clients.sex")}>
                  <Select
                    value={form.sex ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, sex: e.target.value || null })
                    }
                  >
                    <option value="">{t("clients.no_sex")}</option>
                    <option value="female">{t("clients.sex_female")}</option>
                    <option value="male">{t("clients.sex_male")}</option>
                    <option value="intersex">{t("clients.sex_intersex")}</option>
                  </Select>
                </Field>
                <Field label={t("clients.gender")}>
                  <Input
                    value={form.gender ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, gender: e.target.value || null })
                    }
                    placeholder={t("clients.gender_placeholder") ?? ""}
                    list="gender-suggestions"
                  />
                </Field>
                <Field label={t("clients.pronouns")}>
                  <Input
                    value={form.pronouns ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, pronouns: e.target.value || null })
                    }
                    placeholder={t("clients.pronouns_placeholder") ?? ""}
                    list="pronouns-suggestions"
                  />
                </Field>
                <Field label={t("clients.occupation")}>
                  <Input
                    value={form.occupation ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, occupation: e.target.value || null })
                    }
                    list="occupation-suggestions"
                  />
                </Field>
                <Field label={t("clients.language")}>
                  <Select
                    value={form.language ?? ""}
                    onChange={(e) =>
                      setForm({ ...form, language: e.target.value || null })
                    }
                  >
                    <option value="">{t("clients.no_language")}</option>
                    <option value="fr">Français</option>
                    <option value="en">English</option>
                    <option value="nl">Nederlands</option>
                    <option value="de">Deutsch</option>
                  </Select>
                </Field>
              </div>

              <Field label={t("clients.referred_by")}>
                <Select
                  value={form.referred_by ?? ""}
                  onChange={(e) =>
                    setForm({ ...form, referred_by: e.target.value || null })
                  }
                >
                  <option value="">{t("clients.no_referrer")}</option>
                  {clients
                    .filter((c) => c.id !== id)
                    .map((c) => (
                      <option key={c.id} value={c.id}>
                        {c.name}
                        {c.archived_at ? ` (${t("clients.archived")})` : ""}
                      </option>
                    ))}
                </Select>
              </Field>
            </div>
            {err ? <p className="mt-3 text-[13px] text-danger">{err}</p> : null}
          </CardBody>
        </Card>
        <ClientAttributeDatalists values={attributeValues} />
        <div className="mt-4 flex justify-end gap-2">
          <Button
            type="button"
            onClick={() =>
              editing && existing ? navigate(`/clients/${existing.id}`) : navigate("/clients")
            }
          >
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
