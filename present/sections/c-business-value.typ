#import "../theme.typ": *

= Business Value

// S10
== For Engineering Teams

#slide(composer: (1fr, 1fr))[
  === Without ACE

  #text(fill: rgb("#888888"))[
    - Each dev configures AI differently
    - "What conventions do we follow again?"
    - New hire spends days on setup
    - Tribal knowledge lives in someone's head
    - "Works on my machine" — for AI too
  ]
][
  === With ACE

  - *One school* — identical AI behavior, every dev
  #pause
  - *One command* — new hire runs `ace setup`, done
  #pause
  - *Skills are code* — tribal knowledge survives turnover
  #pause
  - *Auto-sync* — no config drift between machines
]

// S11
== For Engineering Managers

*Visibility* — skills are code-reviewed PRs. You see exactly what AI is told.

#pause

*Control* — conventions and guardrails ship org-wide through the school.

#pause

*Consistency* — same skills, same inputs → comparable output quality.

#pause

*Cost* — ramp-up for new projects and new hires drops to `ace setup`.

// S12
== Skills as Product

#slide(composer: (1fr, 1fr))[
  === Encode

  Domain knowledge becomes a *school*. \
  Best practices, conventions, guardrails — \
  code-reviewed `SKILL.md` files.

  #v(0.3em)
  #text(size: 16pt, fill: rgb("#888888"))[Expertise that lived in someone's head — now it ships.]
][
  === Sell

  Publish a school. Customers run `ace setup`. \
  Instant access to your methodology.

  #v(0.3em)
  #text(size: 16pt, fill: rgb("#888888"))[Consulting firms, agencies, platform teams — expertise as a subscription.]
]
