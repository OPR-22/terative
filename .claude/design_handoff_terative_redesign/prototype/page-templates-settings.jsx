// Pages: Modèles facture, Modèles e-mail, Bookmark webview, Paramètres

const TemplatesPage = () => (
  <Shell active="templates" crumbs={["Cabinet Lemaire","Modèles de facture"]}
    title="Modèles de facture"
    sub="3 modèles · le modèle par défaut est utilisé sauf indication contraire"
    actions={<button className="btn primary"><I.Plus size={13}/> Nouveau modèle</button>}>
    <div style={{display:"grid", gridTemplateColumns:"repeat(3, 1fr)", gap:16}}>
      {[
        {n:"Modèle par défaut", l:"Classique", f:"Sans-serif", color:"oklch(0.55 0.15 250)", def:true, accent:"#2563EB"},
        {n:"Suivi mensuel",     l:"Moderne",   f:"Sans-serif", color:"var(--accent)", def:false, accent:"#C26646"},
        {n:"Audit annuel",      l:"Minimal",   f:"Serif",      color:"var(--ink)", def:false, accent:"#1F1F1F"},
      ].map((t,i)=>(
        <div key={i} className="card">
          <div style={{height:200, background:"var(--paper-2)", borderBottom:"1px solid var(--line)", padding:18, display:"flex", flexDirection:"column", gap:6, fontFamily: t.f==="Serif" ? "var(--font-serif)" : "var(--font-sans)"}}>
            <div style={{display:"flex", justifyContent:"space-between"}}>
              <div style={{width:32, height:32, background:t.color, opacity:0.9}}/>
              <div style={{textAlign:"right"}}>
                <div style={{fontSize:14, fontWeight:600, letterSpacing:"0.04em"}}>FACTURE</div>
                <div className="tiny muted">#1001</div>
              </div>
            </div>
            <div style={{height:6, background:"var(--line)", width:"55%", marginTop:14}}/>
            <div style={{height:4, background:"var(--line-soft)", width:"40%"}}/>
            <div style={{height:4, background:"var(--line-soft)", width:"35%"}}/>
            <div style={{flex:1}}/>
            <div style={{height:1, background:"var(--line)"}}/>
            <div style={{display:"flex", justifyContent:"space-between"}}><div style={{height:4, background:"var(--line-soft)", width:"30%"}}/><div style={{height:4, background:t.color, width:"20%"}}/></div>
          </div>
          <div className="card-body">
            <div className="row" style={{justifyContent:"space-between", marginBottom:6}}>
              <div style={{fontWeight:500}}>{t.n}</div>
              {t.def && <Badge kind="final">Par défaut</Badge>}
            </div>
            <div className="tiny muted" style={{marginBottom:12}}>{t.l} · {t.f} · accent <span className="mono">{t.accent}</span></div>
            <div className="row" style={{gap:6, flexWrap:"wrap"}}>
              <button className="btn sm"><I.Edit size={11}/> Modifier</button>
              <button className="btn sm"><I.Copy size={11}/> Dupliquer</button>
              {!t.def && <button className="btn sm"><I.Star size={11}/> Définir par défaut</button>}
              {!t.def && <button className="btn sm danger"><I.Trash size={11}/></button>}
            </div>
          </div>
        </div>
      ))}
    </div>

    <div className="section-title">Aperçu — éditeur de modèle</div>
    <div className="card">
      <div className="card-head">
        <div><div className="card-title">Modifier — Modèle par défaut</div><div className="card-sub">Aperçu mis à jour automatiquement</div></div>
        <div className="row" style={{gap:8}}><button className="btn">Annuler</button><button className="btn primary">Enregistrer</button></div>
      </div>
      <div style={{display:"grid", gridTemplateColumns:"360px 1fr"}}>
        <div style={{padding:"16px 18px", borderRight:"1px solid var(--line)"}}>
          <div className="field"><label className="label">Nom</label><input className="input" defaultValue="Modèle par défaut"/></div>
          <div className="field"><label className="label">Mise en page</label>
            <Pills value="cl" options={[{id:"cl",label:"Classique"},{id:"mo",label:"Moderne"},{id:"mi",label:"Minimal"}]}/>
          </div>
          <div className="field"><label className="label">Police</label>
            <Pills value="sans" options={[{id:"sans",label:"Sans-serif"},{id:"serif",label:"Serif"},{id:"mono",label:"Mono"}]}/>
          </div>
          <div className="field">
            <label className="label">Logo</label>
            <div style={{border:"1px dashed var(--line)", padding:14, display:"flex", alignItems:"center", gap:12}}>
              <div style={{width:42,height:42,background:"var(--paper-3)", display:"grid", placeItems:"center", color:"var(--ink-3)"}}><I.Image size={18}/></div>
              <div style={{flex:1}}>
                <div className="tiny" style={{fontWeight:500}}>cabinet-lemaire.png</div>
                <div className="tiny muted">PNG · 240×240 · 18 KB</div>
              </div>
              <button className="btn sm icon"><I.X size={12}/></button>
            </div>
          </div>
          <div className="field"><label className="label">Couleur d'accent</label>
            <div className="row" style={{gap:8}}>
              <input className="input mono" defaultValue="#2563EB" style={{width:120}}/>
              <span style={{width:32, height:32, background:"#2563EB"}}/>
            </div>
          </div>
          <div className="field"><label className="label">En-tête</label><textarea className="textarea" rows="2" placeholder="Texte affiché en haut du PDF…"/></div>
          <div className="field"><label className="label">Pied de page</label><textarea className="textarea" rows="2" defaultValue="Merci pour votre confiance."/></div>
          <div className="label" style={{marginTop:6}}>Affichage</div>
          {["Téléphone du vendeur","E-mail du vendeur","Numéro d'enregistrement","Numéros fiscaux","Signature","Date d'échéance","Total en lettres"].map((l,i)=>(
            <label key={i} className={"checkbox " + (i<6?"on":"")} style={{padding:"6px 0", display:"flex"}}>
              <span className="box"/>{l}
            </label>
          ))}
        </div>
        <div style={{background:"var(--paper-3)", padding:24, display:"grid", placeItems:"center"}}>
          <div style={{width:380, aspectRatio:"1/1.41", background:"white", color:"#222", padding:28, fontFamily:"var(--font-sans)", fontSize:9, lineHeight:1.5, boxShadow:"0 4px 24px oklch(0 0 0 / 0.08)", position:"relative", overflow:"hidden"}}>
            <div style={{position:"absolute", top:"42%", left:"50%", transform:"translate(-50%,-50%) rotate(-18deg)", fontSize:54, color:"oklch(0 0 0 / 0.06)", letterSpacing:"0.1em", fontWeight:300}}>APERÇU</div>
            <div style={{display:"flex", justifyContent:"space-between", alignItems:"flex-start"}}>
              <div style={{width:34, height:34, background:"#2563EB"}}/>
              <div style={{textAlign:"right"}}>
                <div style={{fontSize:18, fontWeight:600, letterSpacing:"0.06em"}}>FACTURE</div>
                <div style={{fontSize:8, color:"#777", marginTop:2}}>#1001 · 28 avril 2026</div>
                <div style={{fontSize:8, color:"#777"}}>Échéance · 28 mai 2026</div>
              </div>
            </div>
            <div style={{marginTop:24, fontWeight:600}}>Facturé à</div>
            <div>Séraphine Thibault</div>
            <div style={{color:"#666"}}>123 rue Example, 1000 Bruxelles</div>
            <div style={{color:"#666"}}>contact@thibault.fr</div>
            <table style={{width:"100%", borderCollapse:"collapse", marginTop:18, fontSize:9}}>
              <thead><tr style={{borderBottom:"1px solid #222"}}><th style={{textAlign:"left", padding:"4px 0"}}>Description</th><th style={{textAlign:"right"}}>Qté</th><th style={{textAlign:"right"}}>P.U.</th><th style={{textAlign:"right"}}>Total</th></tr></thead>
              <tbody>
                <tr style={{borderBottom:"1px solid #eee"}}><td style={{padding:"4px 0"}}>Audit 471 — analyse trim.</td><td style={{textAlign:"right"}}>1</td><td style={{textAlign:"right"}}>1 200,00 €</td><td style={{textAlign:"right"}}>1 200,00 €</td></tr>
              </tbody>
            </table>
            <div style={{marginTop:14, marginLeft:"auto", width:"55%", fontSize:9}}>
              <div style={{display:"flex", justifyContent:"space-between"}}><span>Sous-total</span><span>1 200,00 €</span></div>
              <div style={{display:"flex", justifyContent:"space-between", color:"#666"}}><span>TVA 6 %</span><span>72,00 €</span></div>
              <div style={{display:"flex", justifyContent:"space-between", color:"#666"}}><span>TVH 21 %</span><span>252,00 €</span></div>
              <div style={{display:"flex", justifyContent:"space-between", borderTop:"1px solid #222", paddingTop:4, marginTop:4, fontWeight:600}}><span>Total</span><span>1 524,00 €</span></div>
            </div>
            <div style={{position:"absolute", bottom:24, left:28, right:28, fontSize:8, color:"#888", borderTop:"1px solid #ddd", paddingTop:8}}>Merci pour votre confiance · TVA: BE0123456789</div>
          </div>
        </div>
      </div>
    </div>
  </Shell>
);

