import {
  useEffect,
  useId,
  useRef,
  useState,
  type ReactNode,
} from "react";

import { Button } from "./Button";

export interface DropdownMenuItem {
  /** Stable id used as React key. */
  id: string;
  label: ReactNode;
  /** Optional leading icon, sized to match the button text. */
  icon?: ReactNode;
  /** Visual emphasis. `danger` is reserved for destructive actions. */
  tone?: "default" | "danger";
  disabled?: boolean;
  onSelect: () => void;
}

interface DropdownMenuProps {
  /**
   * Either a custom trigger node, or `undefined` to use the default `⋯`
   * icon-only button. Custom triggers must accept the standard button
   * props (we wrap them with `cloneElement` to attach the toggle handler).
   */
  trigger?: ReactNode;
  items: DropdownMenuItem[];
  /** Accessible label for the default trigger button. */
  triggerLabel?: string;
  /** Visual alignment of the menu relative to the trigger. */
  align?: "left" | "right";
  className?: string;
}

/**
 * Lightweight dropdown menu — used for the row-level "more actions"
 * overflow on tables. Closes on Escape, click-outside, and item select.
 *
 * Stops propagation on the trigger and menu surface so the parent row
 * (which navigates to the editor) doesn't fire when the user clicks
 * here. Callers don't need to add their own `e.stopPropagation()`.
 */
export function DropdownMenu({
  trigger,
  items,
  triggerLabel,
  align = "right",
  className = "",
}: DropdownMenuProps) {
  const [open, setOpen] = useState(false);
  const menuId = useId();
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!open) return;
    const onDocClick = (e: MouseEvent) => {
      if (!containerRef.current) return;
      if (containerRef.current.contains(e.target as Node)) return;
      setOpen(false);
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") setOpen(false);
    };
    document.addEventListener("mousedown", onDocClick);
    document.addEventListener("keydown", onKey);
    return () => {
      document.removeEventListener("mousedown", onDocClick);
      document.removeEventListener("keydown", onKey);
    };
  }, [open]);

  return (
    <div
      ref={containerRef}
      className={["relative inline-block", className].join(" ")}
      onClick={(e) => e.stopPropagation()}
    >
      {trigger ? (
        <span onClick={() => setOpen((o) => !o)}>{trigger}</span>
      ) : (
        <Button
          size="sm"
          iconOnly
          aria-haspopup="menu"
          aria-expanded={open}
          aria-controls={menuId}
          aria-label={triggerLabel}
          onClick={() => setOpen((o) => !o)}
        >
          <Dots />
        </Button>
      )}
      {open ? (
        <div
          id={menuId}
          role="menu"
          className={[
            "absolute z-30 mt-1 min-w-[180px] bg-paper border border-line rounded-card shadow-card py-1",
            align === "right" ? "right-0" : "left-0",
          ].join(" ")}
        >
          {items.map((item) => (
            <button
              key={item.id}
              type="button"
              role="menuitem"
              disabled={item.disabled}
              onClick={() => {
                if (item.disabled) return;
                setOpen(false);
                item.onSelect();
              }}
              className={[
                "w-full flex items-center gap-2 px-3 py-1.5 text-[13px] text-left transition-colors disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer",
                item.tone === "danger"
                  ? "text-danger hover:bg-danger-soft"
                  : "text-ink hover:bg-paper-2",
              ].join(" ")}
            >
              {item.icon ? (
                <span className="text-ink-3 shrink-0">{item.icon}</span>
              ) : null}
              <span className="flex-1 truncate">{item.label}</span>
            </button>
          ))}
        </div>
      ) : null}
    </div>
  );
}

function Dots() {
  return (
    <svg
      width="14"
      height="14"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.5"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <circle cx="12" cy="5" r="1" fill="currentColor" />
      <circle cx="12" cy="12" r="1" fill="currentColor" />
      <circle cx="12" cy="19" r="1" fill="currentColor" />
    </svg>
  );
}
