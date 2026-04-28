// Pages: Catalogue, Taxes, Comptabilité (3 tabs)

const CATALOG = [
  {k:"Produit",   n:"Carnet 321",       r:"REF-321", p:15.00, u:"pièce", a:true},
  {k:"Produit",   n:"Carnet 681",       r:"REF-681", p:15.00, u:"pièce", a:true},
  {k:"Produit",   n:"Carnet 892",       r:"REF-892", p:15.00, u:"pièce", a:true},
  {k:"Produit",   n:"Stylo plume 981",  r:"REF-981", p:45.00, u:"pièce", a:true},
  {k:"Prestation",n:"Audit 268",        r:"REF-268", p:1200.00,u:"forfait",a:true},
  {k:"Prestation",n:"Audit 324",        r:"REF-324", p:1200.00,u:"forfait",a:true},
  {k:"Prestation",n:"Audit 471",        r:"REF-471", p:1200.00,u:"forfait",a:true},
  {k:"Prestation",n:"Audit 680",        r:"REF-680", p:1200.00,u:"forfait",a:true},
  {k:"Prestation",n:"Suivi mensuel 597",r:"REF-597", p:500.00, u:"mois",  a:true},
  {k:"Prestation",n:"Suivi mensuel 790",r:"REF-790", p:500.00, u:"mois",  a:false},
];

const CatalogPage = () => (
  <Shell active="catalog" crumbs={["Cabinet Lemaire","Catalogue"]}
    title="Catalogue"
    sub="10 articles · 4 produits · 6 prestations"
    actions={<>
      <button className="btn"><I.Upload size={13}/> Importer CSV</button>
      <button className="btn primary"><I.Plus size={13}/> Nouvel article</button>
    </>}>
    <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:14}}>
      <Pills value="all" options={[{id:"all",label:"Tous",count:10},{id:"p",label:"Prestations",count:6},{id:"prod",label:"Produits",count:4}]}/>
      <span className="checkbox"><span className="box"/> Inclure les archivés</span>
    </div>
    <div className="card">
      <table className="t">
        <thead><tr><th>Type</th><th>Nom</th><th>Référence</th><th className="num">Prix par défaut</th><th>Unité</th><th>Statut</th><th></th></tr></thead>
        <tbody>{CATALOG.map((c,i)=>(
          <tr key={i}>
            <td><Badge kind={c.k==="Produit"?"outline":"final"}>{c.k}</Badge></td>
            <td style={{fontWeight:500}}>{c.n}</td>
            <td className="muted mono">{c.r}</td>
            <td className="num"><Money amount={c.p}/></td>
            <td className="muted">{c.u}</td>
            <td>{c.a ? <span className="row tiny" style={{color:"var(--ok)"}}><span className="dot-status ok"/> Actif</span> : <span className="row tiny muted"><span className="dot-status idle"/> Archivé</span>}</td>
            <td className="actions"><div className="row" style={{justifyContent:"flex-end", gap:4}}>
              <button className="btn sm"><I.Edit size={11}/> Modifier</button>
              <button className="btn sm icon"><I.Archive size={12}/></button>
            </div></td>
          </tr>
        ))}</tbody>
      </table>
    </div>

    <div className="section-title">Aperçu — édition d'article</div>
    <div className="card" style={{maxWidth:520}}>
      <div className="card-head"><div className="card-title">Modifier — Audit 471</div><I.X size={14}/></div>
      <div className="card-body">
        <div className="field"><label className="label">Type</label>
          <Pills value="p" options={[{id:"p",label:"Prestation"},{id:"prod",label:"Produit"}]}/>
        </div>
        <div className="field"><label className="label">Nom</label><input className="input" defaultValue="Audit 471"/></div>
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
          <div className="field"><label className="label">Référence</label><input className="input mono" defaultValue="REF-471"/></div>
          <div className="field"><label className="label">Unité</label><div className="select" style={{display:"flex",justifyContent:"space-between"}}>forfait<I.ChevDown size={13}/></div></div>
        </div>
        <div className="field"><label className="label">Prix par défaut</label><div className="row" style={{gap:8}}><input className="input mono" defaultValue="1 200,00" style={{textAlign:"right"}}/><span className="muted">EUR</span></div></div>
        <div className="row" style={{justifyContent:"flex-end", gap:8, marginTop:8}}>
          <button className="btn">Annuler</button>
          <button className="btn primary">Enregistrer</button>
        </div>
      </div>
    </div>
  </Shell>
);

