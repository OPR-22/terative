import { Navigate, Route, Routes } from "react-router-dom";

import { Shell } from "./components/layout/Shell";
import { useBookmarksLayoutBootstrap } from "./hooks/useBookmarksLayoutBootstrap";
import { Accounting } from "./pages/Accounting";
import { BookmarkToolbar } from "./pages/BookmarkToolbar";
import { BookmarkView } from "./pages/BookmarkView";
import { CatalogList } from "./pages/CatalogList";
import { ClientDetail } from "./pages/ClientDetail";
import { ClientList } from "./pages/ClientList";
import { Dashboard } from "./pages/Dashboard";
import { InvoiceList } from "./pages/InvoiceList";
import { PaymentList } from "./pages/PaymentList";
import { EmailTemplates } from "./pages/EmailTemplates";
import { Settings } from "./pages/Settings";
import { TaxList } from "./pages/TaxList";
import { TemplateList } from "./pages/TemplateList";

function App() {
  useBookmarksLayoutBootstrap();
  return (
    <Routes>
      {/* Toolbar route is loaded by the dedicated `bookmark-toolbar` webview
          that lives next to the bookmark on the right side of the window.
          No Shell — just the toolbar UI. */}
      <Route path="bookmark-toolbar/:id" element={<BookmarkToolbar />} />
      <Route element={<Shell />}>
        <Route index element={<Navigate to="/dashboard" replace />} />
        <Route path="dashboard" element={<Dashboard />} />
        <Route path="invoices" element={<InvoiceList />} />
        <Route path="payments" element={<PaymentList />} />
        <Route path="clients" element={<ClientList />} />
        <Route path="clients/:id" element={<ClientDetail />} />
        <Route path="catalog" element={<CatalogList />} />
        <Route path="taxes" element={<TaxList />} />
        <Route path="accounting" element={<Accounting />} />
        <Route path="templates" element={<TemplateList />} />
        <Route path="email-templates" element={<EmailTemplates />} />
        <Route path="bookmarks/:id" element={<BookmarkView />} />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

export default App;
