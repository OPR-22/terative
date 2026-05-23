// Terative — Shared icons + sidebar + shell components
// Lucide-style 1.5px stroke icons

const Icon = ({ d, size = 16, fill = "none", style, className }) => (
  <svg className={className} width={size} height={size} viewBox="0 0 24 24" fill={fill} stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" style={style}>
    {typeof d === "string" ? <path d={d} /> : d}
  </svg>
);

const I = {
  Dashboard: (p) => <Icon {...p} d={<><rect x="3" y="3" width="7" height="9"/><rect x="14" y="3" width="7" height="5"/><rect x="14" y="12" width="7" height="9"/><rect x="3" y="16" width="7" height="5"/></>}/>,
  Invoice: (p) => <Icon {...p} d={<><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/><path d="M8 13h8"/><path d="M8 17h5"/></>}/>,
  Pay: (p) => <Icon {...p} d={<><rect x="2" y="6" width="20" height="12" rx="1"/><circle cx="12" cy="12" r="2.5"/><path d="M6 12h.01M18 12h.01"/></>}/>,
  Users: (p) => <Icon {...p} d={<><path d="M16 21v-2a4 4 0 0 0-4-4H6a4 4 0 0 0-4 4v2"/><circle cx="9" cy="7" r="4"/><path d="M22 21v-2a4 4 0 0 0-3-3.87"/><path d="M16 3.13a4 4 0 0 1 0 7.75"/></>}/>,
  Box: (p) => <Icon {...p} d={<><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><path d="M3.27 6.96 12 12.01l8.73-5.05M12 22.08V12"/></>}/>,
  Percent: (p) => <Icon {...p} d={<><line x1="19" y1="5" x2="5" y2="19"/><circle cx="6.5" cy="6.5" r="2.5"/><circle cx="17.5" cy="17.5" r="2.5"/></>}/>,
  Book: (p) => <Icon {...p} d={<><path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"/><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"/></>}/>,
  Layout: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="1"/><path d="M3 9h18M9 21V9"/></>}/>,
  Mail: (p) => <Icon {...p} d={<><rect x="2" y="4" width="20" height="16" rx="1"/><path d="m22 7-10 6L2 7"/></>}/>,
  Cog: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/></>}/>,
  Bookmark: (p) => <Icon {...p} d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>,
  Search: (p) => <Icon {...p} d={<><circle cx="11" cy="11" r="7"/><path d="m21 21-4.3-4.3"/></>}/>,
  Plus: (p) => <Icon {...p} d="M12 5v14M5 12h14"/>,
  Chevron: (p) => <Icon {...p} d="m9 6 6 6-6 6"/>,
  ChevDown: (p) => <Icon {...p} d="m6 9 6 6 6-6"/>,
  ChevUp: (p) => <Icon {...p} d="m18 15-6-6-6 6"/>,
  Send: (p) => <Icon {...p} d={<><path d="m22 2-7 20-4-9-9-4z"/><path d="M22 2 11 13"/></>}/>,
  Edit: (p) => <Icon {...p} d={<><path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/><path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4z"/></>}/>,
  Eye: (p) => <Icon {...p} d={<><path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7z"/><circle cx="12" cy="12" r="3"/></>}/>,
  Copy: (p) => <Icon {...p} d={<><rect x="9" y="9" width="13" height="13" rx="1"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></>}/>,
  Trash: (p) => <Icon {...p} d={<><path d="M3 6h18"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/></>}/>,
  Archive: (p) => <Icon {...p} d={<><rect x="2" y="3" width="20" height="5"/><path d="M4 8v11a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8"/><path d="M10 12h4"/></>}/>,
  Check: (p) => <Icon {...p} d="M20 6 9 17l-5-5"/>,
  X: (p) => <Icon {...p} d="M18 6 6 18M6 6l18 12" />,
  Calendar: (p) => <Icon {...p} d={<><rect x="3" y="4" width="18" height="18" rx="1"/><path d="M16 2v4M8 2v4M3 10h18"/></>}/>,
  CircleAlert: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="10"/><path d="M12 8v4M12 16h.01"/></>}/>,
  ArrowUp: (p) => <Icon {...p} d="M12 19V5M5 12l7-7 7 7"/>,
  ArrowDown: (p) => <Icon {...p} d="M12 5v14M19 12l-7 7-7-7"/>,
  ArrowLeft: (p) => <Icon {...p} d="M19 12H5M12 19l-7-7 7-7"/>,
  Filter: (p) => <Icon {...p} d="M22 3H2l8 9.46V19l4 2v-8.54z"/>,
  Download: (p) => <Icon {...p} d={<><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M7 10l5 5 5-5"/><path d="M12 15V3"/></>}/>,
  Upload: (p) => <Icon {...p} d={<><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><path d="M17 8l-5-5-5 5"/><path d="M12 3v12"/></>}/>,
  Star: (p) => <Icon {...p} d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01z"/>,
  Refresh: (p) => <Icon {...p} d={<><path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/></>}/>,
  Home: (p) => <Icon {...p} d={<><path d="m3 9 9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/><path d="M9 22V12h6v10"/></>}/>,
  Globe: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15 15 0 0 1 0 20M12 2a15 15 0 0 0 0 20"/></>}/>,
  Lock: (p) => <Icon {...p} d={<><rect x="3" y="11" width="18" height="11" rx="1"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/></>}/>,
  Sun: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M4.93 19.07l1.41-1.41M17.66 6.34l1.41-1.41"/></>}/>,
  Moon: (p) => <Icon {...p} d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>,
  Building: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="1"/><path d="M9 22V12h6v10M9 7h.01M15 7h.01M9 12h.01M15 12h.01"/></>}/>,
  Receipt: (p) => <Icon {...p} d={<><path d="M4 2v20l3-2 3 2 3-2 3 2 3-2 3 2V2l-3 2-3-2-3 2-3-2-3 2z"/><path d="M8 9h8M8 13h6"/></>}/>,
  Trend: (p) => <Icon {...p} d={<><polyline points="22 7 13.5 15.5 8.5 10.5 2 17"/><polyline points="16 7 22 7 22 13"/></>}/>,
  Clock: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></>}/>,
  Image: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="1"/><circle cx="9" cy="9" r="2"/><path d="m21 15-5-5L5 21"/></>}/>,
  Database: (p) => <Icon {...p} d={<><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14a9 3 0 0 0 18 0V5"/><path d="M3 12a9 3 0 0 0 18 0"/></>}/>,
  Sidebar: (p) => <Icon {...p} d={<><rect x="3" y="3" width="18" height="18" rx="1"/><path d="M9 3v18"/></>}/>,
  More: (p) => <Icon {...p} d={<><circle cx="12" cy="5" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="12" cy="19" r="1"/></>}/>,
  Drag: (p) => <Icon {...p} d={<><circle cx="9" cy="6" r="0.5" fill="currentColor"/><circle cx="9" cy="12" r="0.5" fill="currentColor"/><circle cx="9" cy="18" r="0.5" fill="currentColor"/><circle cx="15" cy="6" r="0.5" fill="currentColor"/><circle cx="15" cy="12" r="0.5" fill="currentColor"/><circle cx="15" cy="18" r="0.5" fill="currentColor"/></>}/>,
  Phone: (p) => <Icon {...p} d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07 19.5 19.5 0 0 1-6-6 19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 4.11 2h3a2 2 0 0 1 2 1.72c.13.96.37 1.9.72 2.81a2 2 0 0 1-.45 2.11L8.09 9.91a16 16 0 0 0 6 6l1.27-1.27a2 2 0 0 1 2.11-.45c.91.35 1.85.59 2.81.72A2 2 0 0 1 22 16.92z"/>,
  AtSign: (p) => <Icon {...p} d={<><circle cx="12" cy="12" r="4"/><path d="M16 8v5a3 3 0 0 0 6 0v-1a10 10 0 1 0-3.92 7.94"/></>}/>,
};

