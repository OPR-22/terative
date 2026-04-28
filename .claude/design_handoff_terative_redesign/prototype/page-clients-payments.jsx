// Pages: Payments, Clients (list + detail with 3 tabs)

const PAYS = [
  {d:"2026-04-27", c:"Alphonsine Michel", m:"Espèces",       r:"—",          a:5300.82, al:5300.82},
  {d:"2026-04-25", c:"Dany Vigneron",     m:"Chèque",        r:"CHQ-22481",  a:1506.87, al:1430.00},
  {d:"2026-04-24", c:"Alphonsine Michel", m:"Espèces",       r:"—",          a:2445.45, al:2445.45},
  {d:"2026-04-24", c:"Cyrille Molina",    m:"Carte",         r:"…4421",      a:855.54,  al:855.54},
  {d:"2026-04-17", c:"Lylou Magne",       m:"Chèque",        r:"CHQ-19042",  a:1688.61, al:1688.61},
  {d:"2026-04-15", c:"Alma Jacquot",      m:"Carte",         r:"…0091",      a:5052.81, al:5052.81},
  {d:"2026-04-15", c:"Odile Julien",      m:"Carte",         r:"…7710",      a:2674.53, al:2674.53},
  {d:"2026-04-13", c:"Angèle Chapuis",    m:"Carte",         r:"…1208",      a:2235.98, al:1810.06},
  {d:"2026-04-02", c:"Alphonsine Michel", m:"Espèces",       r:"—",          a:3052.03, al:3052.03},
  {d:"2026-03-31", c:"Dany Vigneron",     m:"Chèque",        r:"CHQ-19844",  a:397.61,  al:397.61},
  {d:"2026-03-31", c:"Alma Jacquot",      m:"Espèces",       r:"—",          a:144.48,  al:144.48},
];

const PaymentsPage = () => (
  <Shell active="payments" crumbs={["Cabinet Lemaire","Paiements"]}
    title="Paiements" sub="11 paiements ce mois · 25 354,73 € encaissés"
    actions={<>
      <button className="btn"><I.Download size={13}/> Exporter</button>
      <button className="btn primary"><I.Plus size={13}/> Nouveau paiement</button>
    </>}>
    <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:14, gap:12}}>
      <Pills value="all" options={[{id:"all",label:"Tous",count:11},{id:"vir",label:"Virement",count:0},{id:"esp",label:"Espèces",count:4},{id:"chq",label:"Chèque",count:3},{id:"crd",label:"Carte",count:4}]}/>
      <div className="row" style={{gap:8}}>
        <div className="search-mini" style={{minWidth:240}}><I.Search size={13}/><span>Client, référence…</span></div>
        <button className="btn"><I.Calendar size={13}/> Avril 2026</button>
      </div>
    </div>
    <div className="card">
      <table className="t">
        <thead><tr><th>Date</th><th>Client</th><th>Méthode</th><th>Référence</th><th className="num">Montant</th><th className="num">Alloué</th><th className="num">Reste</th><th className="num">Factures</th><th></th></tr></thead>
        <tbody>{PAYS.map((p,i)=>{
          const rest = p.a - p.al;
          return (<tr key={i}>
            <td className="muted mono">{p.d}</td>
            <td><div className="row"><span className="avatar">{p.c.split(" ").map(w=>w[0]).slice(0,2).join("")}</span>{p.c}</div></td>
            <td><Badge kind="outline">{p.m}</Badge></td>
            <td className="muted mono">{p.r}</td>
            <td className="num"><Money amount={p.a}/></td>
            <td className="num"><Money amount={p.al}/></td>
            <td className="num">{rest > 0.01 ? <span style={{color:"var(--warn)"}}><Money amount={rest}/></span> : <span className="muted">—</span>}</td>
            <td className="num muted mono">{rest > 0.01 ? "1" : "1"}</td>
            <td className="actions"><div className="row" style={{justifyContent:"flex-end", gap:4}}>
              <button className="btn sm"><I.Edit size={11}/></button>
              <button className="btn sm danger"><I.Trash size={11}/></button>
            </div></td>
          </tr>);
        })}</tbody>
      </table>
    </div>

    {/* Allocations editor preview */}
    <div className="section-title">Aperçu — éditeur de paiement</div>
    <div className="card" style={{maxWidth:720}}>
      <div className="card-head"><div className="card-title">Allocation aux factures</div><div className="tiny muted">Client : Dany Vigneron</div></div>
      <div className="card-body">
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr 1fr", gap:14, marginBottom:14}}>
          <div className="field"><label className="label">Date</label><div className="input mono">25 avr. 2026</div></div>
          <div className="field"><label className="label">Méthode</label><div className="select" style={{display:"flex",justifyContent:"space-between"}}>Chèque<I.ChevDown size={13}/></div></div>
          <div className="field"><label className="label">Montant</label><div className="input mono" style={{textAlign:"right"}}>1 506,87 €</div></div>
        </div>
        <div className="label" style={{marginBottom:8}}>Factures à allouer</div>
        <div style={{border:"1px solid var(--line)", borderRadius:6}}>
          <div style={{display:"grid", gridTemplateColumns:"24px 60px 1fr 100px 140px", padding:"8px 12px", fontSize:11, color:"var(--ink-3)", textTransform:"uppercase", letterSpacing:"0.08em", borderBottom:"1px solid var(--line)", background:"var(--paper-2)"}}>
            <span></span><span>N°</span><span>Date / échéance</span><span style={{textAlign:"right"}}>Reste dû</span><span style={{textAlign:"right"}}>À allouer</span>
          </div>
          {[
            {n:7, d:"13 mars · échéance 13 avr.", due:1506.87, alloc:1430.00, on:true},
            {n:9, d:"22 mars · échéance 22 avr.", due:480.00, alloc:76.87, on:true},
            {n:13, d:"31 mars · échéance 30 avr.", due:1244.00, alloc:0, on:false},
          ].map((f,i)=>(
            <div key={i} style={{display:"grid", gridTemplateColumns:"24px 60px 1fr 100px 140px", padding:"10px 12px", borderBottom:i<2?"1px solid var(--line-soft)":"none", alignItems:"center", fontSize:13}}>
              <span className={"checkbox " + (f.on?"on":"")}><span className="box"/></span>
              <span className="mono">#{f.n}</span>
              <span className="muted tiny">{f.d}</span>
              <span className="num" style={{textAlign:"right"}}><Money amount={f.due}/></span>
              <span className="num" style={{textAlign:"right"}}><input className="input mono" defaultValue={f.alloc.toFixed(2)} style={{textAlign:"right", padding:"4px 8px"}}/></span>
            </div>
          ))}
        </div>
        <div style={{display:"flex", justifyContent:"space-between", marginTop:14, fontSize:13}}>
          <span className="muted">Alloué <span className="mono" style={{color:"var(--ink)"}}>1 506,87 €</span> · Reste <span className="mono" style={{color:"var(--ok)"}}>0,00 €</span></span>
          <div className="row" style={{gap:8}}>
            <button className="btn">Annuler</button>
            <button className="btn primary">Enregistrer</button>
          </div>
        </div>
      </div>
    </div>
  </Shell>
);

