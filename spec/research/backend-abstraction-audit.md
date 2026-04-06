# Backend Abstraction Pattern Audit

**Date:** 2026-04-06  
**Scope:** ACE codebase (`src/config/backend/` and `src/state/actions/exec.rs`)  
**Finding:** Mixed abstraction pattern with strong inconsistencies across backends

---

## Executive Summary

The backend abstraction in ACE exhibits a **hybrid pattern with significant inconsistencies**:

- **Primary pattern:** "Backend owns execution and returns clean data" (MCP operations, trust args)
- **Secondary pattern:** "Backend builds args and caller executes" (session execution via Exec action)
- **Inconsistency:** Some backends directly execute CLI commands (claude, droid), while others manipulate local config files (flaude, opencode), and one (codex) returns errors for unimplemented features

**Recommendation:** Standardize on "Backend owns I/O" for all management operations (MCP, trust args, readiness checks) and clarify the Exec separation between arg-building and execution.

---

## Backend Enum Methods (src/config/backend/mod.rs)

### Overview of Public Methods

```rust
pub fn binary(&self) -> &'static str         // Returns backend binary name
pub fn backend_dir(&self) -> &'static str    // Returns backend home dir (.claude, .opencode, etc)
pub fn instructions_file(&self) -> &'static str  // Returns instructions filename (CLAUDE.md, AGENTS.md)
pub fn trust_args(&self, trust) -> Result<Vec<String>, String>  // Returns CLI args for trust level
pub fn is_ready(&self) -> bool               // Checks if backend is configured
pub fn mcp_list(&self) -> HashSet<String>   // Lists registered MCP servers
pub fn mcp_remove(&self, name) -> Result<(), String>  // Removes an MCP server
pub fn mcp_add(&self, entry) -> Result<(), String>    // Registers an MCP server
```

### Method-by-Method Audit

---

## 1. `binary()` — Returns &'static str

**What it returns:** Backend CLI binary name (e.g., "claude", "droid", "flaude")

**Pattern:** Pure data mapper — no I/O

| Backend | Value |
|---------|-------|
| Claude  | "claude" |
| OpenCode | "opencode" |
| Codex | "codex" |
| Flaude | "flaude" |
| Droid | "droid" |

**Consistency:** ✅ All backends return a static string. No execution.

---

## 2. `backend_dir()` — Returns &'static str

**What it returns:** Directory name where backend stores state (.claude, .opencode, .agents, .factory)

**Pattern:** Pure data mapper — no I/O

| Backend | Value |
|---------|-------|
| Claude  | ".claude" |
| Flaude  | ".claude" |
| OpenCode | ".opencode" |
| Codex | ".agents" |
| Droid | ".factory" |

**Consistency:** ✅ All backends return a static string. No execution.

**Usage:** Called in `cmd/main.rs:30`, `cmd/setup.rs:37`, `cmd/pull.rs:37` to construct paths passed to Prepare, Link, and UpdateGitignore actions.

---

## 3. `instructions_file()` — Returns &'static str

**What it returns:** Filename for backend instructions (CLAUDE.md or AGENTS.md)

**Pattern:** Pure data mapper — no I/O

| Backend | Value |
|---------|-------|
| Claude  | "CLAUDE.md" |
| Flaude  | "CLAUDE.md" |
| OpenCode | "AGENTS.md" |
| Codex | "AGENTS.md" |
| Droid | "AGENTS.md" |

**Consistency:** ✅ All backends return a static string. No execution.

**Usage:** Called in `cmd/setup.rs:42` to construct path; file is created if missing.

---

## 4. `trust_args()` — Returns Vec<String> or Error

**What it returns:** Backend-specific CLI arguments for permission/trust level

**Pattern:** Backend owns arg construction; caller builds command

