# Backend Hooks

Schools can declare hooks that ACE registers with the active backend. Hooks are shell commands
that backends execute in response to session events (tool use, context compaction, session
start, etc.).

## Motivation

Tools like mempalace need hooks to function: a save hook fires every N messages to capture
context, a pre-compact hook fires before context window compression for emergency saves. Without
school-managed hooks, each developer must manually configure these — defeating the purpose of
centralized school management.

## school.toml Format

```toml
[[hooks]]
event = "PreCompact"
command = "mempalace save --emergency"

[[hooks]]
event = "PostToolUse"
matcher = "Edit|Write"
command = "mempalace save --on-edit"

[[hooks]]
event = "SessionStart"
matcher = "startup"
command = "mempalace wake"
```

## Fields

- `event` — Backend hook event name. ACE passes this through to the backend without
  interpretation. See [event compatibility](#event-compatibility) for known events per backend.
- `matcher` — (optional) Filter pattern for the event. Semantics are backend-defined
  (regex for Claude, may differ for Codex). Omit to match all instances of the event.
- `command` — Shell command to execute. Runs in the project directory.
- `timeout` — (optional) Seconds before the hook is killed. Backend default if omitted.

## Event Compatibility

Events are backend-defined strings. ACE does not validate event names — it passes them through.
This keeps ACE decoupled from backend release cycles. Unknown events are silently ignored by
the backend.

Known events as of 2026-04:

| Event | Claude | Codex | Description |
|-------|--------|-------|-------------|
| `SessionStart` | ✓ | TBD | Session begins |
| `PreToolUse` | ✓ | TBD | Before tool executes |
| `PostToolUse` | ✓ | TBD | After tool succeeds |
| `PreCompact` | ✓ | TBD | Before context compaction |
| `PostCompact` | ✓ | TBD | After context compaction |
| `Stop` | ✓ | TBD | User stops the session |
| `Notification` | ✓ | TBD | UI notification |

Codex hook support is pending investigation. If Codex does not support hooks natively, ACE
should skip hook registration for Codex and print a warning listing the skipped hooks.

## Config Struct

```rust
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct HookDecl {
    pub event: String,
    #[serde(skip_serializing_if = "is_empty_str")]
    pub matcher: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u32>,
}
```

Added to `SchoolToml`:

```rust
pub struct SchoolToml {
    // ... existing fields ...
    #[serde(skip_serializing_if = "is_empty_vec")]
    pub hooks: Vec<HookDecl>,
}
```

## Validation

- `event` must be non-empty.
- `command` must be non-empty.
- No duplicate `(event, matcher)` pairs within a school.

## Registration

### Strategy: Settings File

Unlike MCP (which uses backend CLI), hooks are registered by writing to the backend's
project-level settings file. This is because:

- Claude Code hooks live in `.claude/settings.json` (no `claude hooks add` CLI exists).
- Project scope is correct — hooks are school/project policy, not user preference.

### Flow

1. **Load** — Read existing project settings file (if any).
2. **Merge** — Add school-declared hooks. Do not remove or overwrite hooks that exist in the
   file but are not declared by the school. School hooks are additive.
3. **Write** — Write merged settings back.

ACE marks school-managed hooks with a comment or metadata field so it can distinguish
school hooks from user hooks on subsequent runs. This enables clean updates: when the school
changes a hook, ACE updates the corresponding entry without touching user-added hooks.

### Per-Backend Settings

#### Claude

Hooks live in `.claude/settings.json` under the `hooks` key:

```json
{
  "hooks": {
    "PreCompact": [
      {
        "matcher": "auto",
        "hooks": [
          {
            "type": "command",
            "command": "mempalace save --emergency",
            "_ace": true
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Edit|Write",
        "hooks": [
          {
            "type": "command",
            "command": "mempalace save --on-edit",
            "_ace": true
          }
        ]
      }
    ]
  }
}
```

The `_ace: true` marker identifies school-managed entries. On re-registration, ACE removes
entries with `_ace: true` and re-adds the current school declarations. User entries (no
`_ace` marker) are preserved.

#### Codex

Codex hook support is TBD. Possible paths:

1. **Native hooks**: If Codex adds a hooks system, ACE writes to `.codex/config.toml` or
   equivalent with the same merge strategy.
2. **No hooks**: ACE warns and skips. The school can provide a skill with manual hook setup
   instructions as a stopgap.

### Backend Contract Addition

New optional method on Backend:

```rust
pub fn hooks_write(&self, project_dir: &Path, hooks: &[HookDecl]) -> Result<(), String>
```

- Reads existing settings, merges school hooks (keyed by `_ace` marker), writes back.
- Returns `Err` with a message if the backend does not support hooks.

No `hooks_list` or `hooks_remove` — ACE owns the full lifecycle through the settings file.
The write operation is idempotent: calling it with the same hooks produces the same file.

## Interaction with Linked Folders

`.claude/settings.json` is a file, not a linked folder. It coexists with the symlinked
`skills/`, `rules/`, etc. directories inside `.claude/`. ACE already manages files in the
backend directory (e.g., `CLAUDE.md`), so writing `settings.json` follows the same pattern.

If `.claude/settings.json` already exists with user content, ACE merges — it never
overwrites the entire file.

## Scope Boundaries

- ACE registers hooks declared in `school.toml` only. It does not provide a CLI for
  ad-hoc hook management (users edit settings files directly for personal hooks).
- ACE does not interpret hook semantics — it does not know what `PreCompact` means. The
  backend defines and executes hooks; ACE only places them in the right config location.
- Hook commands run in the backend's execution context, not ACE's. ACE is not running when
  hooks fire.
