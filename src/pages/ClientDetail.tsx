import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../stores/toastStore";
import { useNavigate, useParams } from "react-router-dom";
import { Edit, Plus, Trash2 } from "lucide-react";

import { Page } from "../components/layout/Page";
import { Avatar } from "../components/ui/Avatar";
import { Button } from "../components/ui/Button";
import { Card, CardBody, CardHead } from "../components/ui/Card";
import { EmptyState } from "../components/ui/EmptyState";
import { Field, Input, Select, Textarea } from "../components/ui/Input";
import { Modal } from "../components/ui/Modal";
import { StatusDot } from "../components/ui/StatusDot";
import { Table, Td, Th, THead, Tr } from "../components/ui/Table";
import { Tabs, type TabOption } from "../components/ui/Tabs";
import { StatusBadge } from "../components/invoice/StatusBadge";
import { PaymentStatusBadge } from "../components/invoice/PaymentStatusBadge";
import { AddressListEditor } from "../components/client/AddressListEditor";
import { AddressSummary } from "../components/client/AddressSummary";
import { ContactListEditor } from "../components/client/ContactListEditor";
import { ClientAttributeDatalists } from "../components/client/ClientAttributeDatalists";
import { useMoneyFormat } from "../lib/money";
import { useClientStore } from "../stores/clientStore";
import {
  ipc,
  type ClientAddressDto,
  type ClientDto,
  type ClientJournalEntryDto,
  type ClientNotebookViewDto,
  type ContactEntryDto,
  type EmailLogDto,
  type InvoiceDto,
  type NewJournalEntryDto,
  type PaymentDto,
  type PaymentMethodDto,
  type UpdateClientDto,
  type UpdateJournalEntryDto,
} from "../ipc";

type Tab =
  | "info"
  | "addresses"
  | "notebook"
  | "journal"
  | "invoices"
  | "payments"
  | "emails";

function computeAgeLabel(
  dob: string | null,
  t: (key: string, opts?: Record<string, unknown>) => string,
): string | null {
  if (!dob) return null;
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dob);
  if (!m) return null;
  const birthY = Number(m[1]);
  const birthM = Number(m[2]);
  const birthD = Number(m[3]);
  const today = new Date();
  let age = today.getFullYear() - birthY;
  const monthDiff = today.getMonth() + 1 - birthM;
  if (monthDiff < 0 || (monthDiff === 0 && today.getDate() < birthD)) {
    age -= 1;
  }
  if (age < 0) return null;
  return t("clients.age_years", { count: age });
}

