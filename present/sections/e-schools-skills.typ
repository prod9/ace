#import "../theme.typ": *

= Schools & Skills

// S17
== What is a School?

#slide(composer: (1fr, 1fr))[
  #v(0.3em)
  #text(size: 18pt, fill: rgb("#888888"), font: "Source Code Pro")[
    ```
    prod9/school/
    ├── school.toml
    ├── CLAUDE.md
    └── skills/
        ├── general-coding/
        │   └── SKILL.md
        ├── rust-coding/
        │   └── SKILL.md
        └── typst-coding/
            └── SKILL.md
    ```
  ]
][
  #v(0.3em)
  A *git repo* that defines how AI works for your org.

  #pause

  #v(0.5em)
  #text(size: 24pt)[`school.toml`] \
  #text(size: 16pt, fill: rgb("#888888"))[Name, imports, school-level config.]

  #pause

  #v(0.5em)
  #text(size: 24pt)[`skills/`] \
  #text(size: 16pt, fill: rgb("#888888"))[Shared skill packages — symlinked into every project.]

  #pause

  #v(0.5em)
  #text(size: 24pt)[`CLAUDE.md`] \
  #text(size: 16pt, fill: rgb("#888888"))[Global instructions applied to all projects.]
]

// S18
== Skills

A skill is a *`SKILL.md`* with frontmatter + instructions.

#v(0.5em)

#block(
  width: 100%,
  inset: (x: 1.2em, y: 0.8em),
  radius: 6pt,
  fill: rgb("#222222"),
)[
  #text(size: 16pt, font: "Source Code Pro", fill: rgb("#888888"))[`rust-coding/SKILL.md`]
  #v(0.3em)
  #text(size: 16pt, font: "Source Code Pro")[
    ```yaml
    ---
    name: rust-coding
    description: >
      Rust conventions — error handling,
      Option/Result idioms, serde, testing.
    ---
    ```
  ]
]

#pause

#v(0.3em)

AI reads skills *contextually* — not config, not templates. \
*Prose that the AI understands.*

// S19
== Importing Skills

#slide(composer: (1fr, 1fr))[
  === From the community

  ```bash
  $ ace import anthropics/skills \
      --skill commit
  ```

  #v(0.3em)
  #text(size: 16pt, fill: rgb("#888888"))[skills.sh — Anthropic-blessed community registry.]
][
  === How it works

  - No `npx skills` — *`ace import`* handles it

  #pause

  - Imported into the *school repo* \
    #text(size: 16pt, fill: rgb("#888888"))[Whole team gets it.]

  #pause

  - Tracked in `school.toml` imports \
    #text(size: 16pt, fill: rgb("#888888"))[Code-reviewed.]
]
