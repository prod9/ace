#import "../theme.typ": *

= Architecture

// S20
== Three Layers

#slide(composer: (1fr, 1fr, 1fr))[
  === Config

  Pure I/O. No merging. \
  No logic. No opinions.

  #v(0.8em)
  #text(size: 16pt, fill: rgb("#888888"))[
    `ace.toml` \
    `school.toml` \
    `user_config`
  ]
][
  === State

  Domain tree. \
  Merge configs, resolve schools, \
  run actions.

  #v(0.8em)
  #text(size: 16pt, fill: rgb("#888888"))[
    Install, Update, Link, \
    Authenticate, Exec
  ]
][
  === Ace

  Entrypoint. \
  Holds State + output mode, \
  drives the CLI.

  #v(0.8em)
  #text(size: 16pt, fill: rgb("#888888"))[
    *Config* #sym.arrow.l *State* #sym.arrow.l *Ace*
  ]
]

// S21
== The Lifecycle

#slide(composer: (1fr, 1fr, 1fr))[
  === Discover

  #text(size: 18pt)[
    *1.* Find configs \
    *2.* Parse & merge
  ]

  #v(0.3em)
  #text(size: 14pt, fill: rgb("#888888"))[`ace.toml` + school + user prefs → single state tree.]
][
  === Prepare

  #text(size: 18pt)[
    *3.* Authenticate \
    *4.* Fetch school \
    *5.* Sync skills \
    *6.* Check tooling
  ]

  #v(0.3em)
  #text(size: 14pt, fill: rgb("#888888"))[Clone or update, symlink, verify prerequisites.]
][
  === Launch

  #text(size: 18pt)[
    *7.* Select backend \
    *8.* Inject prompt \
    *9.* `exec`
  ]

  #v(0.3em)
  #text(size: 14pt, fill: rgb("#888888"))[ACE replaces itself with the backend. Gone from memory.]
]

= What's Next

// S22
== Roadmap

*PKCE auth flow* — multi-user rollout blocker.

#pause

*Auto-detect school* — `prod9/ace` #sym.arrow.r `prod9/school`, zero-config.

#pause

*More backends* — Aider, Cursor, and beyond. Same school, any tool.

#pause

*School marketplace* — discover and import from the community.

// S23
#focus-slide[
  #text(size: 96pt, weight: "light", font: "Kanit")[`ace`]
  #v(0.5em)
  #text(size: 24pt, fill: rgb("#888888"))[prodigy9.co]
]
