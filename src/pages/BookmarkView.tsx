import { useEffect } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { ipc } from "../ipc";

// Hardcoded bookmarks for MVP. Swap for a DB-backed store when the feature
// graduates from "does it work?" to "ship it."
const BOOKMARKS: Record<string, { label: string; url: string }> = {
  example: { label: "Google", url: "https://google.com" },
};

// Must match `SIDEBAR_WIDTH` in src-tauri/src/commands/bookmark_commands.rs
// (the Tailwind `w-56` class on the sidebar = 14rem = 224px).
const SIDEBAR_WIDTH = 224;

function computeBounds() {
  return {
    x: SIDEBAR_WIDTH,
    y: 0,
    width: Math.max(1, window.innerWidth - SIDEBAR_WIDTH),
    height: Math.max(1, window.innerHeight),
  };
}

export function BookmarkView() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();

  const bookmark = id ? BOOKMARKS[id] : undefined;

  useEffect(() => {
    if (!bookmark) return;

    const open = async () => {
      const b = computeBounds();
      try {
        await ipc.bookmarkOpen(
          bookmark.url,
          b.x,
          b.y,
          b.width,
          b.height,
          window.devicePixelRatio,
        );
      } catch (e) {
        console.error("[bookmark] open failed", e);
      }
    };

    const sync = async () => {
      const b = computeBounds();
      try {
        await ipc.bookmarkSetBounds(b.x, b.y, b.width, b.height);
      } catch (e) {
        console.error("[bookmark] resize failed", e);
      }
    };

    void open();
    const onResize = () => void sync();
    window.addEventListener("resize", onResize);

    return () => {
      window.removeEventListener("resize", onResize);
      // Hide (not close) so a later bookmark open reuses the webview.
      void ipc.bookmarkHide();
    };
  }, [bookmark]);

  if (!bookmark) {
    return <p className="text-sm text-danger">{t("bookmarks.unknown")}</p>;
  }

  // React renders nothing — the native webview occupies the content area.
  return <></>;
}
