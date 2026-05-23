import { useEffect, type ReactNode } from "react";
import { X } from "lucide-react";

interface ModalProps {
  open: boolean;
  onClose: () => void;
  title?: ReactNode;
  subtitle?: ReactNode;
  width?: number | string;
  children: ReactNode;
  footer?: ReactNode;
}

export function Modal({
  open,
  onClose,
  title,
  subtitle,
  width = 520,
  children,
  footer,
}: ModalProps) {
  useEffect(() => {
    if (!open) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;

  return (
    <div
      className="fixed inset-0 z-50 flex items-center justify-center bg-ink/30 p-4"
      onClick={onClose}
    >
      <div
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
        className="bg-paper border border-line rounded-card shadow-card max-h-[90vh] flex flex-col overflow-hidden"
        style={{ width }}
      >
        {(title || subtitle) && (
          <div className="flex items-start justify-between gap-3 px-5 py-3.5 border-b border-line-soft">
            <div className="min-w-0">
              {title ? <div className="text-[14px] font-medium text-ink">{title}</div> : null}
              {subtitle ? <div className="text-[12px] text-ink-3 mt-0.5">{subtitle}</div> : null}
            </div>
            <button
              type="button"
              onClick={onClose}
              className="text-ink-3 hover:text-ink p-1 -m-1"
              aria-label="Fermer"
            >
              <X size={14} />
            </button>
          </div>
        )}
        <div className="flex-1 overflow-y-auto px-5 py-4">{children}</div>
        {footer ? (
          <div className="flex justify-end gap-2 px-5 py-3 border-t border-line-soft bg-paper-2">
            {footer}
          </div>
        ) : null}
      </div>
    </div>
  );
}
