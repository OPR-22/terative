import { FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Lock, Unlock } from "lucide-react";

import { Button } from "../ui/Button";
import { ConfirmModal } from "../ui/ConfirmModal";
import { Input } from "../ui/Input";
import { Modal } from "../ui/Modal";
import { ipc } from "../../ipc";
import { translateError } from "../../ipc/errorCatalog";
import { toast } from "../../stores/toastStore";
import { useOrgStore } from "../../stores/orgStore";

type Action = "set" | "change" | "remove";

/**
 * Settings panel for managing the active org's SQLCipher passphrase.
 *
 * Three flows:
 * - **Set**: plaintext → encrypted (asks for new password).
 * - **Change**: encrypted → encrypted with new key (asks for current + new).
 * - **Remove**: encrypted → plaintext (asks for current).
 *
 * After a successful rekey the active org has been closed server-side, so
 * we close the app's session and route back to the picker — the user
 * re-opens with the new (or no) password.
 */
export function OrgPasswordSection() {
  const { t } = useTranslation();
  const { activeOrg, orgs, refresh, close } = useOrgStore();
  const [action, setAction] = useState<Action | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!activeOrg) return null;
  const summary = orgs.find((o) => o.code === activeOrg.code);
  const encrypted = summary?.has_password ?? false;

  async function handleDone() {
    setAction(null);
    // Server closed the org during rekey; mirror that on the client.
    await close();
  }

  return (
    <section>
      <h2 className="mb-3 text-lg font-semibold text-fg flex items-center gap-2">
        {encrypted ? <Lock size={16} /> : <Unlock size={16} />}
        {t("org_password.section_title", { defaultValue: "Database password" })}
      </h2>
      <p className="mb-3 text-sm text-fg-muted">
        {encrypted
          ? t("org_password.description_locked", {
              defaultValue: "This organisation is encrypted.",
            })
          : t("org_password.description_unlocked", {
              defaultValue:
                "This organisation is currently unencrypted. Setting a password encrypts the database file at rest.",
            })}
      </p>
      <div className="flex flex-wrap gap-2">
        {encrypted ? (
          <>
            <Button onClick={() => setAction("change")}>
              {t("org_password.change_button", { defaultValue: "Change password" })}
            </Button>
            <Button variant="ghost" onClick={() => setAction("remove")}>
              {t("org_password.remove_button", { defaultValue: "Remove password" })}
            </Button>
          </>
        ) : (
          <Button onClick={() => setAction("set")}>
            {t("org_password.set_button", { defaultValue: "Set a password" })}
          </Button>
        )}
      </div>

      {action === "set" || action === "change" ? (
        <PasswordModal
          action={action}
          code={activeOrg.code}
          onClose={() => setAction(null)}
          onDone={handleDone}
        />
      ) : null}

      <ConfirmModal
        open={action === "remove"}
        title={t("org_password.remove_title", { defaultValue: "Remove password" })}
        description={t("org_password.remove_warning", {
          defaultValue:
            "The database will be decrypted on disk. Anyone with file access could read it.",
        })}
        confirmLabel={t("org_password.save_remove", { defaultValue: "Decrypt database" })}
        tone="danger"
        onClose={() => setAction(null)}
        onConfirm={async () => {
          try {
            // current_password comes from the OS keyring via the backend
            // (the org is currently active, so it was just unlocked).
            await ipc.orgRemovePassword(activeOrg.code, null);
            toast.success(
              t("org_password.success_remove", {
                defaultValue: "Password removed. The org is now plaintext.",
              }),
            );
            await handleDone();
          } catch (e) {
            toast.error(translateError(e, t));
          }
        }}
      />
    </section>
  );
}

interface PasswordModalProps {
  action: "set" | "change";
  code: string;
  onClose: () => void;
  onDone: () => Promise<void> | void;
}

function PasswordModal({ action, code, onClose, onDone }: PasswordModalProps) {
  const { t } = useTranslation();
  const [current, setCurrent] = useState("");
  const [next, setNext] = useState("");
  const [confirm, setConfirm] = useState("");
  const [remember, setRemember] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const title =
    action === "set"
      ? t("org_password.set_title", { defaultValue: "Encrypt this organisation" })
      : t("org_password.change_title", { defaultValue: "Change password" });
  const submitLabel =
    action === "set"
      ? t("org_password.save_set", { defaultValue: "Encrypt database" })
      : t("org_password.save_change", { defaultValue: "Change password" });

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (next.length < 4) {
      setError(
        t("org_password.password_too_short", {
          defaultValue: "Password must be at least 4 characters.",
        }),
      );
      return;
    }
    if (next !== confirm) {
      setError(
        t("org_password.passwords_mismatch", {
          defaultValue: "Passwords do not match.",
        }),
      );
      return;
    }
    setError(null);
    setSubmitting(true);
    try {
      await ipc.orgSetPassword(
        code,
        action === "change" ? (current.length > 0 ? current : null) : null,
        next,
        remember,
      );
      toast.success(
        action === "set"
          ? t("org_password.success_set", {
              defaultValue: "Password set. The org is now encrypted.",
            })
          : t("org_password.success_change", {
              defaultValue: "Password changed.",
            }),
      );
      await onDone();
    } catch (err) {
      setError(translateError(err, t));
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal open onClose={onClose} title={title}>
      <form onSubmit={handleSubmit} className="space-y-4">
        {action === "change" ? (
          <div className="space-y-1.5">
            <label
              htmlFor="org-current-password"
              className="text-sm font-medium text-ink"
            >
              {t("org_password.current_password_label", {
                defaultValue: "Current password",
              })}
            </label>
            <Input
              id="org-current-password"
              type="password"
              autoComplete="current-password"
              value={current}
              onChange={(e) => setCurrent(e.currentTarget.value)}
            />
            <p className="text-[11px] text-ink-3">
              {t("org_password.remember_hint", {
                defaultValue:
                  "Leave blank to use the password cached in your OS keychain.",
              })}
            </p>
          </div>
        ) : null}

        <div className="space-y-1.5">
          <label htmlFor="org-new-password" className="text-sm font-medium text-ink">
            {t("org_password.new_password_label", { defaultValue: "New password" })}
          </label>
          <Input
            id="org-new-password"
            type="password"
            autoComplete="new-password"
            autoFocus
            value={next}
            onChange={(e) => setNext(e.currentTarget.value)}
            required
          />
        </div>

        <div className="space-y-1.5">
          <label
            htmlFor="org-new-password-confirm"
            className="text-sm font-medium text-ink"
          >
            {t("org_password.new_password_confirm_label", {
              defaultValue: "Confirm new password",
            })}
          </label>
          <Input
            id="org-new-password-confirm"
            type="password"
            autoComplete="new-password"
            value={confirm}
            onChange={(e) => setConfirm(e.currentTarget.value)}
            required
          />
        </div>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.currentTarget.checked)}
          />
          <span className="text-sm text-ink">
            {t("org_password.remember_label", {
              defaultValue: "Remember on this device",
            })}
          </span>
        </label>

        {error ? (
          <p className="text-[12px] text-danger" role="alert">
            {error}
          </p>
        ) : null}

        <div className="flex justify-end gap-2">
          <Button type="button" onClick={onClose} variant="ghost">
            {t("common.cancel", { defaultValue: "Cancel" })}
          </Button>
          <Button type="submit" variant="primary" disabled={submitting || !next}>
            {submitting
              ? t("org_password.saving", { defaultValue: "Working…" })
              : submitLabel}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