| Backend | Trust::Default | Trust::Auto | Trust::Yolo |
|---------|----------------|-------------|------------|
| Claude  | `[]` | `["--permission-mode", "auto"]` | `["--permission-mode", "bypassPermissions"]` |
| Flaude  | `[]` | `["--auto"]` | `["--yolo"]` |
| Droid   | `[]` | ❌ Error | `["--skip-permissions-unsafe"]` |
| OpenCode | `[]` | ❌ Error | ❌ Error |
| Codex   | `[]` | ❌ Error | ❌ Error |

**Pattern Analysis:**
- **Ownership:** Backend enum owns the mapping; no I/O
- **Caller responsibility:** `cmd/main.rs:44-46` calls `trust_args()`, then extends `backend_args` vec
- **Consistency:** ✅ Uniform pattern: returns args, caller extends vector

**Usage:**
```rust
// cmd/main.rs:44-46
if !trust.is_default() {
    match backend.trust_args(trust) {
        Ok(args) => backend_args.extend(args),
        Err(msg) => ace.warn(&format!("trust ignored: {msg}")),
    }
}
```

---

## 5. `is_ready()` — Returns bool

**What it returns:** Whether backend is configured/authenticated

**Pattern:** Varies significantly by backend

### Claude Implementation (`src/config/backend/claude.rs:101-107`)

```rust
pub(super) fn is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home).join(".claude.json").exists()
}
```

**Ownership:** Backend module owns file check  
**Returns:** bool (file existence check)

### Droid Implementation (`src/config/backend/droid.rs:7-15`)

```rust
pub(super) fn is_ready() -> bool {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return false,
    };
    std::path::Path::new(&home)
        .join(".factory/settings.json")
        .exists()
}
```

**Ownership:** Backend module owns file check  
**Returns:** bool

### OpenCode Implementation (`src/config/backend/opencode.rs:26-39`)

```rust
pub(super) fn is_ready() -> bool {
    let auth_path = data_dir().join("auth.json");
    if !auth_path.exists() {
        return false;
    }

    match std::fs::read_to_string(&auth_path) {
        Ok(content) => {
            let trimmed = content.trim();
            trimmed != "{}" && !trimmed.is_empty()
        }
        Err(_) => false,
    }
}
```

**Ownership:** Backend module owns file check and validation  
**Returns:** bool (existence + non-empty check)

### Codex Implementation (`src/config/backend/codex.rs:16-22`)

```rust
pub(super) fn is_ready() -> bool {
    std::env::var("CODEX_API_KEY").is_ok()
        || std::env::var("OPENAI_API_KEY").is_ok()
        || home_dir()
            .map(|d| d.join("auth.json").exists())
            .unwrap_or(false)
}
```

**Ownership:** Backend module owns env var and file checks  
**Returns:** bool (env vars OR file existence)

### Flaude Implementation

**Status:** Not implemented (returns `true` in mod.rs:88 — hardcoded!)

```rust
Backend::Flaude => true,  // INCONSISTENCY: No actual check
```

**Problem:** Flaude always reports ready without checking anything.

### Consistency Assessment: ❌ MAJOR INCONSISTENCY

- **Claude, Droid, Codex, OpenCode:** Perform file/env checks in backend module
- **Flaude:** Hardcoded `true` (no check at all)
- **Codex additionally:** Checks environment variables, not just files

**Current Usage:** Marked with `#[allow(dead_code)]` in mod.rs:84 — **not called in main codebase** (only in tests and mod.rs). Dead code alert.

---

## 6. `mcp_list()` — Returns HashSet<String>

**What it returns:** Names of registered MCP servers

**Pattern:** Varies significantly by backend

### Claude Implementation (`src/config/backend/claude.rs:7-20`)

```rust
pub(super) fn mcp_list() -> HashSet<String> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return HashSet::new(),
    };

    let path = std::path::Path::new(&home).join(".claude.json");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    parse_mcp_names(&content)
}
```

**Ownership:** Backend module owns file read + JSON parsing  
**Returns:** HashSet of server names (best-effort, empty set on failure)

### Flaude Implementation (`src/config/backend/flaude.rs:21-36`)

