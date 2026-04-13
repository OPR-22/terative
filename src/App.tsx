import { Navigate, Route, Routes } from "react-router-dom";
import { useTranslation } from "react-i18next";

import { Shell } from "./components/layout/Shell";
import { ClientList } from "./pages/ClientList";
import { Placeholder } from "./pages/Placeholder";
import { ServiceList } from "./pages/ServiceList";
import { Settings } from "./pages/Settings";

function App() {
  const { t } = useTranslation();
  return (
    <Routes>
      <Route element={<Shell />}>
        <Route index element={<Navigate to="/clients" replace />} />
        <Route
          path="dashboard"
          element={<Placeholder title={t("nav.dashboard")} />}
        />
        <Route
          path="invoices"
          element={<Placeholder title={t("nav.invoices")} />}
        />
        <Route
          path="payments"
          element={<Placeholder title={t("nav.payments")} />}
        />
        <Route path="clients" element={<ClientList />} />
        <Route path="services" element={<ServiceList />} />
        <Route
          path="accounting"
          element={<Placeholder title={t("nav.accounting")} />}
        />
        <Route
          path="templates"
          element={<Placeholder title={t("nav.templates")} />}
        />
        <Route path="settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

export default App;
