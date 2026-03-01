#import "@preview/touying:0.6.1": *
#import themes.metropolis: *
#import "theme.typ": ace-theme

#show: ace-theme

#title-slide()

= Overview

== What is ACE?

*AI Coding Environment* — automation tooling for AI-assisted development.

#pause

- Check environment readiness
- Install and update skills, agents, conventions
- Configure AI chatbots and model credentials

#pause

One command: `ace`

== The Problem

Every AI coding session starts with the same friction:

#pause

- Which model? Which API key?
- What conventions does this team follow?
- What skills does the agent need?

#pause

*ACE eliminates the setup tax.*

== How It Works

#slide(composer: (1fr, 1fr))[
  === For developers

  ```
  $ ace setup prod9/school
  $ ace
  ```

  That's it. Skills, conventions, and credentials — all configured.
][
  === For teams

  - Define a *school* with shared skills
  - `ace` syncs on every run
  - No manual setup across machines
]

= Architecture

== Three Layers

#slide(composer: (1fr, 1fr, 1fr))[
  === Config
  Dumb I/O. \
  Reads files, \
  no logic.
][
  === State
  Domain tree. \
  Merge, resolve, \
  run actions.
][
  === Ace
  Entrypoint. \
  Holds state, \
  drives CLI.
]
