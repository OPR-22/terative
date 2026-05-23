interface AvatarProps {
  name: string;
  size?: number;
  className?: string;
}

export function initialsOf(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  const first = parts[0][0] ?? "";
  const last = parts.length > 1 ? parts[parts.length - 1][0] : "";
  return (first + last).toUpperCase();
}

export function Avatar({ name, size = 26, className = "" }: AvatarProps) {
  return (
    <span
      className={[
        "inline-grid place-items-center rounded-full bg-accent-soft text-accent-ink font-medium select-none",
        className,
      ].join(" ")}
      style={{
        width: size,
        height: size,
        flex: `0 0 ${size}px`,
        fontSize: Math.round(size * 0.42),
      }}
    >
      {initialsOf(name)}
    </span>
  );
}
