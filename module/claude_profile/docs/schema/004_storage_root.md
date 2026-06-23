# Schema: Storage Root — `PersistPaths`

### Scope

- **Purpose**: Canonical reference for persistent storage path resolution via `PersistPaths`.
- **In Scope**: Resolution chain, computed paths, `PersistPaths` API.
- **Out of Scope**: What is stored under these paths (→ [schema/001](001_credentials_json.md), [schema/002](002_account_json.md), [schema/005](005_active_marker.md)).

### Resolution Chain

```
1. $PRO (if set AND points to existing directory)
   → {root}/.persistent/claude_profile/   (base)
   → {root}/.persistent/claude/credential/ (credential_store)

2. $HOME (Linux/macOS) or $USERPROFILE (Windows)
   → {root}/.persistent/claude_profile/
   → {root}/.persistent/claude/credential/

3. Both unset → error
```

`$PRO` set but pointing to a file or non-existent path → silently skip, continue to step 2.

### `PersistPaths` API

| Method | Resolves to |
|--------|-------------|
| `base()` | `{root}/.persistent/claude_profile/` |
| `credential_store()` | `{root}/.persistent/claude/credential/` |
| `ensure_exists()` | Creates `base()` directory idempotently |

### Credential Store Layout

```
{credential_store}/
  {name}.credentials.json     # OAuth credential snapshot
  {name}.json                 # Supplementary metadata
  _active_{host}_{user}       # Per-machine active marker (gitignored)
```

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/010_persistent_storage.md](../feature/010_persistent_storage.md) | Feature spec with acceptance criteria |
| [schema/001](001_credentials_json.md) | `{name}.credentials.json` format |
| [schema/002](002_account_json.md) | `{name}.json` format |
| [schema/005](005_active_marker.md) | `_active_{host}_{user}` format |
