# Invariant: Parameters Default to Active Context

### Scope

- **Purpose**: Ensure every CLI parameter has a sensible default so that the command works without arguments whenever a reasonable context (active account, active credentials) is available.
- **Responsibility**: Documents the parameter-default invariant for all `claude_profile` CLI commands.
- **In Scope**: All parameters across all 13 `clp` commands; applies to required-vs-optional decisions at both YAML registration and source-code argument extraction.
- **Out of Scope**: Parameters where a default is genuinely impossible (no ambient context exists) — these are the documented exceptions below.

### Invariant Statement

Every `claude_profile` CLI parameter **must** declare a default value unless requiring an explicit user value is absolutely necessary. "Absolutely necessary" means: no ambient context exists that could serve as a default AND the command cannot safely infer a target without an explicit value.

**Measurable threshold:** For every parameter marked `optional: false` in `unilang.commands.yaml`, there must be a documented justification in the relevant feature doc (AC or Notes section) explaining why no default is possible.

**Required pattern — parameter lookup:**
```rust
// Optional with context fallback (correct)
let raw_name = match cmd.arguments.get( "name" )
{
  Some( Value::String( s ) ) => s.clone(),
  _                          => String::new(),  // empty = "use ambient context"
};

// Mandatory (use only when no ambient context exists)
let raw_name = require_nonempty_string_arg( &cmd, "name" )?;
```

**Default sources by context:**
- Account-scoped commands → active account (per-machine active marker) when `name::` omitted
- Credential-scoped commands → live credentials (`~/.claude/.credentials.json`) when `name::` omitted
- Format output commands → `text` when `format::` omitted
- Dry-run commands → `0` (live mode) when `dry::` omitted

### Permitted Exceptions

The following parameters are legitimately required (no ambient default possible):

| Command | Parameter | Justification |
|---------|-----------|---------------|
| `.account.use` | `name::` | Switching to "the active account" is a no-op with no safe interpretation |
| `.account.delete` | `name::` | Deleting "the active account" is destructive and must be explicit |

All other account-name parameters must default to the active account or active credentials when omitted.

### Enforcement Mechanism

- YAML: `optional: false` triggers a documentation review; the feature doc must state why no default exists
- Source: `require_nonempty_string_arg` is the marker — grep `src/` for this call and verify each site is in the exceptions list
- Code review: any new `require_nonempty_string_arg` call must be accompanied by an exception justification in the feature doc

**Detection command:**
```bash
grep -n "require_nonempty_string_arg" dev/module/claude_profile/src/commands/cmd_args.rs
# Expected: only .account.use and .account.delete sites
```

### Violation Consequences

- Users who invoke a command without a name argument receive a confusing "required" error instead of the expected "use active" behavior
- Discoverability suffers — commands appear harder to use than necessary
- UX regression: users must remember to pass `name::` even in single-account setups where the intent is unambiguous

### Sources

| File | Relationship |
|------|-------------|
| `cli_doc.rulebook.md § Parameters Documentation : Default-First Design Principle` | Universal rulebook origin of this invariant — default strategy table, permitted exceptions, design health metric |
| `src/commands/limits.rs` | `account_limits_routine` — reference pattern for optional-name-with-fallback |
| `src/commands/account_relogin.rs` | `account_relogin_routine` — updated to use active-account fallback (TSK-173) |

### Features

| File | Relationship |
|------|-------------|
| [019_account_relogin.md](../feature/019_account_relogin.md) | First command fixed to satisfy this invariant — `name::` now defaults to active account |

### Invariants

| File | Relationship |
|------|-------------|
| [003_clear_errors.md](003_clear_errors.md) | Complementary: errors when fallback fails must be actionable |
