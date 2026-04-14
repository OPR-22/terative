#let font-for(name) = if name == "Serif" {
  "New Computer Modern"
} else if name == "Mono" {
  "DejaVu Sans Mono"
} else {
  "DejaVu Sans"
}

#let rgb-or(default, hex) = if hex == none {
  default
} else {
  rgb(hex)
}

#let seller-block(data) = {
  let s = data.seller
  let l = data.labels
  set text(size: 10pt)
  strong(s.name)
  if s.title != none { linebreak(); s.title }
  if s.address != none { linebreak(); s.address }
  if data.toggles.show_seller_phone and s.phone != none { linebreak(); l.tel + s.phone }
  if data.toggles.show_seller_email and s.email != none { linebreak(); s.email }
  if data.toggles.show_registration_id and s.registration_id != none {
    linebreak(); l.reg + s.registration_id
  }
}

#let client-block(data) = {
  let c = data.client
  set text(size: 10pt)
  strong(c.name)
  if c.address != none { linebreak(); c.address }
  if c.email != none { linebreak(); c.email }
  if c.phone != none { linebreak(); c.phone }
}

#let items-table(data) = {
  let l = data.labels
  let rows = ()
  rows.push((strong(l.description), strong(l.quantity), strong(l.unit_price), strong(l.total)))
  for li in data.invoice.line_items {
    rows.push((li.description, li.quantity, li.unit_price, li.total))
  }
  table(
    columns: (1fr, auto, auto, auto),
    align: (left, right, right, right),
    stroke: 0.5pt + gray,
    ..rows.flatten()
  )
}

#let totals-block(data) = {
  let l = data.labels
  set text(size: 10pt)
  align(right)[
    #l.subtotal: #data.invoice.subtotal \
    #for t in data.invoice.taxes [
      #t.name (#t.percentage): #t.amount \
    ]
    #strong[#l.total: #data.invoice.total]
  ]
}

#let render(data) = {
  let l = data.labels
  set document(title: l.invoice + " " + data.invoice.number)
  set page(
    paper: "a4",
    margin: (x: 1.5cm, y: 1.5cm),
  )
  set text(font: font-for(data.font_family), size: 11pt)
  let accent = rgb-or(rgb("#111827"), data.accent_color)

  if data.watermark != none {
    place(
      center + horizon,
      dx: 0cm,
      dy: 0cm,
      rotate(-30deg, text(size: 120pt, fill: rgb(200, 200, 200, 100), data.watermark))
    )
  }

  grid(
    columns: (1fr, auto),
    gutter: 1em,
    seller-block(data),
    align(right)[
      #text(size: 22pt, fill: accent, weight: "bold")[#l.invoice] \
      \##data.invoice.number \
      #data.invoice.date
      #if data.toggles.show_due_date and data.invoice.due_date != none [
        \ #l.due #data.invoice.due_date
      ]
    ],
  )

  v(1em)
  if data.header_text != none {
    text(data.header_text)
    linebreak()
  }
  v(0.5em)

  strong(l.bill_to)
  linebreak()
  client-block(data)

  v(1em)
  items-table(data)
  v(0.5em)
  totals-block(data)

  if data.toggles.show_total_in_words {
    v(0.5em)
    text(size: 9pt, style: "italic", data.invoice.total_in_words)
  }

  if data.invoice.notes != none {
    v(1em)
    text(size: 9pt, data.invoice.notes)
  }

  if data.toggles.show_tax_id_numbers {
    let ids = data.invoice.taxes
      .filter(t => t.tax_id_number != none)
      .map(t => t.name + ": " + t.tax_id_number)
      .join(", ")
    if ids != none and ids != "" {
      v(0.5em)
      text(size: 8pt, fill: gray, ids)
    }
  }

  if data.toggles.show_signature and data.seller.signature_base64 != none {
    v(1em)
    text(l.signature)
  }

  if data.footer_text != none {
    place(bottom, text(size: 8pt, fill: gray, data.footer_text))
  }
}