const CLIENTS = [
  {n:"Alban Lemaire", e:"josh_dolorem@yahoo.com", p:"05 44 20 29 27", a:true, l:"FR"},
  {n:"Alma Jacquot", e:"carole_quisquam@yahoo.com", p:"05 70 31 89 05", a:true, l:"FR"},
  {n:"Alphonsine Michel", e:"pietro_iste@yahoo.com", p:"09 69 46 52 29", a:true, l:"FR"},
  {n:"Angèle Chapuis", e:"garrett_earum@gmail.com", p:"09 42 92 40 47", a:true, l:"FR"},
  {n:"Benjamin Vannier", e:"seamus_ut@gmail.com", p:"05 93 38 52 78", a:true, l:"NL"},
  {n:"Cyrielle Tardy", e:"adah_et@hotmail.com", p:"08 71 75 30 61", a:true, l:"FR"},
  {n:"Cyrille Molina", e:"kayden_molestias@gmail.com", p:"02 29 40 90 19", a:true, l:"FR"},
  {n:"Dany Vigneron", e:"santiago_explicabo@hotmail.com", p:"01 99 48 78 45", a:true, l:"EN"},
  {n:"Elena Ferreira", e:"neque_waylon@gmail.com", p:"08 57 09 70 76", a:true, l:"FR"},
  {n:"Geoffrey Masse", e:"gillian_dicta@yahoo.com", p:"08 35 22 07 58", a:true, l:"FR"},
  {n:"Ida Marty", e:"heidi_non@gmail.com", p:"02 60 53 91 60", a:true, l:"FR"},
  {n:"Ignace Grenier", e:"josefa_omnis@yahoo.com", p:"05 20 21 37 32", a:true, l:"FR"},
  {n:"Laurent Cornet", e:"alycia_blanditiis@gmail.com", p:"01 49 16 31 91", a:false, l:"FR"},
  {n:"Lylou Magne", e:"heber_enim@gmail.com", p:"05 95 37 93 16", a:true, l:"DE"},
];

