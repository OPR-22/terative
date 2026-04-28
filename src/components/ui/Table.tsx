import type { HTMLAttributes, TdHTMLAttributes, ThHTMLAttributes } from "react";

export function Table({ className = "", children, ...rest }: HTMLAttributes<HTMLTableElement>) {
  return (
    <table className={["w-full border-collapse", className].join(" ")} {...rest}>
      {children}
    </table>
  );
}

export function THead({ children }: { children: React.ReactNode }) {
  return <thead>{children}</thead>;
}

interface ThProps extends ThHTMLAttributes<HTMLTableCellElement> {
  numeric?: boolean;
}

export function Th({ numeric, className = "", children, ...rest }: ThProps) {
  return (
    <th
      className={[
        "text-left font-medium text-[12px] text-ink-3 px-3.5 py-2.5 bg-paper-2 border-b border-line",
        numeric ? "text-right tabular font-mono" : "",
        className,
      ].join(" ")}
      {...rest}
    >
      {children}
    </th>
  );
}

interface TdProps extends TdHTMLAttributes<HTMLTableCellElement> {
  numeric?: boolean;
  muted?: boolean;
  mono?: boolean;
}

export function Td({
  numeric,
  muted,
  mono,
  className = "",
  children,
  ...rest
}: TdProps) {
  return (
    <td
      className={[
        "px-3.5 py-3 text-[13px] border-b border-line-soft align-middle",
        numeric ? "text-right tabular font-mono" : "",
        muted ? "text-ink-3" : "text-ink",
        mono && !numeric ? "font-mono tabular" : "",
        className,
      ].join(" ")}
      {...rest}
    >
      {children}
    </td>
  );
}

export function Tr({ className = "", children, ...rest }: HTMLAttributes<HTMLTableRowElement>) {
  return (
    <tr
      className={[
        "transition-colors hover:bg-paper-2 last:[&>td]:border-b-0",
        className,
      ].join(" ")}
      {...rest}
    >
      {children}
    </tr>
  );
}
