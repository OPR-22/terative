import { useEffect, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { Image as ImageIcon, Upload, X } from "lucide-react";

import { Button } from "../ui/Button";

const DEFAULT_MAX_BYTES = 2 * 1024 * 1024;
const ACCEPT_TYPES = ["image/png", "image/jpeg", "image/jpg"];
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
      <span className="text-[12px] font-medium text-ink-3">{label}</span>
      <div className="flex items-center gap-3 border border-dashed border-line p-3.5">
        {previewUrl ? (
          <img
            src={previewUrl}
            alt={label}
            className="h-10 w-auto max-w-32 bg-paper-3 object-contain border border-line"
          />
        ) : (
          <div className="grid place-items-center w-[42px] h-[42px] bg-paper-3 text-ink-3">
            <ImageIcon size={18} strokeWidth={1.5} />
          </div>
        )}
        <p className="text-[11px] text-ink-3 flex-1">
          {t("image_uploader.hint", { max: `${maxMb} MB` })}
        </p>
        {previewUrl ? (
          <Button
            size="sm"
            variant="ghost"
            iconOnly
            onClick={() => {
              onChange(null);
              setError(null);
            }}
            aria-label={t("image_uploader.remove")}
          >
            <X size={12} strokeWidth={1.5} />
          </Button>
        ) : null}
        <Button
          size="sm"
          onClick={() => inputRef.current?.click()}
          leadingIcon={<Upload size={11} strokeWidth={1.5} />}
        >
          {previewUrl
            ? t("image_uploader.replace")
            : t("image_uploader.upload")}
        </Button>
      </div>
      <input
        ref={inputRef}
        type="file"
        accept={ACCEPT_ATTR}
        className="hidden"
        onChange={onFileChange}
      />
      {error ? <p className="text-[12px] text-danger">{error}</p> : null}
    </div>
  );
}
