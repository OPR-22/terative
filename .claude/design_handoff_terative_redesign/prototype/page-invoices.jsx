// Page 2: Invoices list + editor

const ROWS = [
  {n:16, d:"2026-04-28", c:"Séraphine Thibault", st:"final", ps:"unpaid", t:1524.00, due:"28 mai", paid:0},
  {n:"—", d:"2026-04-22", c:"Séraphine Thibault", st:"draft", ps:null, t:705.51, due:null},
  {n:"—", d:"2026-04-16", c:"Benjamin Vannier", st:"draft", ps:null, t:3484.44, due:null},
  {n:1,  d:"2026-04-12", c:"Cyrille Molina",     st:"final", ps:"paid",   t:855.54, due:"12 mai", paid:855.54},
  {n:"—", d:"2026-04-07", c:"Ignace Grenier",    st:"draft", ps:null, t:629.98, due:null},
  {n:11, d:"2026-04-04", c:"Odile Julien",       st:"final", ps:"unpaid", t:3939.32, due:"04 mai", paid:0},
  {n:"—", d:"2026-04-04", c:"Valmont Aubin",     st:"draft", ps:null, t:2174.49, due:null},
  {n:10, d:"2026-01-02", c:"Alphonsine Michel",  st:"sent",  ps:"overdue", t:1819.49, due:"02 jan", paid:0},
  {n:"—", d:"2026-04-03", c:"Elena Ferreira",    st:"draft", ps:null, t:2293.26, due:null},
  {n:"—", d:"2026-04-03", c:"Benjamin Vannier",  st:"draft", ps:null, t:975.55, due:null},
  {n:7,  d:"2026-03-13", c:"Dany Vigneron",      st:"sent",  ps:"partial", t:1506.87, due:"13 avr", paid:1430.00},
  {n:12, d:"2026-03-13", c:"Angèle Chapuis",     st:"sent",  ps:"overdue", t:425.92, due:"13 avr", paid:0},
];

const StBadge = ({s}) => {
  const m = {draft:["draft","Brouillon"], final:["final","Finalisée"], sent:["sent","Envoyée"], cancel:["cancel","Annulée"]};
  const [k, l] = m[s];
  return <Badge kind={k}><span className="dotb"/>{l}</Badge>;
};
const PsBadge = ({s}) => {
  if (!s) return <span className="muted">—</span>;
  const m = {unpaid:["unpaid","Impayée"], paid:["paid","Payée"], partial:["partial","Partielle"], overdue:["overdue","En retard"]};
  const [k, l] = m[s];
  return <Badge kind={k}><span className="dotb"/>{l}</Badge>;
};

const InvoiceActions = ({ row }) => {
  if (row.st === "draft") return <>
    <button className="btn sm"><I.Edit size={11}/> Modifier</button>
    <button className="btn sm primary">Finaliser</button>
    <button className="btn sm icon" title="Dupliquer"><I.Copy size={12}/></button>
  </>;
  if (row.ps === "paid") return <>
    <button className="btn sm"><I.Eye size={11}/> Voir</button>
    <button className="btn sm icon" title="Dupliquer"><I.Copy size={12}/></button>
    <button className="btn sm icon" title="Télécharger"><I.Download size={12}/></button>
  </>;
  return <>
    <button className="btn sm"><I.Eye size={11}/> Voir</button>
    <button className="btn sm accent"><I.Send size={11}/> {row.ps === "overdue" ? "Relancer" : "Envoyer"}</button>
    <button className="btn sm">Marquer payée</button>
    <button className="btn sm icon" title="Plus"><I.More size={12}/></button>
  </>;
};

