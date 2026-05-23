import { useEffect } from "react";
import { Navigate, Route, Routes, useNavigate } from "react-router-dom";

import { Shell } from "./components/layout/Shell";
import { ToastContainer } from "./components/ui/Toast";
import { useBookmarksLayoutBootstrap } from "./hooks/useBookmarksLayoutBootstrap";
import { setNoActiveOrgHandler } from "./ipc";
import { checkForUpdates } from "./lib/updater";
import { Accounting } from "./pages/Accounting";
import { BookmarkToolbar } from "./pages/BookmarkToolbar";
import { BookmarkView } from "./pages/BookmarkView";
import { CatalogEditor } from "./pages/CatalogEditor";
import { CatalogList } from "./pages/CatalogList";
import { ClientDetail } from "./pages/ClientDetail";
import { ClientEditor } from "./pages/ClientEditor";
import { ClientList } from "./pages/ClientList";
import { Activity } from "./pages/Activity";
import { Dashboard } from "./pages/Dashboard";
import { EmailTemplates } from "./pages/EmailTemplates";
import { InvoiceEditor } from "./pages/InvoiceEditor";
import { InvoiceList } from "./pages/InvoiceList";
import { OrgPicker } from "./pages/OrgPicker";
import { PaymentEditor } from "./pages/PaymentEditor";
import { PaymentList } from "./pages/PaymentList";
import { Settings } from "./pages/Settings";
import { TaxEditor } from "./pages/TaxEditor";
import { TaxList } from "./pages/TaxList";
import { TemplateEditor } from "./pages/TemplateEditor";
import { TemplateList } from "./pages/TemplateList";
import { useOrgStore } from "./stores/orgStore";

function App() {
  useBookmarksLayoutBootstrap();
  const { activeOrg, initializing, bootstrap } = useOrgStore();
  const navigate = useNavigate();

  // Boot: read active org once on mount.
  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  // One-shot silent update check on app start. Deferred a few seconds so it
  // doesn't compete with first paint or the initial org bootstrap, and so a
  // momentary offline state at launch doesn't trigger a noisy error toast.
  // Failures are swallowed in silent mode — the user can always retry from
  // Settings → About → "Check for updates".
  useEffect(() => {
    const handle = window.setTimeout(() => {
      void checkForUpdates({ silent: true });
    }, 3000);
    return () => window.clearTimeout(handle);
  }, []);

  // Wire IPC's NoActiveOrg redirect: if any command bubbles NoActiveOrg,
  // clear the store and navigate to picker.
  useEffect(() => {
    setNoActiveOrgHandler(() => {
      useOrgStore.setState({ activeOrg: null });
      navigate("/picker", { replace: true });
    });
    return () => setNoActiveOrgHandler(null);
  }, [navigate]);

  if (initializing) {
    // Brief blank-screen during boot. Sidebar/Shell would briefly mount with
    // no org and bombard the backend with NoActiveOrg errors.
    return null;
  }

  return (
    <>
    <Routes>
      {/* Toolbar route is loaded by the dedicated `bookmark-toolbar` webview
          that lives next to the bookmark on the right side of the window. */}
      <Route path="bookmark-toolbar/:id" element={<BookmarkToolbar />} />
      <Route path="picker" element={<OrgPicker />} />
      {activeOrg ? (
        // Remount the entire shell + every nested route on org switch by
        // keying on slug. All Zustand stores re-init from scratch, no stale
        // data leak from the previous org.
        <Route key={activeOrg.code} element={<Shell />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />

          <Route path="invoices" element={<InvoiceList />} />
          <Route path="invoices/create" element={<InvoiceEditor />} />
          <Route path="invoices/:id/edit" element={<InvoiceEditor />} />

          <Route path="payments" element={<PaymentList />} />
          <Route path="payments/create" element={<PaymentEditor />} />
          <Route path="payments/:id/edit" element={<PaymentEditor />} />

          <Route path="clients" element={<ClientList />} />
          <Route path="clients/create" element={<ClientEditor />} />
          <Route path="clients/:id" element={<ClientDetail />} />
          <Route path="clients/:id/edit" element={<ClientEditor />} />

          <Route path="catalog" element={<CatalogList />} />
          <Route path="catalog/create" element={<CatalogEditor />} />
          <Route path="catalog/:id/edit" element={<CatalogEditor />} />

          <Route path="taxes" element={<TaxList />} />
          <Route path="taxes/create" element={<TaxEditor />} />
          <Route path="taxes/:id/edit" element={<TaxEditor />} />

          <Route path="accounting" element={<Accounting />} />

          <Route path="templates" element={<TemplateList />} />
          <Route path="templates/create" element={<TemplateEditor />} />
          <Route path="templates/:id/edit" element={<TemplateEditor />} />

          <Route path="email-templates" element={<EmailTemplates />} />

          <Route path="bookmarks/:id" element={<BookmarkView />} />
          <Route path="activity" element={<Activity />} />
          <Route path="settings" element={<Settings />} />
        </Route>
      ) : (
        // No active org → everything else redirects to picker.
        <Route path="*" element={<Navigate to="/picker" replace />} />
      )}
    </Routes>
    {/* Toasts render above every route — including the picker, which sits
        outside the Shell. Without this, `toast.error(...)` from the picker
        flow would silently swallow the message. */}
    <ToastContainer />
    </>
  );
}

export default App;
