// Page 1: Dashboard — Tableau de bord

const DashboardPage = () => (
  <Shell
    active="dashboard"
    crumbs={["Cabinet Lemaire", "Tableau de bord"]}
    title="Tableau de bord"
    sub="Vue d'ensemble — semaine du 27 avril 2026"
    actions={<>
      <button className="btn"><I.Download size={13}/> Exporter</button>
      <button className="btn primary"><I.Plus size={13}/> Nouvelle facture</button>
    </>}
  >
    <div className="kpi-grid">
      <div className="kpi">
        <div className="kpi-label">Revenu cette année</div>
        <div className="kpi-value">17 950,54 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div>
        <div className="kpi-meta"><I.ArrowUp size={11} style={{color:"var(--ok)"}}/> +12,4 % vs 2025 · 14 factures</div>
      </div>
      <div className="kpi warn">
        <div className="kpi-label">En attente de paiement</div>
        <div className="kpi-value">7 784,60 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div>
        <div className="kpi-meta">8 factures envoyées · 3 partiellement payées</div>
      </div>
      <div className="kpi danger">
        <div className="kpi-label">En retard</div>
        <div className="kpi-value">2 320,28 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div>
        <div className="kpi-meta"><I.CircleAlert size={11}/> 3 factures · plus ancienne 116 j</div>
      </div>
      <div className="kpi">
        <div className="kpi-label">Brouillons</div>
        <div className="kpi-value">35</div>
        <div className="kpi-meta">Dernier modifié il y a 2 h</div>
      </div>
    </div>

    <div className="section-title">À traiter</div>
    <div style={{ display: "grid", gridTemplateColumns: "1.6fr 1fr", gap: 16 }}>
      <div className="card">
        <div className="card-head">
          <div>
            <div className="card-title">Factures en retard</div>
            <div className="card-sub">3 factures à relancer · 2 320,28 € total</div>
          </div>
          <button className="btn sm">Voir tout</button>
        </div>
        <table className="t">
          <thead>
            <tr>
              <th>N°</th><th>Client</th><th>Échéance</th><th>Retard</th><th className="num">Montant dû</th><th></th>
            </tr>
          </thead>
          <tbody>
            <tr>
              <td className="muted">#10</td>
              <td><div className="row"><span className="avatar">AM</span> Alphonsine Michel</div></td>
              <td className="muted">2 janv. 2026</td>
              <td><Badge kind="overdue"><span className="dotb"/>116 j</Badge></td>
              <td className="num"><Money amount={1819.49}/></td>
              <td className="actions"><button className="btn sm"><I.Send size={12}/> Relance</button></td>
            </tr>
            <tr>
              <td className="muted">#7</td>
              <td><div className="row"><span className="avatar">DV</span> Dany Vigneron</div></td>
              <td className="muted">13 avr. 2026</td>
              <td><Badge kind="overdue"><span className="dotb"/>15 j</Badge></td>
              <td className="num"><Money amount={75.87}/></td>
              <td className="actions"><button className="btn sm"><I.Send size={12}/> Relance</button></td>
            </tr>
            <tr>
              <td className="muted">#12</td>
              <td><div className="row"><span className="avatar">AC</span> Angèle Chapuis</div></td>
              <td className="muted">13 avr. 2026</td>
              <td><Badge kind="overdue"><span className="dotb"/>15 j</Badge></td>
              <td className="num"><Money amount={425.92}/></td>
              <td className="actions"><button className="btn sm"><I.Send size={12}/> Relance</button></td>
            </tr>
          </tbody>
        </table>
      </div>

      <div className="card">
        <div className="card-head">
          <div className="card-title">Activité récente</div>
        </div>
        <div style={{padding:"6px 0"}}>
          {[
            {ic: I.Pay, t:"Paiement reçu", d:"Alphonsine Michel · 5 300,82 €", w:"il y a 23 min", c:"ok"},
            {ic: I.Send, t:"Facture envoyée", d:"#16 · Séraphine Thibault", w:"il y a 2 h", c:"info"},
            {ic: I.Edit, t:"Brouillon modifié", d:"#— · Benjamin Vannier", w:"il y a 5 h", c:"idle"},
            {ic: I.Receipt, t:"Facture finalisée", d:"#16 · 1 524,00 €", w:"hier, 17:42", c:"info"},
            {ic: I.Users, t:"Nouveau client", d:"Cyrielle Tardy", w:"hier, 14:18", c:"idle"},
          ].map((a, i) => {
            const Ic = a.ic;
            return (
              <div key={i} style={{display:"flex", gap:12, padding:"10px 18px", alignItems:"flex-start", borderBottom: i < 4 ? "1px solid var(--line-soft)" : "none"}}>
                <span style={{width:24, height:24, borderRadius:4, background:"var(--paper-2)", display:"grid", placeItems:"center", color:"var(--ink-2)", flex:"0 0 24px"}}><Ic size={13}/></span>
                <div style={{flex:1, minWidth:0}}>
                  <div style={{fontSize:12.5, fontWeight:500}}>{a.t}</div>
                  <div className="muted tiny" style={{marginTop:2}}>{a.d}</div>
                </div>
                <div className="tiny muted" style={{whiteSpace:"nowrap"}}>{a.w}</div>
              </div>
            );
          })}
        </div>
      </div>
    </div>

    <div className="section-title">Sur les 12 derniers mois</div>
    <div className="card">
      <div className="card-head">
        <div className="card-title">Revenu mensuel</div>
        <div className="row" style={{gap:14}}>
          <div className="row tiny muted"><span style={{width:8,height:8,background:"var(--accent)"}}/> Encaissé</div>
          <div className="row tiny muted"><span style={{width:8,height:8,background:"var(--paper-3)",border:"1px solid var(--line)"}}/> Facturé</div>
        </div>
      </div>
      <div style={{padding:"22px 28px 18px"}}>
        <svg viewBox="0 0 720 160" style={{width:"100%", height:160, display:"block"}}>
          {[1200,1900,1500,2200,2800,2100,1800,2600,3100,2400,2900,3300].map((v, i) => {
            const x = 20 + i * 58;
            const h = (v / 3500) * 120;
            const y = 140 - h;
            return <g key={i}>
              <rect x={x} y={20} width="32" height="120" fill="var(--paper-3)" />
              <rect x={x} y={y} width="32" height={h} fill="var(--accent)" />
              <text x={x+16} y="156" textAnchor="middle" fontSize="9" fill="var(--ink-3)" fontFamily="var(--font-mono)">
                {["mai","jui","jul","aoû","sep","oct","nov","déc","jan","fév","mar","avr"][i]}
              </text>
            </g>;
          })}
          <line x1="20" y1="140" x2="710" y2="140" stroke="var(--line)" strokeWidth="1"/>
        </svg>
      </div>
    </div>
  </Shell>
);

window.DashboardPage = DashboardPage;