const InvoicesPage = () => (
  <Shell
    active="invoices"
    crumbs={["Cabinet Lemaire", "Factures"]}
    title="Factures"
    sub="47 factures au total · 12 affichées"
    actions={<>
      <button className="btn"><I.Download size={13}/> Exporter</button>
      <button className="btn primary"><I.Plus size={13}/> Nouvelle facture</button>
    </>}
  >
    <div style={{ display:"flex", alignItems:"center", justifyContent:"space-between", marginBottom: 14, gap:12 }}>
      <Pills value="all" options={[
        {id:"all", label:"Toutes", count:47},
        {id:"draft", label:"Brouillon", count:35},
        {id:"final", label:"Finalisée", count:5},
        {id:"sent", label:"Envoyée", count:6},
        {id:"cancel", label:"Annulée", count:1},
      ]}/>
      <div className="row" style={{gap:8}}>
        <div className="search-mini" style={{minWidth:240}}>
          <I.Search size={13}/>
          <span>N° de facture, client, montant…</span>
        </div>
        <button className="btn"><I.Filter size={13}/> Filtres</button>
      </div>
    </div>

    <div className="card">
      <table className="t">
        <thead>
          <tr>
            <th style={{width:40}}><span className="checkbox"><span className="box"/></span></th>
            <th>N°</th><th>Date</th><th>Client</th><th>Statut</th><th>Paiement</th>
            <th className="num">Total</th><th className="num">Reste dû</th>
            <th style={{width:340}}></th>
          </tr>
        </thead>
        <tbody>
          {ROWS.map((r, i) => (
            <tr key={i}>
              <td><span className="checkbox"><span className="box"/></span></td>
              <td className="mono" style={{fontFamily:"var(--font-mono)", color: r.n === "—" ? "var(--ink-4)" : "var(--ink)"}}>{r.n}</td>
              <td className="muted mono" style={{fontFamily:"var(--font-mono)"}}>{r.d}</td>
              <td><div className="row"><span className="avatar">{r.c.split(" ").map(w=>w[0]).slice(0,2).join("")}</span>{r.c}</div></td>
              <td><StBadge s={r.st}/></td>
              <td><PsBadge s={r.ps}/></td>
              <td className="num"><Money amount={r.t}/></td>
              <td className="num">{r.ps === "paid" ? <span className="muted">—</span> : r.ps === "partial" ? <Money amount={r.t - r.paid}/> : r.ps ? <Money amount={r.t}/> : <span className="muted">—</span>}</td>
              <td className="actions"><div className="row" style={{justifyContent:"flex-end", gap:4}}><InvoiceActions row={r}/></div></td>
            </tr>
          ))}
        </tbody>
      </table>
      <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", padding:"10px 14px", borderTop:"1px solid var(--line-soft)", fontSize:12, color:"var(--ink-3)"}}>
        <div>1–12 sur 47</div>
        <div className="row" style={{gap:4}}>
          <button className="btn sm" disabled style={{opacity:0.4}}>‹‹</button>
          <button className="btn sm" disabled style={{opacity:0.4}}>‹</button>
          <span className="mono" style={{padding:"0 8px"}}>1 / 4</span>
          <button className="btn sm">›</button>
          <button className="btn sm">››</button>
          <select className="select" style={{width:"auto", padding:"3px 6px", marginLeft:8, fontSize:11}}><option>12 / page</option></select>
        </div>
      </div>
    </div>
  </Shell>
);