const TaxesPage = () => (
  <Shell active="taxes" crumbs={["Cabinet Lemaire","Taxes"]}
    title="Taxes"
    sub="2 taxes actives · utilisées sur 11 factures"
    actions={<button className="btn primary"><I.Plus size={13}/> Nouvelle taxe</button>}>
    <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:14}}>
      <span className="checkbox"><span className="box"/> Inclure les archivés</span>
    </div>
    <div className="card" style={{maxWidth:920}}>
      <table className="t">
        <thead><tr><th>Nom</th><th className="num">Taux</th><th>N° fiscal</th><th>Statut</th><th className="num">Factures</th><th></th></tr></thead>
        <tbody>
          <tr>
            <td style={{fontWeight:500}}>TVA 6 %</td>
            <td className="num mono" style={{fontSize:14}}>6,00 %</td>
            <td className="muted mono">FR-78-6</td>
            <td><span className="row tiny" style={{color:"var(--ok)"}}><span className="dot-status ok"/> Actif</span></td>
            <td className="num muted mono">9</td>
            <td className="actions"><button className="btn sm"><I.Edit size={11}/> Modifier</button> <button className="btn sm icon"><I.Archive size={12}/></button></td>
          </tr>
          <tr>
            <td style={{fontWeight:500}}>TVH 21 %</td>
            <td className="num mono" style={{fontSize:14}}>21,00 %</td>
            <td className="muted mono">BE-21</td>
            <td><span className="row tiny" style={{color:"var(--ok)"}}><span className="dot-status ok"/> Actif</span></td>
            <td className="num muted mono">11</td>
            <td className="actions"><button className="btn sm"><I.Edit size={11}/> Modifier</button> <button className="btn sm icon"><I.Archive size={12}/></button></td>
          </tr>
        </tbody>
      </table>
    </div>
    <div className="section-title">Aperçu — édition</div>
    <div className="card" style={{maxWidth:480}}>
      <div className="card-head"><div className="card-title">Modifier — TVA 6 %</div><I.X size={14}/></div>
      <div className="card-body">
        <div className="field"><label className="label">Nom</label><input className="input" defaultValue="TVA 6 %"/></div>
        <div className="field"><label className="label">Pourcentage</label><div className="row" style={{gap:8}}><input className="input mono" defaultValue="6,00" style={{textAlign:"right"}}/><span className="muted">%</span></div></div>
        <div className="field"><label className="label">Numéro fiscal</label><input className="input mono" defaultValue="FR-78-6" placeholder="Optionnel"/></div>
        <div className="row" style={{justifyContent:"flex-end", gap:8, marginTop:8}}>
          <button className="btn">Annuler</button><button className="btn primary">Enregistrer</button>
        </div>
      </div>
    </div>
  </Shell>
);

