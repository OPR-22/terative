import { useTranslation } from "react-i18next";
import { Plus, Trash2 } from "lucide-react";

import { Button } from "../ui/Button";
import { Field, Input } from "../ui/Input";
import type { ClientAddressDto } from "../../ipc";

interface Props {
  value: ClientAddressDto[];
  onChange: (addresses: ClientAddressDto[]) => void;
}

const blank: ClientAddressDto = {
  id: null,
  label: null,
  street: "",
  apt_suite: null,
  city: "",
  state_province: null,
  postal_code: "",
  country: "",
  is_billing: false,
  is_shipping: false,
};

/** Stable key for an address — by id when persisted, by index otherwise. */
function keyFor(a: ClientAddressDto, idx: number): string {
  return a.id ?? `new-${idx}`;
}

/**
 * Toggles `role` on the row at `idx`. When turning a role ON, the same
 * role is cleared on every other row — the DB enforces "at most one
 * billing / one shipping per client" via partial unique indexes, so the
 * client side mirrors that to keep the UI in sync before save.
 */
function toggleActive(
  list: ClientAddressDto[],
  idx: number,
  role: "billing" | "shipping",
): ClientAddressDto[] {
  const target = list[idx];
  const next = role === "billing" ? !target.is_billing : !target.is_shipping;
  return list.map((a, i) => {
    if (i === idx) {
      return role === "billing"
        ? { ...a, is_billing: next }
        : { ...a, is_shipping: next };
    }
    if (!next) return a;
    // Turning the role on — clear the same flag elsewhere.
    if (role === "billing" && a.is_billing) {
      return { ...a, is_billing: false };
    }
    if (role === "shipping" && a.is_shipping) {
      return { ...a, is_shipping: false };
    }
    return a;
  });
}

export function AddressListEditor({ value, onChange }: Props) {
  const { t } = useTranslation();

  const update = (idx: number, patch: Partial<ClientAddressDto>) => {
    onChange(value.map((a, i) => (i === idx ? { ...a, ...patch } : a)));
  };

  const toggle = (idx: number, role: "billing" | "shipping") => {
    onChange(toggleActive(value, idx, role));
  };

  const remove = (idx: number) => {
    onChange(value.filter((_, i) => i !== idx));
  };

  const add = () => onChange([...value, { ...blank }]);

  return (
    <div className="flex flex-col gap-3">
      {value.length === 0 ? (
        <p className="text-[12px] text-ink-4 px-1">
          {t("clients.no_addresses")}
        </p>
      ) : null}

      {value.map((addr, idx) => (
        <AddressCard
          key={keyFor(addr, idx)}
          addr={addr}
          onChange={(patch) => update(idx, patch)}
          onToggle={(role) => toggle(idx, role)}
          onRemove={() => remove(idx)}
        />
      ))}

      <div>
        <Button
          size="sm"
          variant="ghost"
          type="button"
          onClick={add}
          leadingIcon={<Plus size={11} strokeWidth={1.5} />}
        >
          {t("clients.add_address")}
        </Button>
      </div>
    </div>
  );
}

function AddressCard({
  addr,
  onChange,
  onToggle,
  onRemove,
}: {
  addr: ClientAddressDto;
  onChange: (patch: Partial<ClientAddressDto>) => void;
  onToggle: (role: "billing" | "shipping") => void;
  onRemove: () => void;
}) {
  const { t } = useTranslation();
  return (
    <div className="border border-line rounded-card p-3.5 bg-paper flex flex-col gap-2.5">
      <div className="flex items-center gap-2.5">
        <Input
          value={addr.label ?? ""}
          placeholder={t("clients.address_label_placeholder") ?? ""}
          onChange={(e) => onChange({ label: e.target.value || null })}
          className="!py-1 !text-[12px] flex-1"
        />
        <button
          type="button"
          onClick={onRemove}
          className="text-ink-3 hover:text-danger p-1 cursor-pointer"
          aria-label={t("common.delete") ?? ""}
        >
          <Trash2 size={14} strokeWidth={1.5} />
        </button>
      </div>

      <Field label={t("clients.address_street")}>
        <Input
          value={addr.street}
          onChange={(e) => onChange({ street: e.target.value })}
          required
        />
      </Field>
      <Field label={t("clients.address_apt_suite")}>
        <Input
          value={addr.apt_suite ?? ""}
          onChange={(e) => onChange({ apt_suite: e.target.value || null })}
        />
      </Field>
      <div className="grid grid-cols-1 sm:grid-cols-[1fr_140px_140px] gap-2.5">
        <Field label={t("clients.address_city")}>
          <Input
            value={addr.city}
            onChange={(e) => onChange({ city: e.target.value })}
            required
          />
        </Field>
        <Field label={t("clients.address_state_province")}>
          <Input
            value={addr.state_province ?? ""}
            onChange={(e) =>
              onChange({ state_province: e.target.value || null })
            }
          />
        </Field>
        <Field label={t("clients.address_postal_code")}>
          <Input
            value={addr.postal_code}
            onChange={(e) => onChange({ postal_code: e.target.value })}
            required
          />
        </Field>
      </div>
      <Field label={t("clients.address_country")}>
        <Input
          value={addr.country}
          onChange={(e) => onChange({ country: e.target.value })}
          placeholder="FR"
          required
        />
      </Field>

      <div className="flex flex-wrap items-center gap-2 pt-1">
        <RoleToggle
          active={addr.is_billing}
          label={t("clients.address_billing")}
          onClick={() => onToggle("billing")}
        />
        <RoleToggle
          active={addr.is_shipping}
          label={t("clients.address_shipping")}
          onClick={() => onToggle("shipping")}
        />
      </div>
    </div>
  );
}

function RoleToggle({
  active,
  label,
  onClick,
}: {
  active: boolean;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "px-2.5 py-1 rounded-full text-[12px] border transition-colors cursor-pointer",
        active
          ? "bg-ink text-paper border-ink"
          : "bg-paper text-ink-3 border-line hover:border-ink-3 hover:text-ink",
      ].join(" ")}
    >
      {label}
    </button>
  );
}
