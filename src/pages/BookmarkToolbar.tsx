import { ArrowLeft, ArrowRight, Home, RotateCw, type LucideIcon } from "lucide-react";
import { useEffect, type MouseEventHandler } from "react";
import { useParams } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { ipc } from "../ipc";
import { useBookmarkStore } from "../stores/bookmarkStore";

interface ToolbarIconButtonProps {
  icon: LucideIcon;
  label: string;
  onClick: MouseEventHandler<HTMLButtonElement>;
}

function ToolbarIconButton({ icon: Icon, label, onClick }: ToolbarIconButtonProps) {
  return (
    <button
      type="button"
      title={label}
      aria-label={label}
      onClick={onClick}
      className="rounded-field p-2 text-fg-muted transition-colors hover:bg-surface-muted hover:text-fg"
    >
      <Icon className="h-4 w-4" />
    </button>
  );
}

/// Renders the back/forward/reload/home toolbar inside its OWN dedicated
/// webview (label "bookmark-toolbar"). Sits in the GTK layout above the
/// bookmark webview, sharing horizontal space with it. Reads the active
/// bookmark id from the URL: `/bookmark-toolbar/:id`.
export function BookmarkToolbar() {
  const { t } = useTranslation();
  const { id } = useParams<{ id: string }>();
  const loaded = useBookmarkStore((s) => s.loaded);
  const ensureLoaded = useBookmarkStore((s) => s.ensureLoaded);
  const bookmark = useBookmarkStore((s) =>
    id ? s.byId(id) : undefined,
  );

  useEffect(() => {
    void ensureLoaded();
  }, [ensureLoaded]);

  if (!loaded) {
    return null;
  }
  if (!bookmark) {
    return (
      <div className="flex h-full items-center bg-surface px-3 text-sm text-fg-muted">
        {t("bookmarks.unknown")}
      </div>
    );
  }

  return (
    <div className="flex h-full items-center gap-0.5 border-b border-border bg-surface px-2">
      <ToolbarIconButton
        icon={ArrowLeft}
        label={t("bookmarks.back") ?? ""}
        onClick={() => void ipc.bookmarkNavBack(bookmark.id)}
      />
      <ToolbarIconButton
        icon={ArrowRight}
        label={t("bookmarks.forward") ?? ""}
        onClick={() => void ipc.bookmarkNavForward(bookmark.id)}
      />
      <ToolbarIconButton
        icon={RotateCw}
        label={t("bookmarks.reload") ?? ""}
        onClick={() => void ipc.bookmarkNavReload(bookmark.id)}
      />
      <ToolbarIconButton
        icon={Home}
        label={t("bookmarks.home") ?? ""}
        onClick={() => void ipc.bookmarkNavTo(bookmark.id, bookmark.url)}
      />
      <span className="ml-2 truncate text-sm text-fg-muted" title={bookmark.url}>
        {new URL(bookmark.url).hostname}
      </span>
    </div>
  );
}