```rust
pub(super) fn mcp_list() -> HashSet<String> {
    let Some(path) = mcp_list_path() else {
        return HashSet::new();
    };

    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };

    content
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}
```

**Ownership:** Backend module owns file read + line parsing  
**Returns:** HashSet of server names

### Droid Implementation (`src/config/backend/droid.rs:17-20`)

```rust
pub(super) fn mcp_list() -> HashSet<String> {
    // TODO: Parse ~/.factory/mcp.json when format is confirmed.
    HashSet::new()
}
```

**Ownership:** Backend module (placeholder, unimplemented)  
**Returns:** Empty set

### OpenCode Implementation (`src/config/backend/opencode.rs:43-50`)

```rust
pub(super) fn mcp_list() -> HashSet<String> {
    let config_path = config_dir().join("opencode.json");
    let content = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return HashSet::new(),
    };
    parse_mcp_names(&content)
}
```

**Ownership:** Backend module owns file read + JSON parsing  
**Returns:** HashSet of remote server names (filters out local MCPs)

### Codex Implementation (`src/config/backend/codex.rs:24-26`)

```rust
pub(super) fn mcp_list() -> HashSet<String> {
    HashSet::new()
}
```

**Ownership:** Backend module (placeholder, unimplemented)  
**Returns:** Empty set

### Consistency Assessment: ✅ Pattern Consistent, ❌ Feature Gaps

**Pattern:** All backends own file I/O and return `HashSet<String>` (best-effort, never fail)

**Issues:**
- Droid and Codex return empty sets indefinitely (TODOs noted)
- No differentiation: caller cannot tell if list is empty because unconfigured vs. unimplemented

**Usage:** `state/actions/mcp_register.rs:30`
```rust
let registered = self.backend.mcp_list();
```

---

## 7. `mcp_add()` — Returns Result<(), String>

**What it returns:** Success or error message

**Pattern:** Varies significantly by backend

### Claude Implementation (`src/config/backend/claude.rs:35-49`)

```rust
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    let args = build_mcp_add_args(entry);  // Backend builds args

    let output = Command::new("claude")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}
```

**Ownership:** Backend module builds args **AND executes the command**  
**Returns:** Result (executes `claude mcp add ...`)

### Droid Implementation (`src/config/backend/droid.rs:22-36`)

```rust
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    let args = build_mcp_add_args(entry);

    let output = Command::new("droid")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}
```

**Ownership:** Backend module builds args **AND executes the command**  
**Returns:** Result (executes `droid mcp add ...`)

### Flaude Implementation (`src/config/backend/flaude.rs:80-106`)

```rust
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let record_path = mcp_record_path()
        .ok_or_else(|| "HOME not set".to_string())?;

    let mut headers: Vec<(&String, &String)> = entry.headers.iter().collect();
    headers.sort_by_key(|(k, _)| k.as_str());

    let record = serde_json::json!({
        "action": "mcp_add",
        "name": entry.name,
        "url": entry.url,
        "headers": headers.iter()
            .collect::<Vec<_>>(),
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record_path)
        .map_err(|e| format!("open {}: {e}", record_path.display()))?;

    writeln!(file, "{record}").map_err(|e| format!("write {}: {e}", record_path.display()))?;
    Ok(())
}
```

**Ownership:** Backend module builds record **AND writes to local file**  
**Returns:** Result (writes to `~/.flaude-mcp-records.jsonl`)

**DIFFERENCE:** No CLI execution — records intent for later playback

### OpenCode Implementation (`src/config/backend/opencode.rs:111-135`)

```rust
pub(super) fn mcp_add(entry: &McpDecl) -> Result<(), String> {
    use std::io::Write;

    let config_dir = config_dir();
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("create {}: {e}", config_dir.display()))?;

    let config_path = config_dir.join("opencode.json");

    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read {}: {e}", config_path.display()))?
    } else {
        "{}".to_string()
    };

    let output = merge_mcp_entry(&existing, entry)?;

    let mut file = std::fs::File::create(&config_path)
        .map_err(|e| format!("create {}: {e}", config_path.display()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(())
}
```

