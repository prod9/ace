#import "../theme.typ": *

= What ACE Does

// S06
== Three Things

#text(size: 24pt)[*1.* Check environment readiness]
#v(0.3em)
#text(size: 16pt, fill: rgb("#888888"))[Git? Backend installed? Credentials valid?]

#pause

#v(0.8em)
#text(size: 24pt)[*2.* Install and update skills]
#v(0.3em)
#text(size: 16pt, fill: rgb("#888888"))[Clone school, symlink conventions, keep everything current.]

#pause

#v(0.8em)
#text(size: 24pt)[*3.* Configure and launch]
#v(0.3em)
#text(size: 16pt, fill: rgb("#888888"))[Inject prompt, set credentials, exec the backend.]

// S07
== For Developers

#slide(composer: (1fr, 1fr))[
  === You type

  ```
  $ ace setup prod9/school
  ```
  #text(size: 16pt, fill: rgb("#888888"))[Once per repo.]

  #v(0.8em)

  ```
  $ ace
  ```
  #text(size: 16pt, fill: rgb("#888888"))[Every time after.]
][
  === ACE does

  + Clone the school repo
  + Authenticate with LiteLLM
  + Symlink skills into project
  + Merge configs (#text(size: 18pt)[`ace.toml`] + school)
  + Inject session prompt
  + `exec` the backend

  #v(0.5em)
  #text(size: 16pt, fill: rgb("#888888"))[All transparent. Sub-second.]
]

// S08
== For Teams

Define a *school* — one repo of shared skills for your org.

#pause

#v(0.5em)
Everyone *syncs on every run*. No drift.

#pause

#v(0.5em)
Zero manual setup across machines. \
New hire? #text(size: 18pt)[`ace setup`] and they're ready.

#pause

#v(0.5em)
Edit skills in-place → *propose changes back* to the school.

// S09
== The Backend

#align(center)[
  #text(size: 28pt)[
    ACE is *not* the AI.
  ]
]

#pause

#v(0.8em)

ACE is the *launcher*.

#v(0.3em)

- Backends are pluggable: *Claude Code*, *OpenCode*, *Codex*
- ACE prepares the environment, then calls `exec`
- After exec — ACE is gone from memory

#pause

#v(0.5em)
#text(size: 16pt, fill: rgb("#888888"))[The backend runs the show. ACE just made sure it had everything it needed.]
