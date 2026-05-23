/**
 * Circular avatar used to represent an organisation throughout the UI:
 * the picker grid, the create-modal preview, the sidebar switcher, and the
 * switcher's dropdown items.
 *
 * The background colour is derived deterministically from the code, so the
 * same org always looks the same and orgs are visually distinguishable at
 * a glance.
 */

interface Props {
  /** The org code — drives both the letter and the colour. */
  code: string;
  size?: "sm" | "lg" | "xl";
  className?: string;
}

const SIZES = {
  /** 26px — sidebar trigger + dropdown items. */
  sm: { box: "w-6.5 h-6.5", text: "text-[11px]" },
  /** 56px — create-modal preview. */
  lg: { box: "w-14 h-14", text: "text-xl" },
  /** 64px — picker grid tile. */
  xl: { box: "w-16 h-16", text: "text-2xl" },
} as const;

const PALETTE: ReadonlyArray<{ bg: string; fg: string }> = [
  { bg: "#FEF3C7", fg: "#92400E" }, // amber
  { bg: "#DBEAFE", fg: "#1E40AF" }, // blue
  { bg: "#D1FAE5", fg: "#065F46" }, // emerald
  { bg: "#FCE7F3", fg: "#9D174D" }, // pink
  { bg: "#E0E7FF", fg: "#3730A3" }, // indigo
  { bg: "#FFE4E6", fg: "#9F1239" }, // rose
  { bg: "#F3E8FF", fg: "#6B21A8" }, // purple
  { bg: "#CCFBF1", fg: "#115E59" }, // teal
];

function avatarLetterOf(code: string): string {
  for (const ch of code) {
    if (/[a-z0-9]/i.test(ch)) return ch.toUpperCase();
  }
  return "?";
}

/** djb2 string hash — fast, deterministic, good enough distribution. */
function hashCode(s: string): number {
  let h = 5381;
  for (let i = 0; i < s.length; i++) {
    h = ((h << 5) + h + s.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

export function OrgAvatar({ code, size = "sm", className = "" }: Props) {
  const { box, text } = SIZES[size];
  const swatch = PALETTE[hashCode(code) % PALETTE.length];
  return (
    <span
      className={[
        "grid place-items-center rounded-full font-semibold select-none shrink-0",
        box,
        text,
        className,
      ].join(" ")}
      style={{ backgroundColor: swatch.bg, color: swatch.fg }}
      aria-hidden
    >
      {avatarLetterOf(code)}
    </span>
  );
}
