import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { BOOKMARKS_BY_ID } from "../bookmarks";
import { Button } from "../components/common/Button";
import { ipc } from "../ipc";

/// Renders the back/forward/reload/home toolbar inside its OWN dedicated
/// webview (label "bookmark-toolbar"). Sits in the GTK layout above the
/// bookmark webview, sharing horizontal space with it. Reads the active
/// bookmark id from the URL: `/bookmark-toolbar/:id`.
export function BookmarkToolbar() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const bookmark = id ? BOOKMARKS_BY_ID[id] : undefined;

  if (!bookmark) {
    return (
      <div className="flex h-full items-center bg-surface px-3 text-sm text-fg-muted">
        {t("bookmarks.unknown")}
      </div>
    );
  }

  return (
    <div className="flex h-full items-center gap-1 border-b border-border bg-surface px-3">
      <Button
        variant="secondary"
        title={t("bookmarks.back") ?? ""}
        aria-label={t("bookmarks.back") ?? ""}
        onClick={() => void ipc.bookmarkBack(bookmark.id)}
      >
        ←
      </Button>
      <Button
        variant="secondary"
        title={t("bookmarks.forward") ?? ""}
        aria-label={t("bookmarks.forward") ?? ""}
        onClick={() => void ipc.bookmarkForward(bookmark.id)}
      >
        →
      </Button>
      <Button
        variant="secondary"
        title={t("bookmarks.reload") ?? ""}
        aria-label={t("bookmarks.reload") ?? ""}
        onClick={() => void ipc.bookmarkReload(bookmark.id)}
      >
        ↻
      </Button>
      <Button
        variant="secondary"
        title={t("bookmarks.home") ?? ""}
        aria-label={t("bookmarks.home") ?? ""}
        onClick={() => void ipc.bookmarkNavigate(bookmark.id, bookmark.url)}
      >
        ⌂
      </Button>
      <span className="ml-3 truncate text-sm text-fg-muted" title={bookmark.url}>
        {new URL(bookmark.url).hostname}
      </span>
    </div>
  );
}