// Sidebar
const NAV = [
  { id: "dashboard",  label: "Tableau de bord",  icon: I.Dashboard },
  { id: "invoices",   label: "Factures",         icon: I.Invoice, badge: "47" },
  { id: "payments",   label: "Paiements",        icon: I.Pay },
  { id: "clients",    label: "Clients",          icon: I.Users },
  { id: "catalog",    label: "Catalogue",        icon: I.Box },
  { id: "taxes",      label: "Taxes",            icon: I.Percent },
  { id: "accounting", label: "Comptabilité",     icon: I.Book },
  { id: "templates",  label: "Modèles",          icon: I.Layout },
  { id: "emails",     label: "E-mails",          icon: I.Mail },
  { id: "settings",   label: "Paramètres",       icon: I.Cog },
];
const FAVS = [
  { label: "Wikipedia", host: "wikipedia.org" },
  { label: "Google",    host: "google.com" },
  { label: "GitHub",    host: "github.com" },
];

const Sidebar = ({ active }) => (
  <aside className="sidebar">
    <div className="sb-brand">
      <div className="sb-mark">
        <span className="dot" />
        <span className="name">Terative</span>
      </div>
      <span className="sb-collapse" title="Réduire"><I.Sidebar size={16}/></span>
    </div>
    <div className="sb-section">
      <nav className="sb-nav">
        {NAV.map(n => {
          const Ic = n.icon;
          return (
            <div key={n.id} className={"sb-item" + (active === n.id ? " active" : "")}>
              <Ic className="icon" />
              <span>{n.label}</span>
              {n.badge && <span className="badge">{n.badge}</span>}
            </div>
          );
        })}
      </nav>
    </div>
    <div className="sb-section">
      <div className="sb-label">Favoris</div>
      <nav className="sb-nav">
        {FAVS.map(f => (
          <div key={f.label} className="sb-item">
            <I.Bookmark className="icon" />
            <span>{f.label}</span>
          </div>
        ))}
      </nav>
    </div>
    <div className="sb-foot">
      <div className="sb-avatar">CL</div>
      <div style={{lineHeight:1.2}}>
        <div style={{color:"var(--ink)", fontWeight:500}}>Camille L.</div>
        <div className="tiny">Cabinet Lemaire</div>
      </div>
    </div>
  </aside>
);

