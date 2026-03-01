#import "@preview/touying:0.6.1": *
#import themes.metropolis: *

#let ace-theme(body) = {
  set text(font: "Kanit", weight: "light", size: 20pt)
  show heading: set text(font: "Space Grotesk", weight: "medium")
  show strong: set text(weight: "regular")
  set par(justify: false)

  show: metropolis-theme.with(
    aspect-ratio: "16-9",
    footer: self => self.info.institution,
    config-info(
      title: [ACE],
      subtitle: [AI Coding Environment],
      author: [PRODIGY9],
      date: datetime.today(),
      institution: [prodigy9.co],
    ),
    config-colors(
      primary: rgb("#1a1a1a"),
      secondary: rgb("#00ffff"),
      neutral-lightest: rgb("#fafafa"),
      neutral-darkest: rgb("#1a1a1a"),
    ),
    config-page(margin: (x: 3em, y: 2em)),
  )

  body
}