const AccountingPage = () => (
  <Shell active="accounting" crumbs={["Cabinet Lemaire","Comptabilité"]}
    title="Comptabilité" sub="Période — 1 janvier → 31 décembre 2026"
    actions={<>
      <button className="btn"><I.Download size={13}/> Exporter CSV</button>
      <button className="btn"><I.Download size={13}/> PDF</button>
    </>}>
    <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:18, gap:12}}>
      <Pills value="rev" options={[{id:"rev",label:"Revenus"},{id:"age",label:"Vieillissement"},{id:"sld",label:"Soldes clients"}]}/>
      <div className="row" style={{gap:8}}>
        <div className="input mono row" style={{width:"auto", gap:8}}><I.Calendar size={13}/> 01/01/2026 → 31/12/2026</div>
        <div className="select row" style={{width:"auto", gap:8}}>Regrouper : Mois <I.ChevDown size={13}/></div>
      </div>
    </div>

    <div style={{display:"grid", gridTemplateColumns:"repeat(4, 1fr)", gap:14, marginBottom:20}}>
      <div className="kpi"><div className="kpi-label">Total revenu</div><div className="kpi-value">17 950,54 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div><div className="kpi-meta">14 factures finalisées</div></div>
      <div className="kpi"><div className="kpi-label">Encaissé</div><div className="kpi-value" style={{color:"var(--ok)"}}>15 630,26 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div><div className="kpi-meta">11 paiements</div></div>
      <div className="kpi"><div className="kpi-label">Reste à encaisser</div><div className="kpi-value" style={{color:"oklch(0.5 0.13 60)"}}>2 320,28 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div><div className="kpi-meta">3 factures en retard</div></div>
      <div className="kpi"><div className="kpi-label">Panier moyen</div><div className="kpi-value">1 282,18 <span style={{fontSize:18,color:"var(--ink-3)"}}>€</span></div><div className="kpi-meta">Hors brouillons</div></div>
    </div>

    <div style={{display:"grid", gridTemplateColumns:"1.4fr 1fr", gap:18}}>
      <div className="card">
        <div className="card-head"><div className="card-title">Revenus par période</div><span className="tiny muted">Mois calendaires</span></div>
        <div style={{padding:"18px 22px"}}>
          {[
            {m:"Janv. 2026", v:5300.82, n:1},
            {m:"Févr. 2026", v:1688.61, n:1},
            {m:"Mars 2026",  v:4642.25, n:4},
            {m:"Avr. 2026",  v:6318.86, n:8},
          ].map((r,i)=>{
            const max = 6500;
            const w = (r.v/max)*100;
            return <div key={i} style={{display:"grid", gridTemplateColumns:"110px 1fr 110px 50px", gap:14, alignItems:"center", padding:"10px 0", borderBottom:i<3?"1px solid var(--line-soft)":"none"}}>
              <span className="muted tiny mono">{r.m}</span>
              <div style={{height:18, background:"var(--paper-3)", position:"relative"}}>
                <div style={{position:"absolute", inset:0, width:`${w}%`, background:"var(--accent)"}}/>
              </div>
              <span className="num mono" style={{textAlign:"right"}}><Money amount={r.v}/></span>
              <span className="num muted tiny mono" style={{textAlign:"right"}}>{r.n} fact.</span>
            </div>;
          })}
        </div>
      </div>

      <div className="card">
        <div className="card-head"><div className="card-title">Vieillissement</div><span className="tiny muted">Au 28 avr. 2026</span></div>
        <div style={{padding:"14px 18px"}}>
          {[
            {l:"Courant", v:5464.32, n:8, c:"var(--ok)"},
            {l:"1–30 j", v:1854.46, n:3, c:"var(--ink-2)"},
            {l:"31–60 j", v:0, n:0, c:"var(--ink-3)"},
            {l:"61–90 j", v:0, n:0, c:"var(--warn)"},
            {l:"91+ j", v:1819.49, n:1, c:"var(--danger)"},
          ].map((b,i)=>(
            <div key={i} style={{display:"flex", justifyContent:"space-between", alignItems:"center", padding:"10px 0", borderBottom:i<4?"1px solid var(--line-soft)":"none"}}>
              <div className="row" style={{gap:10}}>
                <span style={{width:8, height:24, background:b.c}}/>
                <div>
                  <div style={{fontWeight:500, fontSize:13}}>{b.l}</div>
                  <div className="tiny muted">{b.n} factures</div>
                </div>
              </div>
              <Money amount={b.v}/>
            </div>
          ))}
        </div>
      </div>
    </div>

    <div className="section-title">Revenus par client</div>
    <div className="card">
      <table className="t">
        <thead><tr><th>Client</th><th className="num">Factures</th><th>Dernière facture</th><th className="num">Encaissé</th><th className="num">Total facturé</th><th className="num">Part</th></tr></thead>
        <tbody>{[
          {c:"Alphonsine Michel", f:1, d:"02 jan.", p:5300.82, t:5300.82},
          {c:"Odile Julien",      f:1, d:"04 avr.", p:2674.53, t:3939.32},
          {c:"Angèle Chapuis",    f:1, d:"13 mar.", p:1810.06, t:2661.90},
          {c:"Dany Vigneron",     f:2, d:"31 mar.", p:1827.61, t:1980.35},
          {c:"Lylou Magne",       f:1, d:"17 avr.", p:1688.61, t:1688.61},
          {c:"Séraphine Thibault",f:1, d:"28 avr.", p:0,       t:1524.00},
          {c:"Cyrille Molina",    f:1, d:"12 avr.", p:855.54,  t:855.54},
        ].map((r,i)=>{
          const pct = (r.t / 17950.54) * 100;
          return <tr key={i}>
            <td><div className="row"><span className="avatar">{r.c.split(" ").map(w=>w[0]).slice(0,2).join("")}</span><span style={{fontWeight:500}}>{r.c}</span></div></td>
            <td className="num muted mono">{r.f}</td>
            <td className="muted">{r.d}</td>
            <td className="num"><Money amount={r.p}/></td>
            <td className="num"><Money amount={r.t}/></td>
            <td className="num"><div className="row" style={{justifyContent:"flex-end", gap:8}}><div style={{width:60, height:6, background:"var(--paper-3)"}}><div style={{width:`${pct}%`, height:"100%", background:"var(--accent)"}}/></div><span className="mono tiny" style={{width:38, textAlign:"right"}}>{pct.toFixed(1)} %</span></div></td>
          </tr>;
        })}</tbody>
      </table>
    </div>
  </Shell>
);

window.CatalogPage = CatalogPage;
window.TaxesPage = TaxesPage;
window.AccountingPage = AccountingPage;