const Topbar = ({ crumbs = [], children }) => (
  <div className="topbar">
    <div className="crumbs">
      {crumbs.map((c, i) => (
        <React.Fragment key={i}>
          {i > 0 && <span className="sep">/</span>}
          <span className={i === crumbs.length - 1 ? "here" : ""}>{c}</span>
        </React.Fragment>
      ))}
    </div>
    <div className="top-actions">
      <div className="search-mini">
        <I.Search size={13}/>
        <span>Rechercher partout</span>
        <kbd>⌘K</kbd>
      </div>
      {children}
    </div>
  </div>
);

const PageHead = ({ title, sub, actions }) => (
  <div className="page-head">
    <div>
      <h1 className="page-title">{title}</h1>
      {sub && <div className="page-sub">{sub}</div>}
    </div>
    {actions && <div className="page-actions">{actions}</div>}
  </div>
);

const Money = ({ amount, currency = "€", lg = false }) => (
  <span className={"money" + (lg ? " lg" : "")}>
    {Number(amount).toLocaleString("fr-FR", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
    <span className="cur"> {currency}</span>
  </span>
);

const Badge = ({ kind, children }) => <span className={"badge " + kind}>{children}</span>;

const Pills = ({ options, value }) => (
  <div className="pills">
    {options.map(o => (
      <span key={o.id} className={"pill" + (o.id === value ? " active" : "")}>
        {o.label}{o.count != null && <span className="ct">{o.count}</span>}
      </span>
    ))}
  </div>
);

const Frame = ({ title = "Terative", w = 1240, h = 820, children }) => (
  <div style={{ width: w, height: h, background: "var(--paper)", overflow: "hidden", display: "flex", flexDirection: "column", border: "1px solid var(--line)" }}>
    <div className="win-chrome">
      <div className="dots"><span className="r"/><span className="y"/><span className="g"/></div>
      <div className="title">{title}</div>
      <div className="sp"/>
    </div>
    <div style={{ flex: 1, minHeight: 0 }}>{children}</div>
  </div>
);

const Shell = ({ active, crumbs, title, sub, actions, children, topRight }) => (
  <div className="app">
    <Sidebar active={active} />
    <div className="main">
      <Topbar crumbs={crumbs}>{topRight}</Topbar>
      <div className="content">
        <PageHead title={title} sub={sub} actions={actions} />
        {children}
      </div>
    </div>
  </div>
);

Object.assign(window, { Icon, I, Sidebar, Topbar, PageHead, Money, Badge, Pills, Frame, Shell, NAV, FAVS });
