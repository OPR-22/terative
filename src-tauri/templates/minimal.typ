// Minimal layout: no colored banner, hairline rules only, compact typography.
// Designed to print cleanly in monochrome.

#let font-for(name) = if name == "Serif" {
  "New Computer Modern"
} else if name == "Mono" {
  "DejaVu Sans Mono"
} else {
  "DejaVu Sans"
}

#let seller-line(data) = {
  let s = data.seller
  let l = data.labels
  let parts = (s.name,)
  if s.address != none { parts.push(s.address) }
  if data.toggles.show_seller_phone and s.phone != none { parts.push(s.phone) }
  if data.toggles.show_seller_email and s.email != none { parts.push(s.email) }
  if data.toggles.show_registration_id and s.registration_id != none {
    parts.push(l.reg + s.registration_id)
  }
  text(size: 9pt, fill: gray, parts.join("  ·  "))
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
  let header = (
    text(size: 8pt, fill: gray, upper(l.description)),
    text(size: 8pt, fill: gray, upper(l.quantity)),
    text(size: 8pt, fill: gray, upper(l.unit_price)),
    text(size: 8pt, fill: gray, upper(l.total)),
  )
  let body-rows = data.invoice.line_items.map(
    li => (li.description, li.quantity, li.unit_price, li.total),
  )
  table(
    columns: (1fr, auto, auto, auto),
    align: (left, right, right, right),
    stroke: (_, row) => if row == 0 {
      (bottom: 0.5pt + black)
    } else {
      (bottom: 0.25pt + gray)
    },
    inset: (x: 4pt, y: 6pt),
    ..header,
    ..body-rows.flatten(),
  )
}

#let render(data) = {
  let l = data.labels
  set document(title: l.invoice + " " + data.invoice.number)
  set page(
    paper: "a4",
    margin: (x: 2cm, y: 2cm),
  )
  set text(font: font-for(data.font_family), size: 10pt)

  if data.watermark != none {
    place(
      center + horizon,
      rotate(-30deg, text(size: 120pt, fill: rgb(220, 220, 220, 80), data.watermark)),
    )
  }

  // Header: invoice word above a hairline rule.
  text(size: 10pt, tracking: 3pt, upper(l.invoice))
  h(1fr)
  text(size: 10pt)[\##data.invoice.number]
  v(0.2em)
  line(length: 100%, stroke: 0.5pt + black)

  v(1em)
  seller-line(data)

  v(1.5em)

  grid(
    columns: (1fr, auto),
    gutter: 2em,
    [
      #text(size: 8pt, fill: gray, upper(l.bill_to)) \
      #v(0.2em)
      #client-block(data)
    ],
    align(right)[
      #text(size: 8pt, fill: gray)[#upper("Date")] \
      #data.invoice.date
      #if data.toggles.show_due_date and data.invoice.due_date != none [
        \ \ #text(size: 8pt, fill: gray)[#upper(l.due)] \
        #data.invoice.due_date
      ]
    ],
  )

  v(1.5em)

  if data.header_text != none {
    text(data.header_text)
    v(0.5em)
  }

  items-table(data)

  v(0.5em)
  align(right)[
    #table(
      columns: (auto, auto),
      align: (right, right),
      stroke: none,
      inset: (x: 6pt, y: 3pt),
      text(size: 9pt, fill: gray, l.subtotal),
      data.invoice.subtotal,
      ..data.invoice.taxes.map(t => (
        text(size: 9pt, fill: gray, t.name + " (" + t.percentage + ")"),
        t.amount,
      )).flatten(),
      text(size: 10pt, weight: "bold", upper(l.total)),
      text(size: 10pt, weight: "bold", data.invoice.total),
    )
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
