import { useTranslation } from "react-i18next";
import { Plus, X } from "lucide-react";

import { Button } from "../ui/Button";
import { Input } from "../ui/Input";
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
    if (next.length > 0 && !next.some((e) => e.is_default)) {
      next[0] = { ...next[0], is_default: true };
    }
    onChange(next);
  };

  const add = () =>
    onChange([
      ...value,
      { id: null, value: "", label: null, is_default: value.length === 0 },
    ]);

  const makeDefault = (idx: number) =>
    onChange(value.map((e, i) => ({ ...e, is_default: i === idx })));

  return (
    <section>
      <div className="text-[12px] font-medium text-ink-3 mb-1.5">{title}</div>
      <div className="border border-line rounded-card overflow-hidden">
        {value.length === 0 ? (
          <p className="px-3 py-2.5 text-[12px] text-ink-4">{emptyLabel}</p>
        ) : (
          value.map((entry, idx) => (
            <div
              key={idx}
              className="grid items-center gap-2.5 px-3 py-2 border-b border-line-soft last:border-b-0"
              style={{ gridTemplateColumns: "auto 110px 1fr 24px" }}
            >
              <label
                className="inline-flex items-center cursor-pointer"
                title={t("clients.default_contact") ?? ""}
              >
                <input
                  type="radio"
                  name={`default-${title}`}
                  checked={entry.is_default}
                  onChange={() => makeDefault(idx)}
                  className="accent-accent"
                />
              </label>
              <Input
                value={entry.label ?? ""}
                placeholder={t("clients.contact_label_placeholder") ?? ""}
                onChange={(e) => update(idx, { label: e.target.value || null })}
                className="!py-1 !text-[12px]"
              />
              <Input
                mono
                type={type}
                value={entry.value}
                placeholder={valuePlaceholder}
                onChange={(e) => update(idx, { value: e.target.value })}
                required
                className="!py-1 !text-[12px]"
              />
              <button
                type="button"
                onClick={() => remove(idx)}
                className="text-ink-3 hover:text-danger"
                aria-label={t("common.delete") ?? ""}
              >
                <X size={13} strokeWidth={1.5} />
              </button>
            </div>
          ))
        )}
        <Button
          size="sm"
          variant="ghost"
          type="button"
          onClick={add}
          className="m-1.5"
          leadingIcon={<Plus size={11} strokeWidth={1.5} />}
        >
          {addLabel}
        </Button>
      </div>
    </section>
  );
}
