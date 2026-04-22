# Architecture

## Layers

Three layers, each with a single responsibility:

### Config (`src/config/`)

Dumb I/O. Loads files from disk, parses TOML, writes back. No merging, no resolution,
no business logic.

- `AceToml` — shape of `ace.toml` / `ace.local.toml` / `~/.config/ace/ace.toml`
- `SchoolToml` — shape of `school.toml`
- `IndexToml` — shape of `~/.cache/ace/index.toml` (tracks downloaded schools)
- `AcePaths` — computes config file locations from project dir
- `SchoolPaths` — computes school clone/root locations from specifier

Config structs are parse-and-forget. They don't know about each other or about
override precedence.

### State (`src/state/`)

The live, mutable domain tree. Passed to actions as the single source of truth.

State imports Config types and owns conversion to/from disk representation:
- **Loading** — reads Config structs, applies merge/resolution semantics (override
  precedence, env merging, school specifier resolution), produces the state tree
- **Saving** — converts state back into Config structs for persistence

State owns:
- **Domain objects** — `School`, `SkillSet`, `DiscoveredSkill`, `Tier`, conventions
- **Merge/resolution rules** — which layers override which, how env keys combine
- **Serialization boundary** — `from(config structs) -> State`, `to() -> config structs`
- **Pure reads** — `discover_skills` lives here alongside `SkillSet` since the
  producer/consumer pipeline is straightforward

State is data, not behavior. Actions consume it.

### Actions (`src/actions/`)

Peer to State, not nested inside it. Actions are operations *on* State and the
filesystem. Grouped by user role, not by mutation subject
(see `spec/decisions/005-action-layout.md`):

- **`actions/project/`** — consumer-side. User is working in their own repo
  that consumes a school. Covers setup, prepare, clone, link, pull,
  register/remove MCP, update gitignore.
- **`actions/school/`** — maintainer-side. User is working inside a school
  repo, curating skills. Covers init, add_import, pull_imports.

### Ace (`src/ace/`)

Entrypoint. Orchestrates the lifecycle:

1. Calls State to load from disk (State uses Config internally)
2. Actions run, mutating the state tree
3. Ace calls State to persist changes (State uses Config internally)

## Ace Instance Contract

A single `Ace` instance is created in `main()` and passed to all commands. It starts with
empty state — functioning only as an output sink.

Commands declare what they need by calling `require_*` methods on the existing instance:

- **`require_state()`** — lazily loads the config tree and resolves State. No-op if already
  loaded. Gives access to `school_specifier`, `backend`, `session_prompt`, `env`, etc.
- **`require_school()`** — resolves school context (root, cache, specifier). Handles
  dual-context detection: `school.toml` in cwd (school repo) vs `ace.toml` specifier (app
  repo). Calls `require_state()` internally when in app repo context.

Commands fall into three tiers:

1. **No state** — `setup`, `fmt`, `school init`. Ace is purely an output sink.
2. **Partial state** — `paths`, `diff`, `import`, `update`. Call `require_state()`
   or `require_school()` for what they need.
3. **Full orchestration** — bare `ace` (no subcommand). Runs Prepare, loads school.toml,
   builds session prompt, execs backend.

Never create new Ace instances inside commands. The single instance is the context — extend
it with lazy loading rather than bypassing it.

## Data Flow

```
disk → State::load() → state tree
                           ↓
                       actions mutate
                           ↓
                       State::save() → disk
```

State internally uses Config for the disk I/O:

```
State::load():  Config::load() → parsed structs → merge/resolve → state tree
State::save():  state tree → config structs → Config::save() → disk
```

## Dependency Direction

```
Config ← State ← Ace
              ← Actions
```

- Config imports nothing from the project
- State imports Config (for disk representation types)
- Ace and Actions import State (for the domain tree)
- Config never imports State
- State never imports Actions

## Standalone Modules

Not everything fits the Config → State → Ace pipeline. Standalone helper modules
live at the `src/` top level when they are independent of the domain tree:

- `src/git.rs` — git subprocess helpers
- `src/glob.rs` — simple glob matching
- `src/upgrade/` — self-update: version check, binary download, self-replacement

These modules may be called from `main()`, `cmd/`, or `Ace` but do not import
State or Actions. They receive only the specific values they need (paths, version
strings, output mode) rather than the full `Ace` instance.
