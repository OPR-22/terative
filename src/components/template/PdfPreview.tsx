import {
  forwardRef,
  useEffect,
  useImperativeHandle,
  useMemo,
  useRef,
  useState,
} from "react";
import { useTranslation } from "react-i18next";

interface Props {
  bytes: Uint8Array | null;
  loading?: boolean;
  error?: string | null;
}

/// Imperative handle exposed to parents that want to drive the embedded
/// viewer (e.g. a "Print" button in the surrounding chrome). Callers
/// access it via `ref` — see `InvoiceEditor` for an example.
export interface PdfPreviewHandle {
  /// Trigger the browser's print dialog targeting the iframe content.
  /// No-op if the PDF hasn't loaded yet.
  print(): void;
}

export const PdfPreview = forwardRef<PdfPreviewHandle, Props>(function PdfPreview(
  { bytes, loading, error },
  ref,
) {
  const { t } = useTranslation();
  const previousUrlRef = useRef<string | null>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    if (previousUrlRef.current) {
      URL.revokeObjectURL(previousUrlRef.current);
      previousUrlRef.current = null;
    }
    if (!bytes) {
      setUrl(null);
      return;
    }
    const blob = new Blob([new Uint8Array(bytes)], { type: "application/pdf" });
    const objectUrl = URL.createObjectURL(blob);
    previousUrlRef.current = objectUrl;
    setUrl(objectUrl);
    return () => {
      URL.revokeObjectURL(objectUrl);
    };
  }, [bytes]);

  useImperativeHandle(ref, () => ({
    print() {
      // Same-origin blob URL means the iframe's contentWindow is reachable.
      // If the PDF hasn't loaded yet (or the user clicked print before the
      // first byte arrived), silently no-op rather than throwing.
      iframeRef.current?.contentWindow?.focus();
      iframeRef.current?.contentWindow?.print();
    },
  }));

  return (
    <div className="relative h-full w-full overflow-hidden rounded-card border border-border bg-surface-muted">
      {url ? (
        <iframe
          ref={iframeRef}
          title="pdf-preview"
          src={url}
          className="h-full w-full border-0"
        />
      ) : (
        <div className="flex h-full items-center justify-center text-sm text-fg-muted">
          {loading ? t("common.loading") : t("preview.empty")}
        </div>
      )}
      {loading && url ? (
        <div className="absolute right-2 top-2 rounded-pill bg-surface px-2 py-1 text-xs text-fg-muted shadow-card">
          {t("common.loading")}
        </div>
      ) : null}
      {error ? (
        <div className="absolute inset-x-2 bottom-2 rounded-field bg-status-cancelled-bg px-3 py-2 text-xs text-status-cancelled-fg">
          {error}
        </div>
      ) : null}
    </div>
  );
});

export function useDebounced<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);
  return useMemo(() => debounced, [debounced]);
}
