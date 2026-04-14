import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { Onboarding } from "../onboarding/Onboarding";
import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import { useTheme } from "../../hooks/useTheme";

export function Shell() {
  useTheme();
  useKeyboardShortcuts();
  return (
    <div className="flex h-full bg-surface text-fg">
      <Sidebar />
      <main className="flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
      <Onboarding />
    </div>
  );
}
