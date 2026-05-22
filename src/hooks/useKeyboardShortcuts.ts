import { useEffect } from "react";
import { useNavigate } from "react-router-dom";

/**
 * Global keyboard shortcuts. Registered once at the Shell level.
 *
 * Current bindings:
 *   ⌘N / ^N       → new invoice (navigates to /invoices)
 *   ⌘,            → settings
 *   ⌘/            → clients
 *
 * ⌘K (global search) is handled separately in the Topbar so it stays
 * reachable while a form field is focused.
 *
 * Shortcuts are swallowed when the event target is an input/textarea so
 * typing in a form isn't hijacked.
 */
export function useKeyboardShortcuts() {
  const navigate = useNavigate();

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement | null;
      if (
        target &&
        (target.tagName === "INPUT" ||
          target.tagName === "TEXTAREA" ||
          target.isContentEditable ||
          target.tagName === "SELECT")
      ) {
        return;
      }
      const mod = e.metaKey || e.ctrlKey;
      if (!mod) return;

      switch (e.key.toLowerCase()) {
        case "n":
          e.preventDefault();
          navigate("/invoices");
          break;
        case ",":
          e.preventDefault();
          navigate("/settings");
          break;
        case "/":
          e.preventDefault();
          navigate("/clients");
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [navigate]);
}
