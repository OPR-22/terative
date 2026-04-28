import { Navigate, Route, Routes } from "react-router-dom";

import { Shell } from "./components/layout/Shell";
import { useBookmarksLayoutBootstrap } from "./hooks/useBookmarksLayoutBootstrap";
import { Accounting } from "./pages/Accounting";
import { BookmarkToolbar } from "./pages/BookmarkToolbar";
import { BookmarkView } from "./pages/BookmarkView";
import { CatalogEditor } from "./pages/CatalogEditor";
import { CatalogList } from "./pages/CatalogList";
import { ClientDetail } from "./pages/ClientDetail";
import { ClientEditor } from "./pages/ClientEditor";
import { ClientList } from "./pages/ClientList";
import { Dashboard } from "./pages/Dashboard";
import { EmailTemplates } from "./pages/EmailTemplates";
import { InvoiceEditor } from "./pages/InvoiceEditor";
import { InvoiceList } from "./pages/InvoiceList";
import { PaymentEditor } from "./pages/PaymentEditor";
import { PaymentList } from "./pages/PaymentList";
import { Settings } from "./pages/Settings";
import { TaxEditor } from "./pages/TaxEditor";
import { TaxList } from "./pages/TaxList";
import { TemplateEditor } from "./pages/TemplateEditor";
import { TemplateList } from "./pages/TemplateList";

function App() {
  useBookmarksLayoutBootstrap();
  return (
    <Routes>
      {/* Toolbar route is loaded by the dedicated `bookmark-toolbar` webview
          that lives next to the bookmark on the right side of the window. */}
      <Route path="bookmark-toolbar/:id" element={<BookmarkToolbar />} />
      <Route element={<Shell />}>
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
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

export default App;