export function ClientDetail() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { id } = useParams<{ id: string }>();
  const [tab, setTab] = useState<Tab>("info");
  const [client, setClient] = useState<ClientDto | null>(null);

  useEffect(() => {
    if (!id) return;
    let cancelled = false;
    ipc
      .clientGet(id)
      .then((c) => {
        if (!cancelled) setClient(c);
      })
      .catch((e) => {
        if (!cancelled) toast.error(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [id]);

  if (!id) return null;
  if (!client) {
    return (
      <Page crumbs={[t("clients.title")]} title="—">
        <EmptyState description={t("common.loading")} />
      </Page>
    );
  }

  const ageLabel = computeAgeLabel(client.date_of_birth, t);
  const languageLabel = client.language
    ? t(`clients.language_${client.language}`)
    : null;
  const tabOptions: TabOption<Tab>[] = [
    { id: "info", label: t("clients.tab_info") },
    { id: "addresses", label: t("clients.tab_addresses") },
    { id: "notebook", label: t("clients.tab_notebook") },
    { id: "journal", label: t("clients.tab_journal") },
    { id: "invoices", label: t("clients.tab_invoices") },
    { id: "payments", label: t("clients.tab_payments") },
    { id: "emails", label: t("clients.tab_emails") },
  ];

  return (
    <Page
      crumbs={[{ label: t("clients.title"), to: "/clients" }, client.name]}
      title={
        <span className="inline-flex items-center gap-3">
          <Avatar name={client.name} size={32} />
          {client.name}
        </span>
      }
      subtitle={
        <span className="text-ink-3">
          {[ageLabel, client.occupation, languageLabel]
            .filter(Boolean)
            .join(" · ") || "—"}
        </span>
      }
      actions={
        <>
          <Button
            leadingIcon={<Plus size={13} strokeWidth={1.5} />}
            onClick={() => navigate("/payments/create")}
          >
            {t("payments.new")}
          </Button>
          <Button
            variant="primary"
            leadingIcon={<Plus size={13} strokeWidth={1.5} />}
            onClick={() => navigate("/invoices/create")}
          >
            {t("invoices.new")}
          </Button>
        </>
      }
    >
      <Tabs<Tab>
        value={tab}
        onChange={setTab}
        options={tabOptions}
        className="mb-5"
      />

      {tab === "info" ? <InfoTab client={client} onSaved={setClient} /> : null}
      {tab === "addresses" ? (
        <AddressesTab client={client} onSaved={setClient} />
      ) : null}
      {tab === "notebook" ? <NotebookTab clientId={client.id} /> : null}
      {tab === "journal" ? <JournalTab clientId={client.id} /> : null}
      {tab === "invoices" ? <InvoicesTab clientId={client.id} /> : null}
      {tab === "payments" ? <PaymentsTab clientId={client.id} /> : null}
      {tab === "emails" ? <EmailsTab clientId={client.id} /> : null}
    </Page>
  );
}

function InfoTab({
  client,
  onSaved,
}: {
  client: ClientDto;
  onSaved: (c: ClientDto) => void;
}) {
  const { t } = useTranslation();
  const {
    clients,
    refresh: refreshClients,
    attributeValues,
    refreshAttributeValues,
  } = useClientStore();
  const [name, setName] = useState(client.name);
  const [emails, setEmails] = useState<ContactEntryDto[]>(client.emails);
  const [phones, setPhones] = useState<ContactEntryDto[]>(client.phones);
  const [notes, setNotes] = useState(client.notes ?? "");
  const [referredBy, setReferredBy] = useState<string | null>(client.referred_by);
  const [dateOfBirth, setDateOfBirth] = useState(client.date_of_birth ?? "");
  const [sex, setSex] = useState(client.sex ?? "");
  const [gender, setGender] = useState(client.gender ?? "");
  const [pronouns, setPronouns] = useState(client.pronouns ?? "");
  const [occupation, setOccupation] = useState(client.occupation ?? "");
  const [language, setLanguage] = useState(client.language ?? "");
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (clients.length === 0) void refreshClients();
    void refreshAttributeValues();
  }, [clients.length, refreshClients, refreshAttributeValues]);

  useEffect(() => {
    setName(client.name);
    setEmails(client.emails);
    setPhones(client.phones);
    setNotes(client.notes ?? "");
    setReferredBy(client.referred_by);
    setDateOfBirth(client.date_of_birth ?? "");
    setSex(client.sex ?? "");
    setGender(client.gender ?? "");
    setPronouns(client.pronouns ?? "");
    setOccupation(client.occupation ?? "");
    setLanguage(client.language ?? "");
  }, [client]);

  const onSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setErr(null);
    setBusy(true);
    try {
      const payload: UpdateClientDto = {
        id: client.id,
        kind: client.kind,
        name: name.trim(),
        contact_name: client.contact_name,
        tax_id: client.tax_id,
        registration_number: client.registration_number,
        emails,
        phones,
        // Addresses are managed in the dedicated Addresses tab; pass them
        // through unchanged so saving Info doesn't wipe them.
        addresses: client.addresses,
        notes: notes.trim() || null,
        referred_by: referredBy,
        date_of_birth: dateOfBirth || null,
        sex: sex || null,
        gender: gender.trim() || null,
        pronouns: pronouns.trim() || null,
        occupation: occupation.trim() || null,
        language: language || null,
      };
      const updated = await ipc.clientUpdate(payload);
      onSaved(updated);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  const referrerOptions = clients.filter((c) => c.id !== client.id);

  return (
    <form onSubmit={onSubmit}>
      <div className="grid grid-cols-1 gap-5 lg:grid-cols-2">
        <Card>
          <CardHead
            title={t("clients.identity")}
            actions={
              saved ? (
                <span className="inline-flex items-center gap-1.5 text-[12px] text-ok-ink">
                  <StatusDot status="ok" /> Enregistré
                </span>
              ) : null
            }
          />
          <CardBody>
            <div className="flex flex-col gap-3.5">
              <Field label={t("common.name")}>
                <Input value={name} onChange={(e) => setName(e.target.value)} required />
              </Field>
              <ContactListEditor
                title={t("clients.emails")}
                value={emails}
                onChange={setEmails}
                type="email"
                addLabel={t("clients.add_email")}
                emptyLabel={t("clients.no_emails")}
              />
              <ContactListEditor
                title={t("clients.phones")}
                value={phones}
                onChange={setPhones}
                type="tel"
                addLabel={t("clients.add_phone")}
                emptyLabel={t("clients.no_phones")}
              />
              <Field label={t("common.notes")}>
                <Textarea
                  rows={2}
                  value={notes}
                  onChange={(e) => setNotes(e.target.value)}
                />
              </Field>
            </div>
          </CardBody>
        </Card>

        <Card>
          <CardHead title={t("clients.demographics")} />
          <CardBody>
            <div className="grid grid-cols-2 gap-3.5">
              <Field label={t("clients.date_of_birth")}>
                <Input
                  mono
                  type="date"
                  value={dateOfBirth}
                  onChange={(e) => setDateOfBirth(e.target.value)}
                />
              </Field>
              <Field label={t("clients.sex")}>
                <Select value={sex} onChange={(e) => setSex(e.target.value)}>
                  <option value="">{t("clients.no_sex")}</option>
                  <option value="female">{t("clients.sex_female")}</option>
                  <option value="male">{t("clients.sex_male")}</option>
                  <option value="intersex">{t("clients.sex_intersex")}</option>
                </Select>
              </Field>
              <Field label={t("clients.gender")}>
                <Input
                  value={gender}
                  onChange={(e) => setGender(e.target.value)}
                  placeholder={t("clients.gender_placeholder") ?? ""}
                  list="gender-suggestions"
                />
              </Field>
              <Field label={t("clients.pronouns")}>
                <Input
                  value={pronouns}
                  onChange={(e) => setPronouns(e.target.value)}
                  placeholder={t("clients.pronouns_placeholder") ?? ""}
                  list="pronouns-suggestions"
                />
              </Field>
              <Field label={t("clients.occupation")}>
                <Input
                  value={occupation}
                  onChange={(e) => setOccupation(e.target.value)}
                  list="occupation-suggestions"
                />
              </Field>
              <Field label={t("clients.language")}>
                <Select value={language} onChange={(e) => setLanguage(e.target.value)}>
                  <option value="">{t("clients.no_language")}</option>
                  <option value="fr">Français</option>
                  <option value="en">English</option>
                  <option value="nl">Nederlands</option>
                  <option value="de">Deutsch</option>
                </Select>
              </Field>
              <Field label={t("clients.referred_by")} className="col-span-2">
                <Select
                  value={referredBy ?? ""}
                  onChange={(e) => setReferredBy(e.target.value || null)}
                >
                  <option value="">{t("clients.no_referrer")}</option>
                  {referrerOptions.map((c) => (
                    <option key={c.id} value={c.id}>
                      {c.name}
                      {c.archived_at ? ` (${t("clients.archived")})` : ""}
                    </option>
                  ))}
                </Select>
              </Field>
            </div>
            {err ? <p className="mt-3 text-[13px] text-danger">{err}</p> : null}
            <div className="mt-5 flex justify-end">
              <Button type="submit" variant="primary" disabled={busy}>
                {t("common.save")}
              </Button>
            </div>
          </CardBody>
        </Card>
      </div>

      <Card className="mt-5">
        <CardHead title={t("clients.addresses")} />
        <CardBody>
          <AddressSummary addresses={client.addresses} />
        </CardBody>
      </Card>

      <ClientAttributeDatalists values={attributeValues} />
    </form>
  );
}

function AddressesTab({
  client,
  onSaved,
}: {
  client: ClientDto;
  onSaved: (c: ClientDto) => void;
}) {
  const { t } = useTranslation();
  const [addresses, setAddresses] = useState<ClientAddressDto[]>(
    client.addresses,
  );
  const [busy, setBusy] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setAddresses(client.addresses);
  }, [client]);

  const dirty =
    JSON.stringify(addresses) !== JSON.stringify(client.addresses);

  const onSave = async () => {
    setBusy(true);
    try {
      // Pass through every other field unchanged — this tab only owns
      // the address list. The backend's UpdateClient replaces the full
      // address set, so omitting fields elsewhere isn't an option.
      const payload: UpdateClientDto = {
        id: client.id,
        kind: client.kind,
        name: client.name,
        contact_name: client.contact_name,
        tax_id: client.tax_id,
        registration_number: client.registration_number,
        emails: client.emails,
        phones: client.phones,
        addresses,
        notes: client.notes,
        referred_by: client.referred_by,
        date_of_birth: client.date_of_birth,
        sex: client.sex,
        gender: client.gender,
        pronouns: client.pronouns,
        occupation: client.occupation,
        language: client.language,
      };
      const updated = await ipc.clientUpdate(payload);
      onSaved(updated);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHead
        title={t("clients.addresses")}
        actions={
          <div className="inline-flex items-center gap-3">
            {saved ? (
              <span className="inline-flex items-center gap-1.5 text-[12px] text-ok-ink">
                <StatusDot status="ok" /> {t("settings.saved")}
              </span>
            ) : null}
            <Button
              type="button"
              variant="primary"
              onClick={onSave}
              disabled={!dirty || busy}
            >
              {t("common.save")}
            </Button>
          </div>
        }
      />
      <CardBody>
        <AddressListEditor value={addresses} onChange={setAddresses} />
      </CardBody>
    </Card>
  );
}

function NotebookTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [view, setView] = useState<ClientNotebookViewDto | null>(null);
  const [contents, setContents] = useState<Record<string, string>>({});
  const [dirty, setDirty] = useState(false);
  const [saved, setSaved] = useState(false);
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const load = () => {
    setErr(null);
    ipc
      .clientNotebookGet(clientId)
      .then((v) => {
        setView(v);
        const map: Record<string, string> = {};
        for (const s of v.sections) map[s.section.id] = s.content;
        setContents(map);
        setDirty(false);
      })
      .catch((e) => toast.error(String(e)));
  };

  useEffect(() => {
    load();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clientId]);

  const updateContent = (sectionId: string, content: string) => {
    setContents((prev) => ({ ...prev, [sectionId]: content }));
    setDirty(true);
  };

  const onSave = async () => {
    if (!view) return;
    setBusy(true);
    setErr(null);
    try {
      await ipc.clientNotebookSave({
        client_id: clientId,
        entries: view.sections.map((s) => ({
          section_id: s.section.id,
          content: contents[s.section.id] ?? "",
        })),
      });
      setDirty(false);
      setSaved(true);
      setTimeout(() => setSaved(false), 1500);
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  if (!view) {
    return err ? (
      <p className="text-[13px] text-danger">{err}</p>
    ) : (
      <EmptyState description={t("common.loading")} />
    );
  }

  if (view.sections.length === 0) {
    return (
      <Card>
        <EmptyState description={t("clients.notebook_no_sections")} />
      </Card>
    );
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-end gap-3">
        {err ? <p className="text-[13px] text-danger">{err}</p> : null}
        {saved ? (
          <span className="inline-flex items-center gap-1.5 text-[12px] text-ok-ink">
            <StatusDot status="ok" /> {t("settings.saved")}
          </span>
        ) : null}
        <Button onClick={onSave} variant="primary" disabled={!dirty || busy}>
          {t("common.save")}
        </Button>
      </div>

      {view.sections.map((s) => (
        <Card key={s.section.id}>
          <CardHead title={s.section.name} />
          <CardBody>
            <Textarea
              rows={6}
              value={contents[s.section.id] ?? ""}
              onChange={(e) => updateContent(s.section.id, e.target.value)}
              placeholder={t("clients.notebook_placeholder") ?? ""}
            />
          </CardBody>
        </Card>
      ))}
    </div>
  );
}

type JournalEditorState =
  | { mode: "closed" }
  | { mode: "create" }
  | { mode: "edit"; entry: ClientJournalEntryDto };

function todayIso(): string {
  return new Date().toISOString().slice(0, 10);
}

function JournalTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<ClientJournalEntryDto[]>([]);
  const [err, setErr] = useState<string | null>(null);
  const [editor, setEditor] = useState<JournalEditorState>({ mode: "closed" });

  const refresh = () => {
    setErr(null);
    ipc
      .journalListForClient(clientId)
      .then(setEntries)
      .catch((e) => toast.error(String(e)));
  };

  useEffect(() => {
    refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [clientId]);

  const onDelete = async (id: string) => {
    if (!confirm(t("common.confirm_delete"))) return;
    try {
      await ipc.journalEntryDelete(id);
      refresh();
    } catch (e) {
      toast.error(String(e));
    }
  };

  return (
    <div className="flex flex-col gap-3">
      <div className="flex items-center justify-end">
        <Button
          variant="primary"
          leadingIcon={<Plus size={13} strokeWidth={1.5} />}
          onClick={() => setEditor({ mode: "create" })}
        >
          {t("clients.journal_new")}
        </Button>
      </div>

      {err ? <p className="text-[13px] text-danger">{err}</p> : null}

      {entries.length === 0 ? (
        <Card>
          <EmptyState description={t("clients.journal_none")} />
        </Card>
      ) : (
        <ul className="flex flex-col gap-3">
          {entries.map((entry) => (
            <li key={entry.id}>
              <Card>
                <CardBody>
                  <div className="flex items-start justify-between mb-2">
                    <span className="font-mono tabular text-[12px] text-ink-3">
                      {entry.entry_date}
                    </span>
                    <div className="flex gap-1">
                      <Button
                        size="sm"
                        iconOnly
                        aria-label={t("common.edit")}
                        onClick={() => setEditor({ mode: "edit", entry })}
                      >
                        <Edit size={11} strokeWidth={1.5} />
                      </Button>
                      <Button
                        size="sm"
                        iconOnly
                        variant="danger"
                        aria-label={t("common.delete")}
                        onClick={() => void onDelete(entry.id)}
                      >
                        <Trash2 size={11} strokeWidth={1.5} />
                      </Button>
                    </div>
                  </div>
                  <p className="whitespace-pre-wrap text-[13px] text-ink">
                    {entry.content}
                  </p>
                </CardBody>
              </Card>
            </li>
          ))}
        </ul>
      )}

      <JournalEntryModal
        clientId={clientId}
        editor={editor}
        onClose={() => {
          setEditor({ mode: "closed" });
          refresh();
        }}
      />
    </div>
  );
}

function JournalEntryModal({
  clientId,
  editor,
  onClose,
}: {
  clientId: string;
  editor: JournalEditorState;
  onClose: () => void;
}) {
  const { t } = useTranslation();
  const initial = editor.mode === "edit" ? editor.entry : null;
  const [date, setDate] = useState(initial?.entry_date ?? todayIso());
  const [content, setContent] = useState(initial?.content ?? "");
  const [err, setErr] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (editor.mode === "closed") return;
    setDate(editor.mode === "edit" ? editor.entry.entry_date : todayIso());
    setContent(editor.mode === "edit" ? editor.entry.content : "");
    setErr(null);
  }, [editor]);

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (content.trim() === "") {
      setErr(t("clients.journal_err_empty"));
      return;
    }
    setBusy(true);
    setErr(null);
    try {
      if (editor.mode === "edit") {
        const payload: UpdateJournalEntryDto = {
          id: editor.entry.id,
          entry_date: date,
          content,
        };
        await ipc.journalEntryUpdate(payload);
      } else {
        const payload: NewJournalEntryDto = {
          client_id: clientId,
          entry_date: date,
          content,
        };
        await ipc.journalEntryCreate(payload);
      }
      onClose();
    } catch (e) {
      toast.error(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Modal
      open={editor.mode !== "closed"}
      onClose={onClose}
      title={initial ? t("clients.journal_edit") : t("clients.journal_new")}
      width={520}
      footer={
        <>
          <Button type="button" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button type="submit" form="journal-form" variant="primary" disabled={busy}>
            {t("common.save")}
          </Button>
        </>
      }
    >
      <form id="journal-form" onSubmit={submit} className="flex flex-col gap-3">
        <Field label={t("common.date")}>
          <Input
            mono
            type="date"
            value={date}
            onChange={(e) => setDate(e.target.value)}
            required
          />
        </Field>
        <Field label={t("clients.journal_content")}>
          <Textarea
            rows={8}
            value={content}
            onChange={(e) => setContent(e.target.value)}
            autoFocus
          />
        </Field>
        {err ? <p className="text-[13px] text-danger">{err}</p> : null}
      </form>
    </Modal>
  );
}

// ---- Invoices tab ----

function InvoicesTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [items, setItems] = useState<InvoiceDto[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const { format } = useMoneyFormat();

  useEffect(() => {
    let cancelled = false;
    setErr(null);
    ipc
      .invoiceList({ client_id: clientId, pagination: { page: 1, per_page: 200 } })
      .then((page) => {
        if (!cancelled) setItems(page.data);
      })
      .catch((e) => {
        if (!cancelled) setErr(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [clientId]);

  if (err) return <p className="text-[13px] text-danger">{err}</p>;
  if (!items) return <EmptyState description={t("common.loading")} />;
  if (items.length === 0)
    return (
      <Card>
        <EmptyState description={t("clients.no_invoices")} />
      </Card>
    );

  return (
    <Card>
      <Table>
        <THead>
          <Tr>
            <Th>{t("invoices.number")}</Th>
            <Th>{t("common.date")}</Th>
            <Th numeric>{t("invoices.total")}</Th>
            <Th>{t("common.status")}</Th>
            <Th>{t("invoices.payment_status")}</Th>
          </Tr>
        </THead>
        <tbody>
          {items.map((inv) => (
            <Tr
              key={inv.id}
              onClick={() => navigate(`/invoices/${inv.id}/edit`)}
              className="cursor-pointer hover:bg-paper-2"
            >
              <Td mono>#{inv.number ?? "—"}</Td>
              <Td muted>{inv.date}</Td>
              <Td numeric>{format(inv.total)}</Td>
              <Td>
                <StatusBadge status={inv.status} />
              </Td>
              <Td>
                {inv.payment_status ? (
                  <PaymentStatusBadge
                    paymentStatus={inv.payment_status}
                    rawStatus={inv.status}
                  />
                ) : (
                  "—"
                )}
              </Td>
            </Tr>
          ))}
        </tbody>
      </Table>
    </Card>
  );
}

// ---- Payments tab ----

function paymentMethodLabel(
  method: PaymentMethodDto,
  t: (k: string) => string,
): string {
  switch (method.kind) {
    case "BankTransfer":
      return t("payments.method_banktransfer");
    case "Cash":
      return t("payments.method_cash");
    case "Check":
      return t("payments.method_check");
    case "Card":
      return t("payments.method_card");
    case "Other":
      return method.detail || t("payments.method_other");
  }
}

function PaymentsTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [items, setItems] = useState<PaymentDto[] | null>(null);
  const [err, setErr] = useState<string | null>(null);
  const { format } = useMoneyFormat();

  useEffect(() => {
    let cancelled = false;
    setErr(null);
    ipc
      .paymentList({ client_id: clientId })
      .then((rows) => {
        if (!cancelled) setItems(rows);
      })
      .catch((e) => {
        if (!cancelled) setErr(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [clientId]);

  if (err) return <p className="text-[13px] text-danger">{err}</p>;
  if (!items) return <EmptyState description={t("common.loading")} />;
  if (items.length === 0)
    return (
      <Card>
        <EmptyState description={t("clients.no_payments")} />
      </Card>
    );

  return (
    <Card>
      <Table>
        <THead>
          <Tr>
            <Th>{t("common.date")}</Th>
            <Th>{t("payments.method")}</Th>
            <Th numeric>{t("payments.amount")}</Th>
            <Th>{t("payments.reference")}</Th>
          </Tr>
        </THead>
        <tbody>
          {items.map((p) => (
            <Tr
              key={p.id}
              onClick={() => navigate(`/payments/${p.id}/edit`)}
              className="cursor-pointer hover:bg-paper-2"
            >
              <Td muted>{p.date}</Td>
              <Td>{paymentMethodLabel(p.method, t)}</Td>
              <Td numeric>{format(p.amount)}</Td>
              <Td muted>{p.reference ?? "—"}</Td>
            </Tr>
          ))}
        </tbody>
      </Table>
    </Card>
  );
}

// ---- Emails tab ----

function formatSentAt(iso: string): string {
  const d = new Date(iso);
  if (Number.isNaN(d.getTime())) return iso;
  return d.toLocaleString();
}

function EmailsTab({ clientId }: { clientId: string }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const [items, setItems] = useState<EmailLogDto[] | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    setErr(null);
    ipc
      .emailLogListForClient(clientId)
      .then((rows) => {
        if (!cancelled) setItems(rows);
      })
      .catch((e) => {
        if (!cancelled) setErr(String(e));
      });
    return () => {
      cancelled = true;
    };
  }, [clientId]);

  if (err) return <p className="text-[13px] text-danger">{err}</p>;
  if (!items) return <EmptyState description={t("common.loading")} />;
  if (items.length === 0)
    return (
      <Card>
        <EmptyState description={t("clients.no_emails")} />
      </Card>
    );

  return (
    <Card>
      <Table>
        <THead>
          <Tr>
            <Th>{t("clients.email_sent_at")}</Th>
            <Th>{t("clients.email_subject")}</Th>
            <Th>{t("clients.email_to")}</Th>
            <Th>{t("clients.email_invoice")}</Th>
          </Tr>
        </THead>
        <tbody>
          {items.map((e) => (
            <Tr key={e.id}>
              <Td muted>{formatSentAt(e.sent_at)}</Td>
              <Td>{e.subject}</Td>
              <Td muted>{e.to_address}</Td>
              <Td>
                {e.invoice_id ? (
                  <button
                    type="button"
                    onClick={() => navigate(`/invoices/${e.invoice_id}/edit`)}
                    className="text-ink hover:underline cursor-pointer"
                  >
                    {t("clients.email_view_invoice")}
                  </button>
                ) : (
                  "—"
                )}
              </Td>
            </Tr>
          ))}
        </tbody>
      </Table>
    </Card>
  );
}