const ClientsPage = () => (
  <Shell active="clients" crumbs={["Cabinet Lemaire","Clients"]}
    title="Clients" sub="32 clients actifs · 4 archivés"
    actions={<>
      <button className="btn"><I.Upload size={13}/> Importer</button>
      <button className="btn primary"><I.Plus size={13}/> Nouveau client</button>
    </>}>
    <div style={{display:"flex", justifyContent:"space-between", alignItems:"center", marginBottom:14, gap:12}}>
      <div className="row" style={{gap:8}}>
        <div className="search-mini" style={{minWidth:280}}><I.Search size={13}/><span>Nom, e-mail, téléphone, profession…</span></div>
        <span className="checkbox"><span className="box"/> Inclure les archivés</span>
      </div>
      <div className="row tiny muted">A–Z · 1–14 sur 36</div>
    </div>
    <div className="card">
      <table className="t">
        <thead><tr><th style={{width:32}}></th><th>Nom</th><th>E-mail</th><th>Téléphone</th><th>Langue</th><th className="num">Factures</th><th className="num">Solde</th><th></th></tr></thead>
        <tbody>{CLIENTS.map((c,i)=>(
          <tr key={i}>
            <td><span className="avatar">{c.n.split(" ").map(w=>w[0]).slice(0,2).join("")}</span></td>
            <td><div><div style={{fontWeight:500}}>{c.n}</div>{!c.a && <span className="badge draft tiny">Archivé</span>}</div></td>
            <td className="muted">{c.e}</td>
            <td className="muted mono">{c.p}</td>
            <td><Badge kind="outline">{c.l}</Badge></td>
            <td className="num muted mono">{Math.floor(Math.random()*8)+1}</td>
            <td className="num"><Money amount={(Math.random()*3000)|0}/></td>
            <td className="actions"><div className="row" style={{justifyContent:"flex-end", gap:4}}>
              <button className="btn sm"><I.Eye size={11}/> Ouvrir</button>
              <button className="btn sm icon"><I.Archive size={12}/></button>
            </div></td>
          </tr>
        ))}</tbody>
      </table>
    </div>
  </Shell>
);

