#import "../theme.typ": *

// S01
#title-slide()

// S02
#focus-slide[
  #text(size: 72pt, weight: "light", font: "Kanit")[
    #text(fill: rgb("#00ffff"))[AI] Coding Environment
  ]
]

// S03
#focus-slide[
  #text(size: 72pt, weight: "light", font: "Kanit")[
    #text(fill: rgb("#00ffff"))[Automated Claude] Environments
  ]
]

= The Problem

// S04
== The Setup Tax

#v(0.5em)

Every AI coding session starts with the same ritual:

#pause

- _"Which model? Where's the API key?"_

#pause

- _"What conventions does this codebase follow?"_

#pause

- _"What skills does the agent need to do its job?"_

#pause

#v(1em)

#align(center)[
  #text(size: 28pt, fill: rgb("#00ffff"))[
    You answer these questions every. single. time.
  ]
]

// S05
== One Command

#v(1em)

#align(center)[
  #block(width: 70%)[
    #align(left)[
      ```bash
      $ ace setup prod9/school   # once per repo
      $ ace                      # every time after
      ```
    ]
  ]
]

#pause

#v(1.5em)

#align(center)[
  #text(size: 32pt, fill: rgb("#00ffff"))[ACE eliminates the setup tax.]
]
