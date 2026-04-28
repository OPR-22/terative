import { create } from "zustand";

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

/// Imperative helper. Single-arg form (`toast.error("...")`) treats the
/// argument as the title and works seamlessly for migrating existing
/// `toast.error(String(e))` call sites.
export const toast = {
  success: (title: string, description?: string, options?: ToastOptions) =>
    useToastStore.getState().push(build("success", title, description, options)),
  error: (title: string, description?: string, options?: ToastOptions) =>
    useToastStore.getState().push(build("error", title, description, options)),
  warn: (title: string, description?: string, options?: ToastOptions) =>
    useToastStore.getState().push(build("warn", title, description, options)),
  info: (title: string, description?: string, options?: ToastOptions) =>
    useToastStore.getState().push(build("info", title, description, options)),
  neutral: (title: string, description?: string, options?: ToastOptions) =>
    useToastStore.getState().push(build("neutral", title, description, options)),
};