const InvoiceEditorPage = () => (
  <Shell
    active="invoices"
    crumbs={["Cabinet Lemaire", "Factures", "#16 — Séraphine Thibault"]}
    title={<>Facture <span style={{color:"var(--ink-3)", fontFamily:"var(--font-mono)", fontSize:24}}>#16</span></>}
    sub={<>Émise le 28 avril 2026 · Échéance le 28 mai 2026 · <Badge kind="final"><span className="dotb"/>Finalisée</Badge></>}
    actions={<>
      <button className="btn"><I.ArrowLeft size={13}/> Retour</button>
      <button className="btn"><I.Copy size={13}/> Dupliquer</button>
      <button className="btn"><I.Download size={13}/> PDF</button>
      <button className="btn"><I.Check size={13}/> Marquer payée</button>
      <button className="btn primary"><I.Send size={13}/> Envoyer par e-mail</button>
    </>}
  >
    <div style={{display:"grid", gridTemplateColumns:"1fr 380px", gap:18}}>
      <div className="card">
        <div className="card-head">
          <div className="card-title">Informations</div>
          <div className="tiny muted row" style={{gap:6}}><span className="dot-status ok"/> Enregistré il y a 4 s</div>
        </div>
        <div className="card-body">
          <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
            <div className="field">
              <label className="label">Client</label>
              <div className="select" style={{display:"flex", justifyContent:"space-between", alignItems:"center"}}>
                <span className="row" style={{gap:8}}><span className="avatar">ST</span> Séraphine Thibault</span>
                <I.ChevDown size={13}/>
              </div>
              <div className="field-help">contact@thibault.fr · FR</div>
            </div>
            <div className="field">
              <label className="label">Modèle</label>
              <div className="select" style={{display:"flex", justifyContent:"space-between", alignItems:"center"}}>
                <span>Modèle par défaut · Classique</span>
                <I.ChevDown size={13}/>
              </div>
            </div>
            <div className="field">
              <label className="label">Date</label>
              <div className="input" style={{display:"flex",justifyContent:"space-between"}}>28 avril 2026 <I.Calendar size={13}/></div>
            </div>
            <div className="field">
              <label className="label">Échéance</label>
              <div className="row" style={{gap:8}}>
                <input className="input mono" defaultValue="30" style={{width:64}}/>
                <span className="muted tiny">jours · échéance le 28 mai 2026</span>
              </div>
            </div>
          </div>
          <div className="field">
            <label className="label">Notes (visibles sur la facture)</label>
            <textarea className="textarea" rows="3" defaultValue="Merci pour votre confiance. Règlement par virement bancaire — coordonnées au verso."></textarea>
          </div>
        </div>

        <div className="card-head" style={{borderTop:"1px solid var(--line)"}}>
          <div className="card-title">Lignes</div>
          <button className="btn sm"><I.Plus size={11}/> Ajouter depuis le catalogue</button>
        </div>
        <table className="t">
          <thead>
            <tr><th style={{width:24}}></th><th>Description</th><th className="num" style={{width:80}}>Qté</th><th className="num" style={{width:140}}>Prix unitaire</th><th className="num" style={{width:120}}>Total</th><th style={{width:36}}></th></tr>
          </thead>
          <tbody>
            <tr>
              <td><I.Drag size={14} style={{color:"var(--ink-4)"}}/></td>
              <td>
                <input className="input" defaultValue="Audit 471 — analyse trimestrielle" style={{border:"1px solid transparent",padding:"4px 6px"}}/>
                <div className="tiny muted" style={{marginLeft:6}}>REF-471 · forfait</div>
              </td>
              <td className="num"><input className="input mono" defaultValue="1" style={{textAlign:"right"}}/></td>
              <td className="num"><input className="input mono" defaultValue="1 200,00" style={{textAlign:"right"}}/></td>
              <td className="num"><Money amount={1200}/></td>
              <td><I.X size={13} style={{color:"var(--ink-3)", cursor:"pointer"}}/></td>
            </tr>
            <tr>
              <td><I.Drag size={14} style={{color:"var(--ink-4)"}}/></td>
              <td><span className="muted">+ Ajouter une ligne</span></td>
              <td colSpan="4"></td>
            </tr>
          </tbody>
        </table>
      </div>

      <div style={{display:"flex", flexDirection:"column", gap:14}}>
        <div className="card">
          <div className="card-head"><div className="card-title">Totaux</div></div>
          <div className="card-body">
            <div style={{display:"flex", justifyContent:"space-between", padding:"6px 0", fontSize:13}}><span className="muted">Sous-total</span><Money amount={1200}/></div>
            <div style={{display:"flex", justifyContent:"space-between", padding:"6px 0", fontSize:13}}><span className="muted">TVA 6 %</span><Money amount={72}/></div>
            <div style={{display:"flex", justifyContent:"space-between", padding:"6px 0", fontSize:13}}><span className="muted">TVH 21 %</span><Money amount={252}/></div>
            <div style={{borderTop:"1px solid var(--line)", marginTop:8, paddingTop:12, display:"flex", justifyContent:"space-between", alignItems:"baseline"}}>
              <span style={{fontWeight:500}}>Total</span>
              <span className="money" style={{fontFamily:"var(--font-serif)", fontSize:26, letterSpacing:"-0.02em"}}>1 524,00<span className="cur" style={{fontSize:14}}> €</span></span>
            </div>
          </div>
        </div>

        <div className="card">
          <div className="card-head"><div className="card-title">Taxes appliquées</div></div>
          <div className="card-body">
            <label className="checkbox on" style={{display:"flex", padding:"6px 0", justifyContent:"space-between"}}>
              <span className="row" style={{gap:8}}><span className="box"/> TVA 6 %<span className="muted tiny">FR-78-6</span></span>
              <span className="mono muted">6,00 %</span>
            </label>
            <label className="checkbox on" style={{display:"flex", padding:"6px 0", justifyContent:"space-between"}}>
              <span className="row" style={{gap:8}}><span className="box"/> TVH 21 %<span className="muted tiny">BE-21</span></span>
              <span className="mono muted">21,00 %</span>
            </label>
            <label className="checkbox" style={{display:"flex", padding:"6px 0", justifyContent:"space-between", color:"var(--ink-3)"}}>
              <span className="row" style={{gap:8}}><span className="box"/> Exonération art. 44</span>
              <span className="mono muted">0,00 %</span>
            </label>
          </div>
        </div>

        <div className="card">
          <div className="card-head"><div className="card-title">Historique d'envoi</div></div>
          <div className="card-body" style={{padding:"6px 18px 14px"}}>
            <div style={{display:"flex", gap:10, padding:"8px 0", borderBottom:"1px solid var(--line-soft)"}}>
              <I.Send size={13} style={{color:"var(--ink-3)", marginTop:2}}/>
              <div style={{flex:1, fontSize:12.5}}>
                <div>Premier contact</div>
                <div className="tiny muted">contact@thibault.fr · 28 avr. 2026, 14:32</div>
              </div>
            </div>
            <div className="tiny muted" style={{padding:"8px 0"}}>1 envoi enregistré</div>
          </div>
        </div>
      </div>
    </div>
  </Shell>
);

window.InvoicesPage = InvoicesPage;
window.InvoiceEditorPage = InvoiceEditorPage;
