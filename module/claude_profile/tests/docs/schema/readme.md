# Schema Doc Entity

SC-N schema constraint specs for `claude_profile`. Each spec covers the on-disk format
contracts defined in `docs/schema/`, verifying field presence, type rules, encoding
invariants, and read/write semantics.

**SC- extension note:** SC- (Schema Constraint) is a project-local element type extension.
`docs/schema/` maps to `tests/docs/schema/` as a project-defined documentation entity
surface (see `tests/docs/readme.md` Surface Index). Min 4 SC- cases per spec.

### Responsibility Table

| File | Schema | SC-N Cases |
|------|--------|-----------|
| `001_credentials_json.md` | `{name}.credentials.json` — OAuth credential snapshot | SC-1 through SC-5 |
| `002_account_json.md` | `{name}.json` — supplementary account metadata | SC-1 through SC-6 |
| `003_file_topology.md` | `ClaudePaths` — path method contracts | SC-1 through SC-4 |
| `004_storage_root.md` | `PersistPaths` — storage root resolution | SC-1 through SC-4 |
| `005_active_marker.md` | `_active_{host}_{user}` — per-machine marker | SC-1 through SC-5 |
| `006_settings_json.md` | `~/.claude/settings.json` — clp-managed fields | SC-1 through SC-5 |
| `007_claude_json.md` | `~/.claude.json` — read-only OAuth state | SC-1 through SC-4 |

### Coverage Summary

| Schema Files | Total SC- Cases |
|-------------|-----------------|
| 7 | 33 (5+6+4+4+5+5+4) |

### See Also

- [docs/schema/](../../../docs/schema/readme.md) — schema source docs
- [docs/invariant/007_json_storage_format.md](../../../docs/invariant/007_json_storage_format.md) — JSON encoding invariant
