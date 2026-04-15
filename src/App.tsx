import { Navigate, Route, Routes } from "react-router-dom";

import { Shell } from "./components/layout/Shell";
import { Accounting } from "./pages/Accounting";
import { CatalogList } from "./pages/CatalogList";
import { ClientDetail } from "./pages/ClientDetail";
import { ClientList } from "./pages/ClientList";
import { Dashboard } from "./pages/Dashboard";
import { InvoiceList } from "./pages/InvoiceList";
import { PaymentList } from "./pages/PaymentList";
import { Settings } from "./pages/Settings";
import { TaxList } from "./pages/TaxList";
import { TemplateList } from "./pages/TemplateList";

function App() {
  return (
    <Routes>
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
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

export default App;
