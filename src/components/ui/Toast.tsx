import {
  useEffect,
  useRef,
  useState,
  type ComponentType,
} from "react";
import { useTranslation } from "react-i18next";
import {
  AlertCircle,
  AlertTriangle,
  Check,
  Info,
  Trash2,
  X,
} from "lucide-react";

import {
  MAX_VISIBLE_TOASTS,
  useToastStore,
  type Toast as ToastModel,
  type ToastKind,
} from "../../stores/toastStore";

interface IconProps {
  size?: number;
  strokeWidth?: number;
  className?: string;
}

/// Icon + filet color per kind. The filet (3 px left border) is the
/// only chrome that carries the semantic; the rest of the toast stays
/// neutral on `--paper`.
const VARIANTS: Record<
  ToastKind,
  { icon: ComponentType<IconProps>; filet: string; iconColor: string }
> = {
  success: { icon: Check,          filet: "border-l-ok",     iconColor: "text-ok" },
  error:   { icon: AlertCircle,    filet: "border-l-danger", iconColor: "text-danger" },
  warn:    { icon: AlertTriangle,  filet: "border-l-warn",   iconColor: "text-warn-ink" },
  info:    { icon: Info,           filet: "border-l-accent", iconColor: "text-accent" },
  neutral: { icon: Trash2,         filet: "border-l-ink-3",  iconColor: "text-ink-3" },
};

/// `aria-live` politeness per kind. Errors and warnings interrupt
/// (`assertive`); confirmations and info wait their turn (`polite`).
function ariaProps(kind: ToastKind): {
  role: "status" | "alert";
  "aria-live": "polite" | "assertive";
} {
  if (kind === "error" || kind === "warn") {
    return { role: "alert", "aria-live": "assertive" };
  }
  return { role: "status", "aria-live": "polite" };
}

/// Global toast container. Fixed bottom-right on desktop, full-width
/// strip at the bottom on mobile. Stacks vertically with the newest
/// toast on top (closest to the top of the visible stack — i.e.,
/// furthest from the screen edge anchor).
export function ToastContainer() {
  const toasts = useToastStore((s) => s.toasts);
  // Newest on top: render in reverse insertion order. Cap at the spec's
  // MAX_VISIBLE so the screen doesn't fill up; anything beyond stays in
  // the store and surfaces as the head pops.
  const visible = toasts.slice(-MAX_VISIBLE_TOASTS).reverse();
  return (
    <div
      className="fixed z-50 flex flex-col gap-2.5 pointer-events-none bottom-4 left-4 right-4 md:left-auto md:bottom-6 md:right-6 md:w-[380px]"
    >
      {visible.map((toast) => (
        <ToastItem key={toast.id} toast={toast} />
      ))}
    </div>
  );
}

function ToastItem({ toast }: { toast: ToastModel }) {
  const { t } = useTranslation();
  const dismissFromStore = useToastStore((s) => s.dismiss);
  const [paused, setPaused] = useState(false);
  const [leaving, setLeaving] = useState(false);
  // Track elapsed-while-running so a hover/focus pause resumes from
  // wherever it left off rather than restarting the clock.
  const startedAtRef = useRef<number>(Date.now());
  const elapsedRef = useRef<number>(0);
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const close = () => {
    if (leaving) return;
    setLeaving(true);
    // Match the exit animation duration so the DOM unmount happens after
    // the visual fade — otherwise the toast would just snap away.
    setTimeout(() => dismissFromStore(toast.id), 180);
  };

  // Auto-dismiss timer. Persistent toasts skip this entirely. Pausing
  // (hover/focus) clears the running timer and records elapsed time;
  // unpausing schedules a fresh timer for the remaining duration.
  useEffect(() => {
    if (toast.persistent || leaving) return;
    if (paused) {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
        elapsedRef.current += Date.now() - startedAtRef.current;
      }
      return;
    }
    startedAtRef.current = Date.now();
    const remaining = Math.max(0, toast.duration - elapsedRef.current);
    timerRef.current = setTimeout(close, remaining);
    return () => {
      if (timerRef.current !== null) {
        clearTimeout(timerRef.current);
        timerRef.current = null;
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [paused, leaving, toast.persistent]);

  const onKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      close();
    }
  };

  const variant = VARIANTS[toast.kind];
  const Icon = variant.icon;

  // Progress bar color is the same semantic. Bar's CSS animation drives
  // the visual countdown; we pause it via inline `animationPlayState` so
  // the visible countdown stays in sync with the JS timer pause.
  const barColor: Record<ToastKind, string> = {
    success: "bg-ok",
    error: "bg-danger",
    warn: "bg-warn",
    info: "bg-accent",
    neutral: "bg-ink-3",
  };

  return (
    <div
      {...ariaProps(toast.kind)}
      onKeyDown={onKeyDown}
      onMouseEnter={() => setPaused(true)}
      onMouseLeave={() => setPaused(false)}
      onFocus={() => setPaused(true)}
      onBlur={() => setPaused(false)}
      tabIndex={0}
      style={{
        transition: "opacity 200ms ease-out, transform 200ms ease-out",
        opacity: leaving ? 0 : 1,
        transform: leaving ? "translateY(8px)" : "translateY(0)",
      }}
      className={[
        "pointer-events-auto relative bg-paper border border-line border-l-[3px] rounded-[3px] overflow-hidden",
        variant.filet,
        // Match the spec's two-layer shadow: 1px hairline + 24px soft.
        "shadow-[0_1px_2px_oklch(0_0_0/0.04),0_8px_24px_oklch(0_0_0/0.08)]",
      ].join(" ")}
    >
      <div
        className="grid items-start gap-3 px-3.5 py-3"
        style={{ gridTemplateColumns: "auto 1fr auto" }}
      >
        <span
          className={[
            "inline-grid place-items-center w-5 h-5 mt-0.5 shrink-0",
            variant.iconColor,
          ].join(" ")}
        >
          <Icon size={18} strokeWidth={1.8} />
        </span>
        <div className="min-w-0">
          <p className="m-0 text-[13.5px] font-bold leading-[1.35] tracking-[-0.005em] text-ink">
            {toast.title}
          </p>
          {toast.description ? (
            <p className="m-0 mt-0.5 text-[13px] text-ink-3 leading-[1.45] break-words">
              {toast.description}
            </p>
          ) : null}
        </div>
        {toast.action ? (
          <button
            type="button"
            onClick={() => {
              toast.action?.onClick();
              close();
            }}
            className="shrink-0 -mt-px self-start text-[12.5px] font-semibold text-accent-ink underline underline-offset-[3px] hover:opacity-80 transition-opacity cursor-pointer bg-transparent border-0 p-0"
          >
            {toast.action.label}
          </button>
        ) : (
          <button
            type="button"
            onClick={close}
            aria-label={t("common.close")}
            className="shrink-0 -mt-1 -mr-1 grid place-items-center w-6 h-6 rounded-sm text-ink-3 hover:bg-paper-3 hover:text-ink cursor-pointer bg-transparent border-0 transition-colors"
          >
            <X size={14} strokeWidth={1.8} />
          </button>
        )}
      </div>
      {!toast.persistent ? (
        <div className="h-0.5 bg-line-soft relative">
          <div
            className={[
              "absolute inset-y-0 left-0 opacity-50 toast-progress",
              barColor[toast.kind],
            ].join(" ")}
            style={{
              animationDuration: `${toast.duration}ms`,
              animationPlayState: paused || leaving ? "paused" : "running",
            }}
          />
        </div>
      ) : null}
    </div>
  );
}
