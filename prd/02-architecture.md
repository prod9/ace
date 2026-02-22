# Architecture

## Layers

Three layers, each with a single responsibility:

### Config (`src/config/`)

Dumb I/O. Loads files from disk, parses TOML, writes back. No merging, no resolution,
no business logic.

- `AceToml` — shape of `ace.toml` / `ace.local.toml` / user `config.toml`
- `SchoolToml` — shape of `school.toml`
- `UserConfig` — shape of user credentials file
- `AcePaths` — computes config file locations from project dir
- `SchoolPaths` — computes school cache/root locations from specifier

Config structs are parse-and-forget. They don't know about each other or about
override precedence.

### State (`src/state/`)

The live, mutable domain tree. Passed to actions as the single source of truth.

State imports Config types and owns conversion to/from disk representation:
- **Loading** — reads Config structs, applies merge/resolution semantics (override
  precedence, env merging, school specifier resolution), produces the state tree
- **Saving** — converts state back into Config structs for persistence

State owns:
- **Domain objects** — `School`, `Service`, skills, conventions
- **Merge/resolution rules** — which layers override which, how env keys combine
- **Serialization boundary** — `from(config structs) -> State`, `to() -> config structs`

Actions receive and mutate the state tree.

### Ace (`src/ace/`)

Entrypoint. Orchestrates the lifecycle:

1. Calls State to load from disk (State uses Config internally)
2. Actions run, mutating the state tree
3. Ace calls State to persist changes (State uses Config internally)

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
