import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "./Button";

const DEFAULT_MAX_BYTES = 2 * 1024 * 1024;
// `image/jpeg` is the standard MIME for both .jpg and .jpeg files; a handful
// of systems emit `image/jpg` incorrectly, so we accept it too.
const ACCEPT_TYPES = ["image/png", "image/jpeg", "image/jpg"];
// Explicit extensions in `accept` as a fallback for file pickers that don't
// pre-filter by MIME.
const ACCEPT_ATTR = "image/png,image/jpeg,.png,.jpg,.jpeg";

interface ImageUploaderProps {
  label: string;
  value: number[] | null;
  onChange: (bytes: number[] | null) => void;
  maxBytes?: number;
}

export function ImageUploader({
  label,
  value,
  onChange,
  maxBytes = DEFAULT_MAX_BYTES,
}: ImageUploaderProps) {
  const { t } = useTranslation();
  const [error, setError] = useState<string | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (!value || value.length === 0) {
      setPreviewUrl(null);
      return;
    }
    const blob = new Blob([new Uint8Array(value)]);
    const url = URL.createObjectURL(blob);
    setPreviewUrl(url);
    return () => URL.revokeObjectURL(url);
  }, [value]);

  const maxMb = Math.floor(maxBytes / (1024 * 1024));

  const onFileChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    setError(null);
    const file = e.target.files?.[0];
    if (!file) return;
    if (!ACCEPT_TYPES.includes(file.type)) {
      setError(t("image_uploader.err_type") ?? "Unsupported image type");
      e.target.value = "";
      return;
    }
    if (file.size > maxBytes) {
      setError(
        t("image_uploader.err_size", { max: `${maxMb} MB` }) ??
          `File too large (max ${maxMb} MB)`,
      );
      e.target.value = "";
      return;
    }
    const buf = await file.arrayBuffer();
    onChange(Array.from(new Uint8Array(buf)));
    e.target.value = "";
  };

  return (
    <div className="flex flex-col gap-2">
      <label className="text-sm font-medium text-fg-muted">{label}</label>
      {previewUrl ? (
        <div className="flex items-start gap-3">
          <img
            src={previewUrl}
            alt={label}
            className="h-24 w-auto max-w-48 rounded-field border border-border bg-surface object-contain"
          />
          <div className="flex flex-col gap-2">
            <Button
              type="button"
              variant="secondary"
              onClick={() => inputRef.current?.click()}
            >
              {t("image_uploader.replace")}
            </Button>
            <Button
              type="button"
              variant="danger"
              onClick={() => {
                onChange(null);
                setError(null);
              }}
            >
              {t("image_uploader.remove")}
            </Button>
          </div>
        </div>
      ) : (
        <Button
          type="button"
          variant="secondary"
          onClick={() => inputRef.current?.click()}
          className="self-start"
        >
          {t("image_uploader.upload")}
        </Button>
      )}
      <input
        ref={inputRef}
        type="file"
        accept={ACCEPT_ATTR}
        className="hidden"
        onChange={onFileChange}
      />
      {error ? <p className="text-sm text-danger">{error}</p> : null}
      <p className="text-xs text-fg-muted">
        {t("image_uploader.hint", { max: `${maxMb} MB` })}
      </p>
    </div>
  );
}