const ClientDetailPage = () => (
  <Shell active="clients" crumbs={["Cabinet Lemaire","Clients","Alban Lemaire"]}
    title="Alban Lemaire"
    sub={<><Badge kind="outline">FR</Badge> &nbsp;<span className="muted">40 ans · Médecin · Client depuis avril 2024</span></>}
    actions={<>
      <button className="btn"><I.ArrowLeft size={13}/> Retour</button>
      <button className="btn"><I.Plus size={13}/> Paiement</button>
      <button className="btn primary"><I.Plus size={13}/> Nouvelle facture</button>
    </>}>
    {/* Mini KPI strip */}
    <div style={{display:"grid", gridTemplateColumns:"repeat(4, 1fr)", gap:12, marginBottom:18}}>
      <div className="card flat" style={{padding:"12px 14px"}}>
        <div className="kpi-label">Total facturé</div>
        <div className="kpi-value mono" style={{marginTop:6}}>4 850,00 €</div>
      </div>
      <div className="card flat" style={{padding:"12px 14px"}}>
        <div className="kpi-label">Encaissé</div>
        <div className="kpi-value mono" style={{marginTop:6, color:"var(--ok)"}}>4 850,00 €</div>
      </div>
      <div className="card flat" style={{padding:"12px 14px"}}>
        <div className="kpi-label">Solde dû</div>
        <div className="kpi-value mono" style={{marginTop:6}}>0,00 €</div>
      </div>
      <div className="card flat" style={{padding:"12px 14px"}}>
        <div className="kpi-label">Factures</div>
        <div className="kpi-value mono" style={{marginTop:6}}>5 <span className="tiny muted">dont 1 brouillon</span></div>
      </div>
    </div>

    <div className="tabs" style={{marginBottom:18}}>
      <span className="tab active">Infos</span>
      <span className="tab">Carnet</span>
      <span className="tab">Journal <span className="badge" style={{marginLeft:6}}>12</span></span>
      <span className="tab">Factures <span className="badge" style={{marginLeft:6}}>5</span></span>
      <span className="tab">Paiements</span>
    </div>

    <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:18}}>
      <div className="card">
        <div className="card-head"><div className="card-title">Identité</div><span className="tiny muted row" style={{gap:6}}><span className="dot-status ok"/> Tout est à jour</span></div>
        <div className="card-body">
          <div className="field"><label className="label">Nom</label><input className="input" defaultValue="Alban Lemaire"/></div>
          <div className="field">
            <label className="label">Adresses e-mail</label>
            <div style={{border:"1px solid var(--line)", borderRadius:6}}>
              <div style={{display:"grid", gridTemplateColumns:"auto 110px 1fr 24px", padding:"8px 10px", alignItems:"center", gap:10, borderBottom:"1px solid var(--line-soft)"}}>
                <span className="checkbox on" title="Par défaut"><span className="box" style={{borderRadius:"50%"}}/></span>
                <input className="input" defaultValue="Personnel" style={{padding:"4px 8px"}}/>
                <input className="input mono" defaultValue="josh_dolorem@yahoo.com" style={{padding:"4px 8px"}}/>
                <I.X size={13} style={{color:"var(--ink-3)"}}/>
              </div>
              <button className="btn ghost sm" style={{margin:6}}><I.Plus size={11}/> Ajouter un e-mail</button>
            </div>
          </div>
          <div className="field">
            <label className="label">Téléphones</label>
            <div style={{border:"1px solid var(--line)", borderRadius:6}}>
              <div style={{display:"grid", gridTemplateColumns:"auto 110px 1fr 24px", padding:"8px 10px", alignItems:"center", gap:10, borderBottom:"1px solid var(--line-soft)"}}>
                <span className="checkbox on"><span className="box" style={{borderRadius:"50%"}}/></span>
                <input className="input" defaultValue="Mobile" style={{padding:"4px 8px"}}/>
                <input className="input mono" defaultValue="05 44 20 29 27" style={{padding:"4px 8px"}}/>
                <I.X size={13} style={{color:"var(--ink-3)"}}/>
              </div>
              <button className="btn ghost sm" style={{margin:6}}><I.Plus size={11}/> Ajouter un téléphone</button>
            </div>
          </div>
          <div className="field"><label className="label">Adresse</label><input className="input" defaultValue="177 Jérémy Lakes, 469 Pétronille, France"/></div>
          <div className="field"><label className="label">Notes</label><textarea className="textarea" rows="2" placeholder="Notes internes (non visibles par le client)…"></textarea></div>
        </div>
      </div>

      <div style={{display:"flex", flexDirection:"column", gap:18}}>
        <div className="card">
          <div className="card-head"><div className="card-title">Démographie</div></div>
          <div className="card-body" style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
            <div className="field"><label className="label">Date de naissance</label><div className="input mono">12 / 04 / 1986</div><div className="field-help">40 ans</div></div>
            <div className="field"><label className="label">Sexe</label><div className="select" style={{display:"flex", justifyContent:"space-between"}}>Femme<I.ChevDown size={13}/></div></div>
            <div className="field"><label className="label">Genre</label><input className="input" defaultValue="non-binaire"/></div>
            <div className="field"><label className="label">Pronoms</label><input className="input" defaultValue="he/him"/></div>
            <div className="field"><label className="label">Profession</label><input className="input" defaultValue="Médecin"/></div>
            <div className="field"><label className="label">Langue</label><div className="select" style={{display:"flex", justifyContent:"space-between"}}>Français<I.ChevDown size={13}/></div></div>
            <div className="field" style={{gridColumn:"1 / -1"}}><label className="label">Recommandé par</label><div className="select" style={{display:"flex", justifyContent:"space-between"}}>Cyrille Molina<I.ChevDown size={13}/></div></div>
          </div>
        </div>

        <div className="card">
          <div className="card-head">
            <div className="card-title">Journal — extrait</div>
            <button className="btn sm"><I.Plus size={11}/> Entrée</button>
          </div>
          <div className="card-body" style={{padding:"6px 18px 14px"}}>
            {[
              {d:"24 avril", t:"Bilan trimestriel — RAS, prochaine consultation en juin."},
              {d:"02 février", t:"Dossier réglé en espèces, demande facture nominative."},
            ].map((j,i)=>(
              <div key={i} style={{padding:"10px 0", borderBottom:i<1?"1px solid var(--line-soft)":"none"}}>
                <div className="row" style={{justifyContent:"space-between"}}>
                  <span className="tiny muted mono">{j.d} 2026</span>
                  <button className="btn ghost sm"><I.Edit size={11}/></button>
                </div>
                <div style={{marginTop:4, fontSize:13}}>{j.t}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  </Shell>
);

window.PaymentsPage = PaymentsPage;
window.ClientsPage = ClientsPage;
window.ClientDetailPage = ClientDetailPage;
