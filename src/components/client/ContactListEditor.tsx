import { useTranslation } from "react-i18next";

import { Button } from "../common/Button";
import { Input } from "../common/Input";
import type { ContactEntryDto } from "../../ipc";

interface Props {
  title: string;
  value: ContactEntryDto[];
  onChange: (entries: ContactEntryDto[]) => void;
  type?: "email" | "tel" | "text";
  valuePlaceholder?: string;
  addLabel: string;
  emptyLabel: string;
}

export function ContactListEditor({
  title,
  value,
  onChange,
  type = "text",
  valuePlaceholder,
  addLabel,
  emptyLabel,
}: Props) {
  const { t } = useTranslation();

  const update = (idx: number, patch: Partial<ContactEntryDto>) =>
    onChange(value.map((e, i) => (i === idx ? { ...e, ...patch } : e)));

  const remove = (idx: number) => {
    const next = value.filter((_, i) => i !== idx);
    // If we removed the default, promote the first remaining entry.
    if (next.length > 0 && !next.some((e) => e.is_default)) {
      next[0] = { ...next[0], is_default: true };
    }
    onChange(next);
  };

  const add = () =>
    onChange([
      ...value,
      {
        id: null,
        value: "",
        label: null,
        is_default: value.length === 0,
      },
    ]);

  const makeDefault = (idx: number) =>
    onChange(value.map((e, i) => ({ ...e, is_default: i === idx })));

  return (
    <section>
      <h3 className="mb-2 text-sm font-semibold text-fg-muted">{title}</h3>
      {value.length === 0 ? (
        <p className="mb-2 text-xs text-fg-subtle">{emptyLabel}</p>
      ) : (
        <div className="mb-2 flex flex-col gap-2">
          {value.map((entry, idx) => (
            <div
              key={idx}
              className="flex items-center gap-2 rounded-field border border-border p-2"
            >
              <label
                className="flex cursor-pointer items-center gap-1 text-xs text-fg-muted"
                title={t("clients.default_contact") ?? ""}
              >
                <input
                  type="radio"
                  name={`default-${title}`}
                  checked={entry.is_default}
                  onChange={() => makeDefault(idx)}
                />
                {t("clients.default_short")}
              </label>
              <Input
                value={entry.label ?? ""}
                placeholder={t("clients.contact_label_placeholder") ?? ""}
                onChange={(e) => update(idx, { label: e.target.value || null })}
                className="w-28"
              />
              <Input
                type={type}
                value={entry.value}
                placeholder={valuePlaceholder}
                onChange={(e) => update(idx, { value: e.target.value })}
                className="flex-1"
                required
              />
              <button
                type="button"
                onClick={() => remove(idx)}
                className="px-2 text-fg-subtle hover:text-danger"
                aria-label={t("common.delete") ?? ""}
              >
                ×
              </button>
            </div>
          ))}
        </div>
      )}
      <Button variant="secondary" type="button" onClick={add}>
        {addLabel}
      </Button>
    </section>
  );
}
