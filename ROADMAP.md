# ACE Roadmap

Tracked in Linear (project: ACE, team: PRODIGY9, key: PROD9).

## In Progress

### MCP Registration (3 work items)

Register `[[mcp]]` from school.toml into the active backend. Design finalized in `spec/mcp.md`
and `spec/backend.md`. Three sequential work items:

1. **PROD9-11: Placeholder template engine** (`src/template.rs`)
   - 4-state parser (Text → MaybeOpen → Name → MaybeClose)
   - `extract_placeholders(input) -> Vec<String>`
   - `substitute(input, values) -> String`
   - Pure functions, no regex, no deps

2. **McpDecl struct update** (`src/config/school_toml.rs`)
   - Add `headers: HashMap<String, String>` (optional)
   - Add `instructions: String` (optional)

3. **Registration action** (`src/state/actions/register_mcp.rs` or similar)
   - For each `[[mcp]]` entry:
     - `claude mcp get <name>` — exists? warn and skip
     - Placeholders in headers? Print `instructions`, prompt user, substitute
     - `claude mcp add -t http -s user <name> <url> [-H "K: V" ...]`
   - Inject `instructions` into session prompt for mid-session auth help
   - Claude first. OpenCode/Codex deferred (file-based writes).
   - Runs during setup, after school install

Not in scope: OAuth, secret storage, stdio MCP, OpenCode/Codex.

## Backlog (in Linear)

- PROD9-6: `ace setup` wrong specifier blocks re-run
- PROD9-7: School init .gitignore template
- PROD9-9: Investigate additional backends (Cursor, Continue, Cline)
- PROD9-10: School scripts for machine/software setup
- PROD9-12: Startup logo animation
- Codex backend (already implemented as Backend::Codex, needs MCP + polish)
- `tool` field in AceToml
- `role`/`description` fields in AceToml
- Propose pending school cache changes
- Test infrastructure (network tests, DRY boilerplate, extract shared git/fs utils)
- Ctrl+C / signal handling
- Auto `--continue` magic
- Build & release (cross-build, release workflow, self-update)
- `ace switch` (switch between backends)
- Skill diff tool
