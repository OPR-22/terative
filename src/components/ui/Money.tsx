import { useMoneyFormat } from "../../lib/money";
import type { MoneyDto } from "../../ipc";

interface MoneyDisplayProps {
  amount: MoneyDto;
  large?: boolean;
  muted?: boolean;
  className?: string;
}

export function MoneyDisplay({ amount, large, muted, className = "" }: MoneyDisplayProps) {
  const { format } = useMoneyFormat();
  return (
    <span
      className={[
        "font-mono tabular whitespace-nowrap",
        large ? "text-[16px] font-medium" : "",
        muted ? "text-ink-3" : "",
        className,
      ].join(" ")}
    >
      {format(amount)}
    </span>
  );
}