**Ownership:** Backend module builds config **AND writes to JSON file**  
**Returns:** Result (updates `~/.config/opencode/opencode.json`)

**DIFFERENCE:** No CLI execution — modifies config directly

### Codex Implementation (`src/config/backend/codex.rs:32-34`)

```rust
pub(super) fn mcp_add(_entry: &McpDecl) -> Result<(), String> {
    Err("MCP registration not yet implemented for this backend".to_string())
}
```

**Ownership:** Unimplemented  
**Returns:** Error (feature not available)

### Consistency Assessment: ❌ MAJOR INCONSISTENCY

| Backend | Execution Model | I/O Method |
|---------|-----------------|-----------|
| Claude | Owns execution | Executes CLI (`claude mcp add ...`) |
| Droid | Owns execution | Executes CLI (`droid mcp add ...`) |
| Flaude | Owns execution | Writes JSON record (no CLI) |
| OpenCode | Owns execution | Modifies JSON config (no CLI) |
| Codex | Not implemented | Returns error |

**Problem:** Three different execution models within the same method:
1. **CLI execution** (Claude, Droid)
2. **File writing** (Flaude, OpenCode)
3. **Not implemented** (Codex)

---

## 8. `mcp_remove()` — Returns Result<(), String>

**What it returns:** Success or error message

**Pattern:** Same inconsistency as mcp_add

### Claude Implementation (`src/config/backend/claude.rs:74-88`)

```rust
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    let args = build_mcp_remove_args(name);

    let output = Command::new("claude")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}
```

**Ownership:** Backend module **executes CLI**  
**Returns:** Result (executes `claude mcp remove ...`)

### Droid Implementation (`src/config/backend/droid.rs:38-52`)

```rust
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    let args = build_mcp_remove_args(name);

    let output = Command::new("droid")
        .args(&args)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    Ok(())
}
```

**Ownership:** Backend module **executes CLI**  
**Returns:** Result (executes `droid mcp remove ...`)

### Flaude Implementation (`src/config/backend/flaude.rs:39-77`)

```rust
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    use std::io::Write;

    // -- append removal record --
    let record_path = mcp_record_path()
        .ok_or_else(|| "HOME not set".to_string())?;

    let record = serde_json::json!({
        "action": "mcp_remove",
        "name": name,
    });

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&record_path)
        .map_err(|e| format!("open {}: {e}", record_path.display()))?;

    writeln!(file, "{record}").map_err(|e| format!("write {}: {e}", record_path.display()))?;

    // -- update list file --
    let Some(list_path) = mcp_list_path() else {
        return Ok(());
    };

    let existing = std::fs::read_to_string(&list_path).unwrap_or_default();
    let updated: Vec<&str> = existing
        .lines()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && *s != name)
        .collect();

    std::fs::write(&list_path, updated.join("\n"))
        .map_err(|e| format!("write {}: {e}", list_path.display()))?;

    Ok(())
}
```

**Ownership:** Backend module **modifies files**  
**Returns:** Result (appends to record file, updates list file)

### OpenCode Implementation (`src/config/backend/opencode.rs:75-95`)

```rust
pub(super) fn mcp_remove(name: &str) -> Result<(), String> {
    use std::io::Write;

    let config_path = config_dir().join("opencode.json");

    let existing = if config_path.exists() {
        std::fs::read_to_string(&config_path)
            .map_err(|e| format!("read {}: {e}", config_path.display()))?
    } else {
        return Ok(());
    };

    let output = remove_mcp_entry(&existing, name)?;

    let mut file = std::fs::File::create(&config_path)
        .map_err(|e| format!("create {}: {e}", config_path.display()))?;
    file.write_all(output.as_bytes())
        .map_err(|e| format!("write {}: {e}", config_path.display()))?;

    Ok(())
}
```

**Ownership:** Backend module **modifies JSON config**  
**Returns:** Result (updates config file)

