import { useEffect, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "../../stores/toastStore";

import { Button } from "./Button";
import { Input } from "./Input";
import { Modal } from "./Modal";

interface ConfirmModalProps {
  open: boolean;
  title: ReactNode;
  description?: ReactNode;
  confirmLabel?: ReactNode;
  cancelLabel?: ReactNode;
  /** Visual emphasis of the confirm button — `danger` for destructive ops. */
  tone?: "primary" | "danger";
  /**
   * If set, the user must type this exact phrase (case-insensitive, trimmed)
   * to enable the confirm button. Use for high-stakes actions like restore
   * or delete, where an extra friction step is worth the safety.
   */
  requireText?: string;
  onConfirm: () => void | Promise<void>;
  onClose: () => void;
}

/**
 * App-styled replacement for the browser's `confirm()` dialog. Use for any
 * destructive or one-way action (cancel invoice, delete payment, etc.) so
 * the user gets the design system's chrome instead of a native popup.
 *
 * Tracks its own `submitting` state so async confirm handlers (typical IPC
 * calls) disable the button until the promise resolves; closes on success
 * automatically. Errors thrown by the handler are caught and re-displayed
 * via `alert` for now — callers that want richer error UI can wrap the
 * handler themselves.
 */
export function ConfirmModal({
  open,
  title,
  description,
  confirmLabel,
  cancelLabel,
  tone = "primary",
  requireText,
  onConfirm,
  onClose,
}: ConfirmModalProps) {
  const { t } = useTranslation();
  const [submitting, setSubmitting] = useState(false);
  const [typed, setTyped] = useState("");

  useEffect(() => {
    if (!open) setTyped("");
  }, [open]);

  const matches =
    !requireText ||
    typed.trim().toLowerCase() === requireText.trim().toLowerCase();

  const handleConfirm = async () => {
    setSubmitting(true);
    try {
      await onConfirm();
      onClose();
    } catch (e) {
      toast.error(e);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <Modal
      open={open}
      onClose={submitting ? () => {} : onClose}
      title={title}
      width={460}
      footer={
        <>
          <Button type="button" disabled={submitting} onClick={onClose}>
            {cancelLabel ?? t("common.cancel")}
          </Button>
          <Button
            type="button"
            variant={tone === "danger" ? "danger" : "primary"}
            disabled={submitting || !matches}
            onClick={() => void handleConfirm()}
          >
            {confirmLabel ?? t("common.confirm")}
          </Button>
        </>
      }
    >
      {description ? (
        <p className="text-[13px] text-ink-2 leading-[1.55]">{description}</p>
      ) : null}
      {requireText ? (
        <div className="mt-3">
          <p className="text-[12px] text-ink-3 mb-1.5">
            {t("common.confirm_type_prompt", { phrase: requireText })}
          </p>
          <Input
            mono
            autoFocus
            value={typed}
            onChange={(e) => setTyped(e.target.value)}
            placeholder={requireText}
            disabled={submitting}
            onKeyDown={(e) => {
              if (e.key === "Enter" && matches && !submitting) {
                e.preventDefault();
                void handleConfirm();
              }
            }}
          />
        </div>
      ) : null}
    </Modal>
  );
}
