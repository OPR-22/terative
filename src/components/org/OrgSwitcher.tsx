import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { ChevronsUpDown, ListChecks, Lock } from "lucide-react";

import { DropdownMenu, type DropdownMenuItem } from "../ui/DropdownMenu";
import { errorCodeOf, translateError } from "../../ipc/errorCatalog";
import { toast } from "../../stores/toastStore";
import { useOrgStore } from "../../stores/orgStore";
import { OrgAvatar } from "./OrgAvatar";
import { OrgUnlockModal } from "./OrgUnlockModal";

interface Props {
  collapsed: boolean;
}

/**
 * Bottom-of-sidebar org context + switcher. Shows the active org's code
 * with its deterministic-colour avatar and opens a menu listing other
 * orgs (each with their own avatar) plus a separated "Manage orgs" entry.
 */
export function OrgSwitcher({ collapsed }: Props) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { activeOrg, orgs, refresh, open } = useOrgStore();
  const [unlockCode, setUnlockCode] = useState<string | null>(null);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  if (!activeOrg) return null;

  const others = orgs.filter((o) => o.code !== activeOrg.code);

  const items: DropdownMenuItem[] = [
    ...others.map<DropdownMenuItem>((o) => ({
      id: o.code,
      label: o.code,
      icon: (
        <span className="flex items-center gap-1.5">
          <OrgAvatar code={o.code} size="sm" />
          {o.has_password ? <Lock size={10} className="text-ink-3" /> : null}
        </span>
      ),
      onSelect: async () => {
        try {
          await open(o.code);
          // Shell remounts via key={activeOrg.code}; route stays the same.
        } catch (e) {
          const ec = errorCodeOf(e);
          if (ec === "org_password_required" || ec === "org_wrong_password") {
            setUnlockCode(o.code);
          } else {
            toast.error(translateError(e, t));
          }
        }
      },
    })),
    {
      id: "__manage__",
      label: t("org_switcher.manage", { defaultValue: "Manage orgs…" }),
      icon: <ListChecks size={14} />,
      separated: others.length > 0,
      onSelect: () => navigate("/picker"),
    },
  ];

  async function handleUnlock(password: string, remember: boolean) {
    if (!unlockCode) return;
    await open(unlockCode, password, remember);
    setUnlockCode(null);
  }

  return (
    <>
    <OrgUnlockModal
      code={unlockCode}
      onClose={() => setUnlockCode(null)}
      onSubmit={handleUnlock}
    />
    <DropdownMenu
      align="left"
      placement="up"
      className="block! w-full"
      items={items}
      trigger={
        <button
          type="button"
          aria-label={t("org_switcher.aria_open", {
            defaultValue: "Switch organisation",
          })}
          className={[
            "w-full text-left transition cursor-pointer",
            collapsed
              ? "p-2 grid place-items-center"
              : "px-3 py-3 flex items-center gap-2.5",
            "hover:bg-paper-3",
          ].join(" ")}
        >
          <OrgAvatar code={activeOrg.code} size="sm" />
          {collapsed ? null : (
            <>
              <span className="leading-tight min-w-0 flex-1">
                <span className="block text-ink font-medium text-[12px] truncate">
                  {activeOrg.code}
                </span>
                <span className="block text-[11px] text-ink-3">
                  {t("org_switcher.label", { defaultValue: "organisation" })}
                </span>
              </span>
              <ChevronsUpDown size={14} className="text-ink-3 shrink-0" />
            </>
          )}
        </button>
      }
    />
    </>
  );
}
