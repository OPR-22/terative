import { useTranslation } from "react-i18next";

import type { ClientAddressDto } from "../../ipc";

interface Props {
  addresses: ClientAddressDto[];
}

/**
 * Resolves the active billing/shipping address. By construction (DB
 * partial unique indexes + domain validation) at most one row can have
 * each flag set, so a simple `find` suffices.
 */
function resolveBilling(addresses: ClientAddressDto[]): ClientAddressDto | null {
  return addresses.find((a) => a.is_billing) ?? null;
}

function resolveShipping(addresses: ClientAddressDto[]): ClientAddressDto | null {
  return addresses.find((a) => a.is_shipping) ?? null;
}

function formatLines(a: ClientAddressDto): string[] {
  const lines: string[] = [a.street];
  if (a.apt_suite && a.apt_suite.trim() !== "") {
    lines.push(a.apt_suite.trim());
  }
  let cityLine = `${a.postal_code} ${a.city}`;
  if (a.state_province && a.state_province.trim() !== "") {
    cityLine += `, ${a.state_province.trim()}`;
  }
  lines.push(cityLine);
  lines.push(a.country);
  return lines;
}

export function AddressSummary({ addresses }: Props) {
  const { t } = useTranslation();
  const billing = resolveBilling(addresses);
  const shipping = resolveShipping(addresses);

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
      <Slot title={t("clients.billing_address")} addr={billing} />
      <Slot
        title={t("clients.shipping_address")}
        addr={shipping}
        sameAsBilling={
          billing != null && shipping != null && billing.id === shipping.id
        }
      />
    </div>
  );
}

function Slot({
  title,
  addr,
  sameAsBilling,
}: {
  title: string;
  addr: ClientAddressDto | null;
  sameAsBilling?: boolean;
}) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-1">
      <div className="text-[11px] font-medium text-ink-3 uppercase tracking-wide">
        {title}
      </div>
      {addr == null ? (
        <p className="text-[13px] text-ink-4">
          {t("clients.no_default_address")}
        </p>
      ) : sameAsBilling ? (
        <p className="text-[13px] text-ink-3 italic">
          {t("clients.shipping_same_as_billing")}
        </p>
      ) : (
        <div className="text-[13px] text-ink leading-snug">
          {addr.label ? (
            <div className="text-[12px] text-ink-3 mb-0.5">{addr.label}</div>
          ) : null}
          {formatLines(addr).map((line, i) => (
            <div key={i}>{line}</div>
          ))}
        </div>
      )}
    </div>
  );
}
