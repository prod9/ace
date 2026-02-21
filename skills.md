# Skills

Skills are folders of instructions, scripts, and resources that AI coding agents load dynamically
to improve performance on specialized tasks. They teach agents how to complete specific tasks in a
repeatable way.

ACE manages skills by syncing them from a school repository into the local environment.

## Skill Structure

```
skill-name/
├── SKILL.md          # Required. YAML frontmatter + markdown instructions.
├── scripts/          # Optional. Executable code (Python/Bash).
├── references/       # Optional. Docs loaded into context on-demand.
└── assets/           # Optional. Templates, images, fonts for output.
```

No other files. No README, CHANGELOG, INSTALLATION_GUIDE, or auxiliary docs.

## SKILL.md Format

```yaml
---
name: skill-name
description: What the skill does and when to use it.
---

# Instructions here
```

### Frontmatter

Two required fields only:

- **name** — Lowercase, hyphens for spaces. Short and descriptive.
- **description** — Primary trigger mechanism. Must include both what the skill does AND when to
  use it. All "when to use" info goes here, not in the body. The body is only loaded after
  triggering, so "When to Use" sections in the body are wasted.

### Body

Markdown instructions for the agent. Target < 500 lines / ~5k words.

Use imperative/infinitive form ("Extract text", "Run the script", not "You should extract text").

## Core Principles

### 1. Concise is Key

The context window is shared with system prompt, conversation history, other skills, and the user
request. Default assumption: the agent is already smart. Only add what it doesn't already know.

Prefer concise examples over verbose explanations. Challenge each paragraph: "Does this justify
its token cost?"

### 2. Degrees of Freedom

Match specificity to fragility:

- **High freedom** (text instructions) — Multiple valid approaches, context-dependent decisions.
- **Medium freedom** (pseudocode, parameterized scripts) — Preferred pattern exists, some
  variation acceptable.
- **Low freedom** (specific scripts, few parameters) — Fragile operations, consistency critical,
  exact sequence required.

### 3. Progressive Disclosure

Three-level loading to manage context efficiently:

1. **Metadata** (name + description) — Always in context. ~100 words.
2. **SKILL.md body** — Loaded when skill triggers. < 5k words.
3. **Bundled resources** — Loaded as-needed by the agent. Unlimited.

Keep SKILL.md lean. Split into reference files when approaching 500 lines. Reference files must
be linked from SKILL.md with clear descriptions of when to read them.

## Bundled Resources

### scripts/

Executable code for tasks that need deterministic reliability or are repeatedly rewritten.

- Include when the same code would be rewritten each time.
- Example: `scripts/rotate_pdf.py` for PDF rotation.
- Scripts may be executed without loading into context (token efficient).
- Scripts may still need to be read by the agent for patching or environment adjustments.

### references/

Documentation loaded into context on-demand.

- Include for schemas, API docs, domain knowledge, company policies, detailed workflows.
- Example: `references/schema.md` for database table schemas.
- Keeps SKILL.md lean — loaded only when the agent determines it's needed.
- For large files (> 10k words), include grep search patterns in SKILL.md.
- Information lives in either SKILL.md or references, never both.

### assets/

Files used in output, not loaded into context.

- Include for templates, images, icons, boilerplate, fonts.
- Example: `assets/logo.png` for brand assets, `assets/template/` for project boilerplate.
- Separates output resources from documentation.

## Progressive Disclosure Patterns

### Pattern 1: High-level guide with references

```markdown
# PDF Processing

## Quick start
Extract text with pdfplumber: [example]

## Advanced features
- **Form filling**: See references/forms.md
- **API reference**: See references/api.md
```

Agent loads reference files only when needed.

### Pattern 2: Domain-specific organization

```
bigquery-skill/
├── SKILL.md (overview and navigation)
└── references/
    ├── finance.md
    ├── sales.md
    └── product.md
```

User asks about sales metrics — agent reads only `sales.md`.

### Pattern 3: Conditional details

```markdown
# DOCX Processing

## Creating documents
Use docx-js. See references/docx-js.md.

## Editing documents
For simple edits, modify XML directly.

**For tracked changes**: See references/redlining.md
**For OOXML details**: See references/ooxml.md
```

### Guidelines

- Keep references one level deep from SKILL.md. No nested references.
- For files > 100 lines, include a table of contents at the top.
- Avoid duplication between SKILL.md and reference files.

## Writing a Skill

### 1. Understand

Gather concrete usage examples. What would a user say to trigger this skill? What workflows does
it support? Skip only when usage patterns are already clearly understood.

### 2. Plan

Analyze each example:
- What scripts would avoid rewriting the same code?
- What references would avoid re-discovering the same information?
- What assets would avoid recreating the same boilerplate?

### 3. Create

Create the skill directory with SKILL.md and any bundled resources.

### 4. Write SKILL.md

- Write the description as the primary trigger. Be specific about capabilities and contexts.
- Write body instructions. Only include non-obvious procedural knowledge.
- Reference bundled resources with clear "when to read" guidance.
- Delete any unused directories (empty `scripts/`, `references/`, `assets/`).

### 5. Test

- Run any scripts to verify they work.
- Use the skill on real tasks.
- Check that triggering works correctly based on the description.

### 6. Iterate

- Use on real tasks, notice struggles or inefficiencies.
- Update SKILL.md or bundled resources.
- Test again.
