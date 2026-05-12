import { create } from "zustand";

import { translateError } from "../ipc/errorCatalog";

export type ToastKind = "success" | "error" | "warn" | "info" | "neutral";

export interface ToastAction {
  /** Button label, e.g. "Réessayer", "Annuler". */
  label: string;
  /** Invoked when the user clicks the action; the toast dismisses
   *  automatically right after, regardless of what the handler does. */
  onClick: () => void;
}

export interface Toast {
  id: string;
  kind: ToastKind;
  title: string;
  description?: string;
  action?: ToastAction;
  /** Persistent toasts have no progress bar and don't auto-dismiss. */
  persistent: boolean;
  /** Auto-dismiss duration in ms. Ignored when `persistent` is true. */
  duration: number;
}

/// Default visible durations per the design spec.
const DEFAULT_DURATIONS: Record<ToastKind, number> = {
  success: 5000,
  info: 5000,
  neutral: 5000,
  warn: 7000,
  error: 3000,
};

/// Maximum visible toasts at once. Anything beyond is held in the store
/// but not rendered until the head pops.
export const MAX_VISIBLE_TOASTS = 3;

interface ToastState {
  toasts: Toast[];
  push: (toast: Toast) => void;
  dismiss: (id: string) => void;
  clear: () => void;
}

const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  push: (toast) => set((s) => ({ toasts: [...s.toasts, toast] })),
  dismiss: (id) =>
    set((s) => ({ toasts: s.toasts.filter((t) => t.id !== id) })),
  clear: () => set({ toasts: [] }),
}));

export { useToastStore };

interface ToastOptions {
  /** Optional CTA — when present and non-persistent, replaces the close
   *  button. For `error`, providing an action implicitly makes the toast
   *  persistent unless `persistent` is explicitly set to `false`. */
  action?: ToastAction;
  /** Override default auto-dismiss behavior. */
  persistent?: boolean;
}

function makeId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `${Date.now()}-${Math.random()}`;
}

function build(
  kind: ToastKind,
  title: string,
  description?: string,
  options?: ToastOptions,
): Toast {
  // Per spec: error + action defaults to persistent. Other kinds default
  // to the table durations. Caller can always override with `persistent`.
  const persistent =
    options?.persistent ?? (kind === "error" && Boolean(options?.action));
  return {
    id: makeId(),
    kind,
    title,
    description,
    action: options?.action,
    persistent,
    duration: DEFAULT_DURATIONS[kind],
  };
}

/// Coerce any caller-supplied title to a string. When the value isn't a
/// string (e.g. an `IpcError`, an `Error`, or an unknown thrown value), we
/// route it through `translateError` so legacy `toast.error(e)` and
/// `toast.error(e)` call sites all produce localized text instead of a raw
/// error code.
function resolveTitle(title: unknown): string {
  if (typeof title === "string") return title;
  return translateError(title);
}

/// Imperative helper. Accepts a string title or any caught error value
/// (`unknown`) — errors are auto-translated via the i18n catalog.
export const toast = {
  success: (title: unknown, description?: string, options?: ToastOptions) =>
    useToastStore
      .getState()
      .push(build("success", resolveTitle(title), description, options)),
  error: (title: unknown, description?: string, options?: ToastOptions) =>
    useToastStore
      .getState()
      .push(build("error", resolveTitle(title), description, options)),
  warn: (title: unknown, description?: string, options?: ToastOptions) =>
    useToastStore
      .getState()
      .push(build("warn", resolveTitle(title), description, options)),
  info: (title: unknown, description?: string, options?: ToastOptions) =>
    useToastStore
      .getState()
      .push(build("info", resolveTitle(title), description, options)),
  neutral: (title: unknown, description?: string, options?: ToastOptions) =>
    useToastStore
      .getState()
      .push(build("neutral", resolveTitle(title), description, options)),
};