### Codex Implementation (`src/config/backend/codex.rs:28-30`)

```rust
pub(super) fn mcp_remove(_name: &str) -> Result<(), String> {
    Err("MCP removal not yet implemented for this backend".to_string())
}
```

**Ownership:** Unimplemented  
**Returns:** Error

### Consistency Assessment: ❌ MAJOR INCONSISTENCY (Same as mcp_add)

| Backend | Execution Model | I/O Method |
|---------|-----------------|-----------|
| Claude | Owns execution | Executes CLI (`claude mcp remove ...`) |
| Droid | Owns execution | Executes CLI (`droid mcp remove ...`) |
| Flaude | Owns execution | Appends/updates files (no CLI) |
| OpenCode | Owns execution | Modifies JSON config (no CLI) |
| Codex | Not implemented | Returns error |

---

## Session Execution Pattern: Exec Action

### Location: `src/state/actions/exec.rs`

```rust
pub struct Exec {
    pub backend: Backend,
    pub session_prompt: String,
    pub project_dir: PathBuf,
    pub env: HashMap<String, String>,
    pub backend_args: Vec<String>,
}

impl Exec {
    pub fn run(&self, _ace: &mut Ace) -> Result<(), std::io::Error> {
        if self.backend == Backend::Flaude {
            flaude_record_exec(&self.backend_args)?;
            return Ok(());
        }

        let mut cmd = Command::new(self.backend.binary());
        cmd.current_dir(&self.project_dir);

        for (key, val) in &self.env {
            cmd.env(key, val);
        }

        let args = build_exec_args(&self.session_prompt, &self.backend_args);
        cmd.args(&args);

        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        Err(err)
    }
}

fn build_exec_args(session_prompt: &str, backend_args: &[String]) -> Vec<String> {
    let mut args = vec![
        "--system-prompt".to_string(),
        session_prompt.to_string(),
    ];
    args.extend_from_slice(backend_args);
    args
}
```

### Pattern Analysis

**Ownership:**
- **Exec action owns execution** for all backends except Flaude
- **Args are pre-built** by `Backend::trust_args()` and passed to Exec as `backend_args`
- **Exec just appends** system-prompt to the pre-built args

**Data Flow:**
1. `cmd/main.rs:44-46` calls `Backend::trust_args()` → gets Vec<String>
2. `cmd/main.rs:65` passes to Exec as `backend_args`
3. `Exec::run()` calls `build_exec_args()` to prepend `--system-prompt`
4. Exec executes the backend CLI with combined args (or records for Flaude)

**Special Case: Flaude**
- Flaude just **records the call** instead of executing (line 18-20)
- Records to `~/.flaude-exec-records.jsonl` for test assertion

**Consistency:** ✅ Pattern is consistent for session execution

---

## Caller Usage Pattern

### How Callers Interact with Backend

**MCP Management** (`state/actions/mcp_register.rs` and `mcp_remove.rs`):
- Caller retrieves list: `backend.mcp_list()` → own I/O
- Caller adds entry: `backend.mcp_add()` → backend owns execution
- Caller removes: `backend.mcp_remove()` → backend owns execution
- **Pattern:** Backend owns everything

**Session Execution** (`cmd/main.rs` → `exec.rs`):
- Caller asks for trust args: `backend.trust_args()` → returns Vec<String>
- Caller passes to Exec struct
- Exec builds final args (prepends system-prompt)
- **Pattern:** Exec owns final execution, backend provides args

**Configuration** (various callers):
- Caller gets directory: `backend.backend_dir()` → pure data
- Caller gets filename: `backend.instructions_file()` → pure data
- **Pattern:** Backend provides data, caller uses it

---

## Inconsistencies Summary

### 1. **is_ready() Implementation Varies**
- Claude, Droid, OpenCode, Codex: File/env checks in backend module
- Flaude: Hardcoded `true` (never checks)
- **Status:** Marked dead code; not used in production

