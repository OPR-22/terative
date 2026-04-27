import { useEffect } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { BOOKMARKS_BY_ID } from "../bookmarks";
import { ipc } from "../ipc";
import { currentSidebarWidth } from "../stores/sidebarStore";

function computeBounds() {
  const sidebar = currentSidebarWidth();
  return {
    x: sidebar,
    y: 0,
    width: Math.max(1, window.innerWidth - sidebar),
    height: Math.max(1, window.innerHeight),
  };
}

export function BookmarkView() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();

  const bookmark = id ? BOOKMARKS_BY_ID[id] : undefined;

  useEffect(() => {
    if (!bookmark) return;

    const open = async () => {
      const b = computeBounds();
      try {
        await ipc.bookmarkOpen(
          bookmark.id,
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

    void open();

    return () => {
      void ipc.bookmarkHide();
    };
  }, [bookmark]);

  if (!bookmark) {
    return <p className="text-sm text-danger">{t("bookmarks.unknown")}</p>;
  }

  // React renders nothing — the native bookmark webview occupies the right
  // side of the window (everything past the sidebar) on Linux. macOS/Windows
  // overlay the webview at the rect computed above.
  return <></>;
}