const EmailsPage = () => (
  <Shell active="emails" crumbs={["Cabinet Lemaire","Modèles d'e-mail"]}
    title="Modèles d'e-mail"
    sub="Gérez les e-mails utilisés lors de l'envoi de factures"
    actions={null}>
    {[
      {sec:"Premier contact", desc:"Envoyé la première fois que vous envoyez une facture à un client.",
       items:[{n:"Default", s:"Invoice {{number}} from {{seller_name}}", b:"Hi {{client_name}}, please find invoice {{number}} attached. Total: {{total}}. — {{seller_name}}", def:true},
              {n:"Premier envoi (FR)", s:"Facture {{number}} — {{seller_name}}", b:"Bonjour {{client_name}}, vous trouverez ci-joint la facture {{number}} d'un montant de {{total}}, à régler avant le {{due_date}}.", def:false}]},
      {sec:"Relance", desc:"Envoyé comme rappel lorsque le client n'a pas répondu.",
       items:[{n:"Default reminder", s:"Reminder: invoice {{number}}", b:"Hi {{client_name}}, this is a friendly reminder regarding invoice {{number}} ({{total}}), due on {{due_date}}.", def:true},
              {n:"Relance ferme", s:"Rappel — facture {{number}} en retard", b:"Madame, Monsieur, sauf erreur de notre part, la facture {{number}} d'un montant de {{total}} reste impayée…", def:false}]},
    ].map((sec,si)=>(
      <div key={si} style={{marginBottom:28}}>
        <div style={{display:"flex", justifyContent:"space-between", alignItems:"flex-end", marginBottom:12}}>
          <div>
            <h2 style={{fontFamily:"var(--font-serif)", fontSize:22, margin:0, letterSpacing:"-0.01em"}}>{sec.sec}</h2>
            <div className="tiny muted" style={{marginTop:4}}>{sec.desc}</div>
          </div>
          <button className="btn"><I.Plus size={13}/> Ajouter</button>
        </div>
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
          {sec.items.map((it,i)=>(
            <div key={i} className="card">
              <div className="card-head">
                <div className="row" style={{gap:10}}>
                  <I.Mail size={14} style={{color:"var(--ink-3)"}}/>
                  <div>
                    <div style={{fontWeight:500, fontSize:13}}>{it.n}</div>
                    <div className="tiny muted">Sujet · <span className="mono">{it.s}</span></div>
                  </div>
                </div>
                {it.def && <Badge kind="final">Par défaut</Badge>}
              </div>
              <div className="card-body">
                <div style={{fontSize:12, color:"var(--ink-2)", lineHeight:1.55, fontFamily:"var(--font-mono)", background:"var(--paper-2)", padding:10, borderRadius:4, border:"1px solid var(--line-soft)"}}>{it.b}</div>
                <div className="row" style={{gap:6, marginTop:12}}>
                  <button className="btn sm"><I.Edit size={11}/> Modifier</button>
                  {!it.def && <button className="btn sm"><I.Star size={11}/> Définir par défaut</button>}
                  {!it.def && <button className="btn sm danger"><I.Trash size={11}/></button>}
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    ))}

    <div className="section-title">Aperçu — éditeur</div>
    <div className="card">
      <div style={{display:"grid", gridTemplateColumns:"1fr 280px"}}>
        <div style={{padding:18, borderRight:"1px solid var(--line)"}}>
          <div className="field"><label className="label">Nom</label><input className="input" defaultValue="Premier envoi (FR)"/></div>
          <div className="field"><label className="label">Sujet</label><input className="input mono" defaultValue="Facture {{number}} — {{seller_name}}"/></div>
          <div className="field"><label className="label">Corps</label>
            <textarea className="textarea" rows="8" defaultValue={"Bonjour {{client_name}},\n\nVous trouverez ci-joint la facture {{number}} d'un montant de {{total}}, à régler avant le {{due_date}}.\n\nCordialement,\n{{seller_name}}"}/>
          </div>
        </div>
        <div style={{padding:18, background:"var(--paper-2)"}}>
          <div className="label">Variables disponibles</div>
          <div style={{display:"flex", flexDirection:"column", gap:4, fontFamily:"var(--font-mono)", fontSize:11}}>
            {["{{number}}","{{client_name}}","{{date}}","{{due_date}}","{{total}}","{{subtotal}}","{{seller_name}}","{{currency_code}}"].map(v=>(
              <div key={v} className="row" style={{gap:6, padding:"4px 0"}}>
                <span style={{padding:"2px 6px", background:"var(--paper)", border:"1px solid var(--line-soft)", borderRadius:3}}>{v}</span>
              </div>
            ))}
          </div>
          <div className="tiny muted" style={{marginTop:14, lineHeight:1.5}}>Cliquez pour insérer dans le sujet ou le corps à la position du curseur.</div>
        </div>
      </div>
    </div>
  </Shell>
);

const BookmarkPage = () => (
  <Shell active="dashboard" crumbs={["Cabinet Lemaire","Favoris","Wikipedia"]}
    title={null} sub={null} actions={null}>
    {/* Replace title with browser toolbar */}
    <div style={{display:"flex", flexDirection:"column", height:"calc(100% - 0px)", marginTop:-20}}>
      <div className="card" style={{padding:0, borderRadius:0, borderLeft:0, borderRight:0, marginLeft:-28, marginRight:-28, marginTop:-24}}>
        <div className="row" style={{padding:"10px 24px", gap:8, borderBottom:"1px solid var(--line)"}}>
          <button className="btn icon"><I.ArrowLeft size={14}/></button>
          <button className="btn icon"><svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M5 12h14M12 5l7 7-7 7"/></svg></button>
          <button className="btn icon"><I.Refresh size={14}/></button>
          <button className="btn icon"><I.Home size={14}/></button>
          <div className="search-mini" style={{flex:1, minWidth:0}}>
            <I.Lock size={11} style={{color:"var(--ok)"}}/>
            <span style={{color:"var(--ink)"}}>en.wikipedia.org</span>
            <span className="muted">/wiki/Double-entry_bookkeeping</span>
          </div>
          <button className="btn icon"><I.Star size={14}/></button>
        </div>
      </div>
      <div style={{flex:1, padding:"40px 60px", background:"white", color:"#222", overflow:"auto", marginLeft:-28, marginRight:-28, marginBottom:-32, fontFamily:"Georgia, serif"}}>
        <div style={{maxWidth:720, margin:"0 auto"}}>
          <div style={{fontSize:12, color:"#666", borderBottom:"1px solid #eee", paddingBottom:6}}>From a public reference site — embedded webview</div>
          <h1 style={{fontFamily:"var(--font-serif)", fontSize:36, marginTop:24, marginBottom:8, color:"#111"}}>Double-entry bookkeeping</h1>
          <div style={{fontSize:13, color:"#666", fontStyle:"italic", marginBottom:20}}>An overview article opened from your sidebar bookmarks.</div>
          <p style={{fontSize:14, lineHeight:1.7, color:"#222"}}>This is a placeholder for the embedded webview content. Bookmarks open in a native webview hosted inside Terative; you keep your invoicing context (sidebar, status bar) without leaving the app.</p>
          <p style={{fontSize:14, lineHeight:1.7, color:"#222"}}>Use the toolbar above to navigate. Bookmarks are managed under Paramètres → Favoris.</p>
          <div style={{height:200, background:"#f3f1ec", border:"1px solid #ddd", marginTop:20, display:"grid", placeItems:"center", color:"#888", fontFamily:"var(--font-mono)", fontSize:11}}>{"<webview-content>"}</div>
        </div>
      </div>
    </div>
  </Shell>
);

const SettingsPage = () => {
  const Section = ({title, sub, status="saved", children}) => (
    <div style={{display:"grid", gridTemplateColumns:"260px 1fr", gap:32, padding:"28px 0", borderTop:"1px solid var(--line)"}}>
      <div>
        <h2 style={{fontFamily:"var(--font-serif)", fontSize:22, margin:0, letterSpacing:"-0.01em"}}>{title}</h2>
        <div className="tiny muted" style={{marginTop:6, lineHeight:1.55}}>{sub}</div>
        <div className="row tiny muted" style={{marginTop:14, gap:6}}>
          <span className={"dot-status " + (status==="saved"?"ok":status==="saving"?"warn":"idle")}/>
          {status==="saved" ? "Enregistré il y a 1 min" : status==="saving" ? "Enregistrement…" : "Non modifié"}
        </div>
      </div>
      <div>{children}</div>
    </div>
  );

  return <Shell active="settings" crumbs={["Cabinet Lemaire","Paramètres"]}
    title="Paramètres" sub="Profil, préférences et sauvegardes du cabinet" actions={null}>
    <Section title="Profil vendeur" sub="Apparaît sur toutes les factures émises et dans la signature des e-mails.">
      <div className="card"><div className="card-body">
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
          <div className="field"><label className="label">Nom / raison sociale</label><input className="input" defaultValue="Cabinet Lemaire SCS"/></div>
          <div className="field"><label className="label">Titre</label><input className="input" defaultValue="Comptabilité & conseil"/></div>
          <div className="field"><label className="label">N° d'enregistrement</label><input className="input mono" defaultValue="BE 0123 456 789"/></div>
          <div className="field"><label className="label">E-mail</label><input className="input" defaultValue="contact@cabinet-lemaire.fr"/></div>
          <div className="field"><label className="label">Téléphone</label><input className="input mono" defaultValue="+33 1 23 45 67 89"/></div>
          <div className="field"><label className="label">Adresse</label><input className="input" defaultValue="14 rue Beaumarchais, 75011 Paris"/></div>
        </div>
        <div className="field"><label className="label">Signature</label>
          <div style={{border:"1px dashed var(--line)", padding:14, display:"flex", alignItems:"center", gap:12}}>
            <div style={{width:80, height:34, background:"var(--paper-3)", display:"grid", placeItems:"center", color:"var(--ink-3)", fontFamily:"var(--font-serif)", fontStyle:"italic", fontSize:14}}>C. Lemaire</div>
            <div className="tiny muted">PNG, JPG ou JPEG · max 2 Mo</div>
            <button className="btn sm" style={{marginLeft:"auto"}}><I.Upload size={11}/> Téléverser</button>
          </div>
        </div>
        <div className="row" style={{justifyContent:"flex-end"}}><button className="btn primary">Enregistrer</button></div>
      </div></div>
    </Section>

    <Section title="Devise" sub="Devise par défaut pour les nouvelles factures et l'affichage des soldes.">
      <div className="card"><div className="card-body" style={{maxWidth:420}}>
        <div className="field"><label className="label">Devise</label>
          <div className="select row" style={{justifyContent:"space-between"}}>EUR — Euro<I.ChevDown size={13}/></div>
          <div className="field-help">Exemple : <span className="mono">1 234,56 €</span></div>
        </div>
        <button className="btn primary">Enregistrer</button>
      </div></div>
    </Section>

    <Section title="E-mail (SMTP)" sub="Sortant. Testez la connexion avant d'envoyer aux clients.">
      <div className="card"><div className="card-body">
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
          <div className="field"><label className="label">Serveur SMTP</label><input className="input mono" defaultValue="smtp.cabinet-lemaire.fr"/></div>
          <div className="field"><label className="label">Port</label><input className="input mono" defaultValue="587"/></div>
          <div className="field"><label className="label">Adresse expéditeur</label><input className="input" defaultValue="contact@cabinet-lemaire.fr"/></div>
          <div className="field"><label className="label">Utilisateur</label><input className="input mono" defaultValue="contact@cabinet-lemaire.fr"/></div>
          <div className="field"><label className="label">Mot de passe</label><input className="input" type="password" defaultValue="••••••••••"/></div>
          <div className="field" style={{display:"flex", alignItems:"flex-end", paddingBottom:8}}>
            <span className="row" style={{gap:10}}><span className="toggle on"/> Utiliser TLS</span>
          </div>
        </div>
        <div className="row" style={{justifyContent:"space-between", marginTop:6}}>
          <div className="row tiny" style={{color:"var(--ok)", gap:6}}><I.Check size={12}/> Test réussi · 142 ms</div>
          <div className="row" style={{gap:8}}><button className="btn">Tester</button><button className="btn primary">Enregistrer</button></div>
        </div>
      </div></div>
    </Section>

    <Section title="Favoris" sub="Raccourcis web visibles dans la barre latérale, ouverts dans un webview natif.">
      <div className="card"><div className="card-body">
        <div className="row" style={{justifyContent:"space-between", marginBottom:14}}>
          <span className="row" style={{gap:10}}><span className="toggle on"/> Activer la barre des favoris</span>
          <span className="row" style={{gap:10}}><span className="toggle"/> Sauvegarde auto à la fermeture</span>
        </div>
        <div style={{border:"1px solid var(--line)", borderRadius:6}}>
          {[
            {l:"Wikipedia", u:"https://en.wikipedia.org"},
            {l:"Google",    u:"https://google.com"},
            {l:"GitHub",    u:"https://github.com"},
          ].map((b,i,arr)=>(
            <div key={i} style={{display:"grid", gridTemplateColumns:"24px 180px 1fr 80px", padding:"10px 12px", alignItems:"center", borderBottom:i<arr.length-1?"1px solid var(--line-soft)":"none", gap:10}}>
              <I.Drag size={14} style={{color:"var(--ink-4)"}}/>
              <input className="input" defaultValue={b.l} style={{padding:"4px 8px"}}/>
              <input className="input mono" defaultValue={b.u} style={{padding:"4px 8px"}}/>
              <div className="row" style={{gap:4, justifyContent:"flex-end"}}>
                <button className="btn sm icon"><I.Edit size={11}/></button>
                <button className="btn sm icon danger"><I.Trash size={11}/></button>
              </div>
            </div>
          ))}
        </div>
        <button className="btn ghost sm" style={{marginTop:8}}><I.Plus size={11}/> Ajouter un favori</button>
      </div></div>
    </Section>

    <Section title="Sections du carnet" sub="Sections globales partagées par tous les clients (ordre éditable).">
      <div className="card"><div className="card-body">
        <div style={{border:"1px solid var(--line)", borderRadius:6}}>
          {[
            {n:"Antécédents", c:42},
            {n:"Traitement en cours", c:38},
            {n:"Préférences administratives", c:21},
            {n:"Notes diverses", c:14},
          ].map((s,i,arr)=>(
            <div key={i} style={{display:"grid", gridTemplateColumns:"24px 1fr 100px 100px", padding:"10px 12px", alignItems:"center", borderBottom:i<arr.length-1?"1px solid var(--line-soft)":"none", gap:10}}>
              <I.Drag size={14} style={{color:"var(--ink-4)"}}/>
              <input className="input" defaultValue={s.n} style={{padding:"4px 8px", border:"1px solid transparent"}}/>
              <span className="tiny muted mono" style={{textAlign:"right"}}>{s.c} entrées</span>
              <div className="row" style={{justifyContent:"flex-end", gap:4}}>
                <button className="btn sm icon danger"><I.Trash size={11}/></button>
              </div>
            </div>
          ))}
        </div>
        <button className="btn ghost sm" style={{marginTop:8}}><I.Plus size={11}/> Ajouter une section</button>
      </div></div>
    </Section>

    <Section title="Préférences" sub="Apparence, langue et options par défaut pour les nouvelles factures.">
      <div className="card"><div className="card-body">
        <div style={{display:"grid", gridTemplateColumns:"1fr 1fr", gap:14}}>
          <div className="field"><label className="label">Thème</label>
            <Pills value="lt" options={[{id:"lt",label:"Clair"},{id:"dk",label:"Sombre"},{id:"sys",label:"Système"}]}/>
          </div>
          <div className="field"><label className="label">Langue</label>
            <Pills value="fr" options={[{id:"fr",label:"Français"},{id:"en",label:"English"}]}/>
          </div>
          <div className="field"><label className="label">Dossier d'export PDF</label><div className="input mono">~/Documents/Terative/Factures</div></div>
          <div className="field"><label className="label">Dossier de sauvegarde</label><div className="input mono">~/Documents/Terative/Backups</div></div>
          <div className="field"><label className="label">Échéance par défaut</label>
            <div className="row" style={{gap:8}}><input className="input mono" defaultValue="30" style={{width:80}}/><span className="muted">jours (0–365)</span></div>
          </div>
        </div>
      </div></div>
    </Section>

    <Section title="Données" sub="Sauvegardes manuelles et automatiques. Restaurer écrasera les données actuelles.">
      <div className="card">
        <div className="card-body">
          <div className="row" style={{gap:8, marginBottom:14}}>
            <button className="btn primary"><I.Download size={13}/> Sauvegarder maintenant</button>
            <button className="btn"><I.Upload size={13}/> Restaurer depuis un fichier</button>
          </div>
        </div>
        <table className="t">
          <thead><tr><th>Date</th><th>Type</th><th>Portée</th><th className="num">Taille</th><th></th></tr></thead>
          <tbody>{[
            {d:"28 avr. 2026, 03:00", t:"Automatique", p:"Système", s:"4,2 Mo"},
            {d:"27 avr. 2026, 18:14", t:"Manuelle",    p:"Utilisateur", s:"4,1 Mo"},
            {d:"26 avr. 2026, 03:00", t:"Automatique", p:"Système", s:"4,1 Mo"},
            {d:"22 avr. 2026, 09:32", t:"Manuelle",    p:"Utilisateur", s:"3,9 Mo"},
          ].map((b,i)=>(
            <tr key={i}>
              <td className="muted mono">{b.d}</td>
              <td><Badge kind={b.t==="Manuelle"?"final":"outline"}>{b.t}</Badge></td>
              <td className="muted">{b.p}</td>
              <td className="num mono">{b.s}</td>
              <td className="actions"><div className="row" style={{justifyContent:"flex-end", gap:4}}>
                <button className="btn sm">Restaurer</button>
                {b.p==="Utilisateur" && <button className="btn sm icon danger"><I.Trash size={11}/></button>}
              </div></td>
            </tr>
          ))}</tbody>
        </table>
      </div>
    </Section>

    <Section title="Développeur" sub="Disponible uniquement en build de débogage." status="idle">
      <div className="card"><div className="card-body">
        <div className="label">Initialiser la base avec des données de test</div>
        <div style={{display:"grid", gridTemplateColumns:"repeat(3, 1fr)", gap:14, marginTop:8}}>
          {[["Clients","20"],["Articles","15"],["Taxes","2"],["Factures","30"],["Favoris","3"],["Entrées journal / client","5"]].map(([l,v],i)=>(
            <div key={i} className="field"><label className="label">{l}</label><input className="input mono" defaultValue={v}/></div>
          ))}
        </div>
        <div className="row" style={{justifyContent:"flex-end"}}><button className="btn"><I.Database size={13}/> Lancer le seed</button></div>
      </div></div>
    </Section>
  </Shell>;
};

window.TemplatesPage = TemplatesPage;
window.EmailsPage = EmailsPage;
window.BookmarkPage = BookmarkPage;
window.SettingsPage = SettingsPage;
