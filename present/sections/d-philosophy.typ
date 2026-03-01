#import "../theme.typ": *

= Philosophy

// S13
== Why Not Just Commit Skills?

#slide(composer: (1fr, 1fr))[
  === The naive approach

  Copy skills into each repo. \
  Check them in. Ship it.

  #v(0.3em)
  #text(size: 16pt, fill: rgb("#888888"))[Feels right on day one.]
][
  === Day thirty

  #pause

  One typo fix in a skill. \
  14 repos to patch. \
  3 are already diverged.

  #v(0.3em)
  #text(size: 16pt, fill: rgb("#888888"))[Copying is easy. Keeping copies in sync is the real problem.]
]

// S14
== Why Schools?

#slide(composer: (1fr, 1fr, 1fr))[
  === Single source

  School = git repo \
  Skills live once \
  Cache on each machine

  #text(size: 16pt, fill: rgb("#888888"))[No copies to drift.]
][
  === Edit in place

  Symlinks point to cache \
  Edit in any project \
  `ace school propose` back

  #text(size: 16pt, fill: rgb("#888888"))[Contribution is free.]
][
  === Instant onboard

  `ace setup prod9/school` \
  Clone, link, done \
  Same skills as everyone

  #text(size: 16pt, fill: rgb("#888888"))[Convention over configuration.]
]

// S15
== Why Always-Latest?

Traditional: pin a version, freeze behavior, prevent breakage. \
Assumes the consumer is *dumb*.

#pause

#v(0.8em)

An LLM is not dumb. It *reads*. It *adapts*. \
Same prompt, different day — different output anyway.

#pause

#v(1em)

#align(center)[
  #text(size: 28pt, fill: rgb("#00ffff"))[
    The consumer is intelligent. Always-latest is cheap.
  ]
]

// S16
== Dumb Sync, Smart Backend

#align(center)[
  #text(size: 28pt)[ACE = file sync + config merge + `exec`]
]

#pause

#v(0.8em)

No skill resolution. No dependency graph. No runtime logic. \
ACE is gone after `exec`.

#pause

#v(1em)

#align(center)[
  #text(size: 28pt, fill: rgb("#00ffff"))[
    The best orchestrator does the least orchestration.
  ]
]
