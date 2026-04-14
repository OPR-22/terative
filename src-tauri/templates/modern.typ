// Modern layout: colored accent band at the top, minimal rules, right-aligned
// "big number" total. The table uses zebra striping instead of full grid lines.

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

#let seller-block(data, fill-color) = {
  let s = data.seller
  let l = data.labels
  set text(size: 10pt, fill: fill-color)
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

#let items-table(data, accent) = {
  let l = data.labels
  let header = (
    strong(l.description),
    strong(l.quantity),
    strong(l.unit_price),
    strong(l.total),
  )
  let body-rows = data.invoice.line_items.map(
    li => (li.description, li.quantity, li.unit_price, li.total),
  )
  table(
    columns: (1fr, auto, auto, auto),
    align: (left, right, right, right),
    stroke: none,
    fill: (_, row) => if row == 0 {
      accent.lighten(85%)
    } else if calc.odd(row) {
      rgb("#fafafa")
    } else {
      none
    },
    inset: (x: 8pt, y: 6pt),
    ..header,
    ..body-rows.flatten(),
  )
}

#let render(data) = {
  let l = data.labels
  let accent = rgb-or(rgb("#2563eb"), data.accent_color)
  set document(title: l.invoice + " " + data.invoice.number)
  set page(
    paper: "a4",
    margin: (x: 1.5cm, top: 0cm, bottom: 1.5cm),
  )
  set text(font: font-for(data.font_family), size: 11pt)

  if data.watermark != none {
    place(
      center + horizon,
      rotate(-30deg, text(size: 120pt, fill: rgb(200, 200, 200, 80), data.watermark)),
    )
  }

  // Colored band spanning full page width.
  block(
    width: 100%,
    fill: accent,
    inset: (x: 1.5cm, y: 1.2cm),
    [
      #grid(
        columns: (1fr, auto),
        seller-block(data, white),
        align(right)[
          #text(size: 28pt, fill: white, weight: "bold")[#l.invoice] \
          #text(size: 12pt, fill: white.transparentize(20%))[\##data.invoice.number]
        ],
      )
    ],
  )

  v(1.2em)

  grid(
    columns: (1fr, auto),
    gutter: 2em,
    [
      #text(size: 9pt, fill: gray, upper(l.bill_to)) \
      #client-block(data)
    ],
    align(right)[
      #text(size: 9pt, fill: gray)[#upper("Date")] \
      #data.invoice.date
      #if data.toggles.show_due_date and data.invoice.due_date != none [
        \ \ #text(size: 9pt, fill: gray)[#upper(l.due)] \
        #data.invoice.due_date
      ]
    ],
  )

  v(1.5em)

  if data.header_text != none {
    text(data.header_text)
    v(0.5em)
  }

  items-table(data, accent)

  v(1em)
  align(right)[
    #text(size: 10pt, fill: gray)[#l.subtotal] \
    #text(size: 12pt)[#data.invoice.subtotal] \
    #for t in data.invoice.taxes [
      #text(size: 10pt, fill: gray)[#t.name (#t.percentage)] \
      #text(size: 12pt)[#t.amount] \
    ]
    #v(0.3em)
    #rect(fill: accent, inset: (x: 12pt, y: 8pt), radius: 4pt, [
      #text(size: 10pt, fill: white)[#upper(l.total)] #h(1em) #text(size: 14pt, fill: white, weight: "bold")[#data.invoice.total]
    ])
  ]

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
