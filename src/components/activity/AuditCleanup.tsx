import { useState } from "react";
import { useTranslation } from "react-i18next";

import { ipc } from "../../ipc";
import { toast } from "../../stores/toastStore";
import { Button } from "../ui/Button";
import { ConfirmModal } from "../ui/ConfirmModal";

const YEAR_OPTIONS = [1, 2, 3, 4, 5] as const;
type Years = (typeof YEAR_OPTIONS)[number];

/**
 * Maintenance affordance: delete audit rows older than N years (1..=5).
 * Two-step UX — pick the window, then confirm in a modal before the
 * destructive call lands. Destructive ops use the `danger` tone +
 * `requireText` to make accidental clicks harder.
 */
export function AuditCleanup({ onCleaned }: { onCleaned?: () => void }) {
  const { t } = useTranslation();
  const [years, setYears] = useState<Years>(2);
  const [confirmOpen, setConfirmOpen] = useState(false);

  async function runCleanup() {
    // Compute the cutoff client-side from today minus N years. Date arithmetic
    // in JS respects month/day so leap years are handled naturally.
    const cutoff = new Date();
    cutoff.setFullYear(cutoff.getFullYear() - years);
    try {
      const removed = await ipc.auditCleanupOlderThan(cutoff.toISOString());
      toast.success(t("audit.cleanup.success", { count: removed }));
      onCleaned?.();
    } catch (e) {
      toast.error(e);
    }
  }

  return (
    <div className="flex items-center gap-3 text-[12.5px]">
      <span className="text-ink-2">{t("audit.cleanup.prompt")}</span>
      <select
        className="rounded-field border border-border bg-surface px-2 py-1 text-sm text-fg shadow-sm"
        value={years}
        onChange={(e) => setYears(Number(e.target.value) as Years)}
      >
        {YEAR_OPTIONS.map((n) => (
          <option key={n} value={n}>
            {t("audit.cleanup.years", { count: n })}
          </option>
        ))}
      </select>
      <Button variant="danger" size="sm" onClick={() => setConfirmOpen(true)}>
        {t("audit.cleanup.action")}
      </Button>

      <ConfirmModal
        open={confirmOpen}
        title={t("audit.cleanup.confirm_title")}
        description={t("audit.cleanup.confirm_body", { count: years })}
        confirmLabel={t("audit.cleanup.action")}
        tone="danger"
        requireText={t("settings.confirm_delete_phrase")}
        onConfirm={runCleanup}
        onClose={() => setConfirmOpen(false)}
      />
    </div>
  );
}
