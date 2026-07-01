# Command Verbs

Domain action documentation for the `clp` CLI. Each verb represents an action applied to domain objects, following the `noun.verb` command naming pattern.

| File | Verb | Nouns | Commands | Idempotent |
|------|------|-------|---------|-----------|
| [001_save.md](001_save.md) | save | account | 1 | Conditional |
| [002_use.md](002_use.md) | use | account | 1 | Conditional |
| [003_delete.md](003_delete.md) | delete | account | 1 | Conditional |
| [004_limits.md](004_limits.md) | limits | account | 1 | Yes |
| [005_relogin.md](005_relogin.md) | relogin | account | 1 | No |
| [006_rotate.md](006_rotate.md) | rotate *(DEPRECATED — Feature 038)* | account | — | — |
| [007_renewal.md](007_renewal.md) | renewal | account | 1 | Yes |
| [008_inspect.md](008_inspect.md) | inspect | account | 1 | Yes |
| [009_assign.md](009_assign.md) | assign *(REMOVED — Feature 037)* | account | — | — |
| [010_status.md](010_status.md) | status | token, credentials | 2 | Yes |
| [011_unclaim.md](011_unclaim.md) | unclaim *(REMOVED — Feature 064)* | account | — | — |
**Total:** 8 active domain verbs (11 entries; 1 DEPRECATED: rotate Feature 038; 2 REMOVED: assign Feature 037, unclaim Feature 064)

### Verb Coverage Matrix

| Verb | account | token | credentials |
|------|---------|-------|-------------|
| save | yes | — | — |
| use | yes | — | — |
| delete | yes | — | — |
| limits | yes | — | — |
| relogin | yes | — | — |
| rotate *(DEPRECATED)* | — | — | — |
| renewal | yes | — | — |
| inspect | yes | — | — |
| assign *(REMOVED)* | — | — | — |
| status | — | yes | yes |
| unclaim *(REMOVED)* | — | — | — |

### See Also

- [../command/](../command/readme.md) — individual command specifications
- [../command_noun/](../command_noun/readme.md) — domain noun documentation
- [../param/](../param/readme.md) — parameter specifications
- [../user_story/](../user_story/readme.md) — user stories exercising these domain verbs