### 2. **mcp_list() Feature Completeness Gaps**
- Claude, Flaude, OpenCode: Working implementations
- Droid, Codex: Return empty sets indefinitely (TODOs noted)
- **Impact:** Caller cannot distinguish "not registered" from "unimplemented"

### 3. **mcp_add() / mcp_remove() Execution Models Differ**
- Claude, Droid: Execute backend CLI commands
- Flaude: Write JSON records (no CLI execution)
- OpenCode: Modify JSON config directly (no CLI execution)
- Codex: Return error (unimplemented)
- **Impact:** Backends have fundamentally different behavior despite same interface

### 4. **Trust Args: Only Abstract Model Partially Defines Support**
- No backend can list which trust levels it supports (must try-catch)
- Error messages are generic ("trust=Yolo not supported for ...")
- Codex, OpenCode, Droid don't support Auto mode, but this isn't obvious from code

---

## TODO Comment Analysis

**mod.rs:53-56** — Explicit acknowledgment of the problem:

```rust
// TODO: Re-analyze the abstraction boundary between ACE and backends. Currently
// ACE knows backend-specific flags (yolo, system-prompt, etc.) scattered across
// Exec and here. Consider whether backends should own their full arg construction
// (a BackendOpts struct or trait) instead of ACE assembling args piecemeal.
```

This TODO directly identifies the core issue: **mixed ownership of arg construction and execution**.

---

## Recommendations

### 1. **Unify MCP Operation Ownership**

All backends should follow the same pattern for MCP operations. Two options:

**Option A: Backend owns execution (current for CLI backends)**
- Simplify Flaude/OpenCode to use CLI execution like Claude/Droid
- Standardize error handling and return types

**Option B: Caller owns execution (reverse direction)**
- Backend modules return structured data (args, file content, config patch)
- Caller decides whether to execute CLI or modify files
- Better for testing and consistency

**Recommendation:** Option A (backend owns execution) requires less refactoring but needs CLI binaries for all backends. Option B is more testable but requires restructuring.

### 2. **Fix is_ready() Dead Code**

Current implementation in mod.rs:85-92 is marked dead code but should be:
- Implemented for Flaude
- Integrated into setup/validation workflows
- Provide clear diagnostics when backend is not configured

### 3. **Complete Droid and Codex MCP Support**

- Droid: Parse `~/.factory/mcp.json` (format TBD)
- Codex: Decide on MCP strategy and implement or document intentional non-support

### 4. **Create Backend Capability Matrix**

Formalize which features each backend supports (MCP registration, trust levels, etc.) as a data structure rather than error messages:

```rust
pub trait BackendCapabilities {
    fn supports_trust_level(&self, level: Trust) -> bool;
    fn supports_mcp(&self) -> bool;
    fn list_supported_trust_levels(&self) -> Vec<Trust>;
}
```

### 5. **Clarify Exec Abstraction Boundary**

Decide explicitly:
- **Who builds args?** Backend (via `trust_args`) or Exec?
- **Who executes?** Exec (current) or Backend?
- **How are special backends handled?** (Flaude vs CLI backends)

Document in architecture decision record.

### 6. **Standardize Error Handling for MCP Operations**

Current pattern: `Result<(), String>` with informal error messages
- Consider adding context (which backend, what operation)
- Add structured error enum for MCP-specific failures

---

## Conclusion

The backend abstraction pattern is **functionally working but architecturally inconsistent**. The primary issue is that different backends have different execution models (CLI vs. file-based vs. unimplemented), yet they present the same interface. This creates hidden assumptions in the caller code.

The TODO in mod.rs correctly identifies that **ACE and backends have poorly defined boundaries**. Recommend:

1. **Short-term:** Document which features are supported by which backends
2. **Medium-term:** Standardize execution models (all CLI, all file-based, or caller-executes)
3. **Long-term:** Introduce capability traits to formalize backend feature matrix

The session execution pattern (Exec action) is relatively clean and consistent. Focus refactoring efforts on the MCP operations (add/remove/list).
