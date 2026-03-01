#import "@preview/touying:0.6.1": *
#import themes.metropolis: *

#let ace-theme(body) = {
  set text(font: "Kanit", weight: "light", size: 20pt, fill: rgb("#e0e0e0"))
  show heading: set text(font: "Space Grotesk", weight: "medium", fill: rgb("#00ffff"))
  show strong: set text(weight: "regular", fill: rgb("#00ffff"))
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
      primary: rgb("#00ffff"),
      secondary: rgb("#1a1a1a"),
      neutral-lightest: rgb("#1a1a1a"),
      neutral-darkest: rgb("#e0e0e0"),
    ),
    config-page(margin: (x: 3em, y: 2em)),
  )

  body
}
