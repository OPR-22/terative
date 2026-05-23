import { Outlet } from "react-router-dom";

import { useKeyboardShortcuts } from "../../hooks/useKeyboardShortcuts";
import { useTheme } from "../../hooks/useTheme";
import { Onboarding } from "../onboarding/Onboarding";
import { PageMetaProvider } from "./PageMeta";
import { Sidebar } from "./Sidebar";
import { Topbar } from "./Topbar";

export function Shell() {
  useTheme();
  useKeyboardShortcuts();
  return (
    <PageMetaProvider>
      <div className="grid h-full bg-paper text-ink" style={{ gridTemplateColumns: "auto 1fr" }}>
        <Sidebar />
        <div className="flex flex-col min-w-0 overflow-hidden">
          <Topbar />
          <main className="flex-1 overflow-y-auto px-7 pt-6 pb-8">
            <Outlet />
          </main>
        </div>
        <Onboarding />
      </div>
    </PageMetaProvider>
  );
}
