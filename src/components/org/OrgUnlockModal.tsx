import { FormEvent, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
import { Modal } from "../ui/Modal";
import { OrgAvatar } from "./OrgAvatar";
import { errorCodeOf, translateError } from "../../ipc/errorCatalog";

interface Props {
  /** Org code to unlock; modal is hidden when `null`. */
  code: string | null;
  /** Pre-supplied error (e.g. wrong password from a prior attempt). */
  error?: string | null;
  onClose: () => void;
  /** Called with the user's password and whether to cache it in the OS
   *  keyring. Throws on wrong password — caught here and displayed. */
  onSubmit: (password: string, remember: boolean) => Promise<void>;
}

export function OrgUnlockModal({ code, error: externalError, onClose, onSubmit }: Props) {
  const { t } = useTranslation();
  const [password, setPassword] = useState("");
  const [remember, setRemember] = useState(false);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (code === null) {
      setPassword("");
      setRemember(false);
      setError(null);
      setSubmitting(false);
    } else {
      setError(externalError ?? null);
    }
  }, [code, externalError]);

  if (code === null) return null;

  async function handleSubmit(e: FormEvent) {
    e.preventDefault();
    if (!password) return;
    setSubmitting(true);
    setError(null);
    try {
      await onSubmit(password, remember);
    } catch (err) {
      const wrong = errorCodeOf(err) === "org_wrong_password";
      setError(translateError(err, t));
      if (wrong) {
        // Re-focus the field by clearing it — feels less stuck than a
        // password that's flagged invalid still in the box.
        setPassword("");
      }
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <Modal
      open
      onClose={onClose}
      title={t("org_unlock.title", { defaultValue: "Unlock {{code}}", code })}
    >
      <form onSubmit={handleSubmit} className="space-y-5">
        <div className="flex items-center gap-4">
          <OrgAvatar code={code} size="lg" />
          <p className="text-sm text-ink-3 flex-1">
            {t("org_unlock.description", {
              defaultValue: "This organisation is protected by a password.",
            })}
          </p>
        </div>

        <div className="space-y-1.5">
          <label htmlFor="org-unlock-password" className="text-sm font-medium text-ink">
            {t("org_unlock.password_label", { defaultValue: "Password" })}
          </label>
          <Input
            id="org-unlock-password"
            type="password"
            autoFocus
            autoComplete="current-password"
            value={password}
            onChange={(e) => setPassword(e.currentTarget.value)}
            required
          />
          {error ? (
            <p className="text-[12px] text-danger" role="alert">
              {error}
            </p>
          ) : null}
        </div>

        <label className="flex items-start gap-2.5 cursor-pointer">
          <input
            type="checkbox"
            checked={remember}
            onChange={(e) => setRemember(e.currentTarget.checked)}
            className="mt-1"
          />
          <span className="text-sm">
            <span className="block text-ink">
              {t("org_unlock.remember_label", {
                defaultValue: "Remember on this device",
              })}
            </span>
            <span className="block text-[11px] text-ink-3">
              {t("org_unlock.remember_hint", {
                defaultValue:
                  "Stored in your OS keychain so you don't have to re-enter it on next launch.",
              })}
            </span>
          </span>
        </label>

        <div className="flex justify-end gap-2">
          <Button type="button" onClick={onClose} variant="ghost">
            {t("common.cancel", { defaultValue: "Cancel" })}
          </Button>
          <Button type="submit" variant="primary" disabled={submitting || !password}>
            {submitting
              ? t("org_unlock.submitting", { defaultValue: "Unlocking…" })
              : t("org_unlock.submit", { defaultValue: "Unlock" })}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
