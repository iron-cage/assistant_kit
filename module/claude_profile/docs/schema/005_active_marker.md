# Schema: Active Marker — `_active_{host}_{user}`

### Scope

- **Purpose**: Define the per-machine active-account marker file format and naming convention.
- **In Scope**: Filename derivation, content format, sanitization rules, `.gitignore` exclusion.
- **Out of Scope**: Old shared `_active` file (deprecated, ignored); switching logic (→ [feature/004](../feature/004_account_use.md)).

### File Location

```
{credential_store}/_active_{hostname}_{user}
```

### Filename Derivation

```
hostname: $HOSTNAME env var
          → /etc/hostname (fallback)
          → "local" (fallback)

user:     $USER env var
          → $USERNAME (fallback)
          → "user" (fallback)

sanitize: keep [a-zA-Z0-9\-\.]; replace all other chars with '_'
```

Example: hostname `w003`, user `user1` → `_active_w003_user1`

### Content Format

Plain text. Single account name (email address), trimmed of whitespace. No JSON.

```
alice@example.com
```

### `.gitignore`

Pattern `_active_*` in the credential store's `.gitignore` excludes all per-machine markers from version control. Each machine is fully independent.

### `other_machines_active()` API

Reads all `_active_*` files in the credential store **except** the current machine's own marker. Returns `HashSet<String>` of account names (trimmed; empty strings excluded; missing/unreadable files skipped silently).

### Old `_active` File

The previous single-file `_active` marker is silently ignored. Running `.account.use` once on a machine populates the machine-specific marker and there is no migration step.

### Cross-References

| File | Relationship |
|------|-------------|
| [feature/025_per_machine_active_marker.md](../feature/025_per_machine_active_marker.md) | Feature spec with acceptance criteria |
| [schema/004](004_storage_root.md) | Credential store path (parent directory) |
| [feature/004_account_use.md](../feature/004_account_use.md) | Switch writes this marker |
| [feature/009_token_usage.md](../feature/009_token_usage.md) | Sessions table reads all `_active_*` markers |
