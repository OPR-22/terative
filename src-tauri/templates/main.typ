#import "classic.typ"
#import "modern.typ"
#import "minimal.typ"

#let data = json.decode(sys.inputs.data)

#if data.layout == "Modern" {
  modern.render(data)
} else if data.layout == "Minimal" {
  minimal.render(data)
} else {
  classic.render(data)
}
