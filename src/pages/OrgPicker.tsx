import { FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { Plus, Trash2 } from "lucide-react";

import { OrgAvatar } from "../components/org/OrgAvatar";
import { Button } from "../components/ui/Button";
import { ConfirmModal } from "../components/ui/ConfirmModal";
import { Input } from "../components/ui/Input";
import { Modal } from "../components/ui/Modal";
import { translateError } from "../ipc/errorCatalog";
import { toast } from "../stores/toastStore";
import { useOrgStore } from "../stores/orgStore";

/** Live filter as user types — strip illegal chars (preserves case). */
function sanitizeInput(s: string): string {
  return s.replace(/[^a-zA-Z0-9_-]/g, "");
}

export function OrgPicker() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { orgs, refresh, open, delete: deleteOrg, create } = useOrgStore();
  const [createOpen, setCreateOpen] = useState(false);
  const [createCode, setCreateCode] = useState("");
  const [creating, setCreating] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const empty = orgs.length === 0;

  async function handleOpen(code: string) {
    try {
      await open(code);
      navigate("/dashboard");
    } catch (e) {
      toast.error(translateError(e, t));
    }
  }

  async function handleCreate(ev: FormEvent) {
    ev.preventDefault();
    const code = createCode.trim();
    if (!code) return;
    setCreating(true);
    try {
      const summary = await create(code);
      setCreateOpen(false);
      setCreateCode("");
      await open(summary.code);
      navigate("/dashboard");
    } catch (e) {
      toast.error(translateError(e, t));
    } finally {
      setCreating(false);
    }
  }

  async function handleDelete(code: string) {
    try {
      await deleteOrg(code);
      setConfirmDelete(null);
    } catch (e) {
      toast.error(translateError(e, t));
    }
  }

  return (
    <div className="min-h-screen bg-paper text-ink flex items-center justify-center p-8">
      <div className="w-full max-w-md space-y-8">
        <header className="text-center space-y-2">
          <h1 className="text-2xl font-semibold">
            {t("org_picker.title", { defaultValue: "Choose an organisation" })}
          </h1>
          <p className="text-sm text-ink-3">
            {empty
              ? t("org_picker.empty_subtitle", {
                  defaultValue: "Create your first org to get started.",
                })
              : t("org_picker.subtitle", {
                  defaultValue: "Open an existing org or create a new one.",
                })}
          </p>
        </header>

        <ul className="grid grid-cols-2 sm:grid-cols-3 gap-4">
          {orgs.map((org) => (
            <li key={org.code} className="group relative">
              <button
                type="button"
                onClick={() => handleOpen(org.code)}
                className="w-full flex flex-col items-center gap-2 p-3 rounded-lg hover:bg-paper-2 transition cursor-pointer"
              >
                <OrgAvatar code={org.code} size="xl" />
                <span className="text-[13px] font-medium text-ink truncate max-w-full">
                  {org.code}
                </span>
              </button>
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  setConfirmDelete(org.code);
                }}
                aria-label={t("org_picker.delete_aria", {
                  defaultValue: "Delete org",
                })}
                className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 p-1.5 rounded-md text-ink-3 hover:text-danger hover:bg-paper-3 transition cursor-pointer"
              >
                <Trash2 size={14} />
              </button>
            </li>
          ))}
          <li>
            <button
              type="button"
              onClick={() => setCreateOpen(true)}
              aria-label={
                empty
                  ? t("org_picker.create_first", {
                      defaultValue: "Create your first org",
                    })
                  : t("org_picker.create_new", { defaultValue: "Create new org" })
              }
              className="w-full flex flex-col items-center gap-2 p-3 rounded-lg hover:bg-paper-2 transition cursor-pointer"
            >
              <span
                className="grid place-items-center w-16 h-16 rounded-full border-2 border-dashed border-line text-ink-3 hover:text-accent-ink hover:border-accent transition"
                aria-hidden
              >
                <Plus size={28} strokeWidth={1.5} />
              </span>
              <span className="text-[13px] font-medium text-ink-3 truncate max-w-full">
                {empty
                  ? t("org_picker.create_first", {
                      defaultValue: "Create your first org",
                    })
                  : t("org_picker.create_new_short", { defaultValue: "New" })}
              </span>
            </button>
          </li>
        </ul>
      </div>

      <Modal
        open={createOpen}
        onClose={() => setCreateOpen(false)}
        title={t("org_picker.create_title", { defaultValue: "Create new org" })}
      >
        <form onSubmit={handleCreate} className="space-y-5">
          <div className="flex items-center gap-4">
            <OrgAvatar code={createCode || "?"} size="lg" />
            <div className="flex-1 space-y-1.5">
              <label htmlFor="org-code" className="text-sm font-medium text-ink">
                {t("org_picker.create_code_label", { defaultValue: "Organisation code" })}
              </label>
              <Input
                id="org-code"
                value={createCode}
                onChange={(e) => setCreateCode(sanitizeInput(e.currentTarget.value))}
                placeholder={t("org_picker.create_code_placeholder", {
                  defaultValue: "acme_corp",
                })}
                autoFocus
                autoComplete="off"
                spellCheck={false}
                required
                pattern="[a-zA-Z0-9_-]+"
                maxLength={50}
              />
              <p className="text-[11px] text-ink-3">
                {t("org_picker.create_code_hint", {
                  defaultValue:
                    "Lowercase letters, digits, underscore, hyphen. No spaces.",
                })}
              </p>
            </div>
          </div>
          <div className="flex justify-end gap-2">
            <Button type="button" onClick={() => setCreateOpen(false)} variant="ghost">
              {t("common.cancel", { defaultValue: "Cancel" })}
            </Button>
            <Button
              type="submit"
              variant="primary"
              disabled={creating || !createCode.trim()}
            >
              {creating
                ? t("common.creating", { defaultValue: "Creating…" })
                : t("common.create", { defaultValue: "Create" })}
            </Button>
          </div>
        </form>
      </Modal>

      <ConfirmModal
        open={confirmDelete !== null}
        onClose={() => setConfirmDelete(null)}
        onConfirm={async () => {
          if (confirmDelete) await handleDelete(confirmDelete);
        }}
        title={t("org_picker.delete_title", { defaultValue: "Delete this org?" })}
        description={t("org_picker.delete_message", {
          defaultValue:
            "All data, backups and PDFs in '{{code}}' will be permanently removed. This cannot be undone.",
          code: confirmDelete ?? "",
        })}
        confirmLabel={t("common.delete", { defaultValue: "Delete" })}
        tone="danger"
        requireText={confirmDelete ?? undefined}
      />
    </div>
  );
}
