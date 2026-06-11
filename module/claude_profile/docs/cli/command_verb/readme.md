# Command Verbs

Domain action documentation for the `clp` CLI. Each verb represents an action applied to domain objects, following the `noun.verb` command naming pattern.

| File | Verb | Nouns | Commands | Idempotent |
|------|------|-------|---------|-----------|
| [001_save.md](001_save.md) | save | account | 1 | Conditional |
| [002_use.md](002_use.md) | use | account | 1 | Conditional |
| [003_delete.md](003_delete.md) | delete | account | 1 | Conditional |
| [004_limits.md](004_limits.md) | limits | account | 1 | Yes |
| [005_relogin.md](005_relogin.md) | relogin | account | 1 | No |
| [006_rotate.md](006_rotate.md) | rotate | account | 1 | No |
| [007_renewal.md](007_renewal.md) | renewal | account | 1 | Yes |
| [008_inspect.md](008_inspect.md) | inspect | account | 1 | Yes |
| [009_assign.md](009_assign.md) | assign | account | 1 | Yes |
| [010_status.md](010_status.md) | status | token, credentials | 2 | Yes |

**Total:** 10 domain verbs

### Verb Coverage Matrix

| Verb | account | token | credentials |
|------|---------|-------|-------------|
| save | yes | — | — |
| use | yes | — | — |
| delete | yes | — | — |
| limits | yes | — | — |
| relogin | yes | — | — |
| rotate | yes | — | — |
| renewal | yes | — | — |
| inspect | yes | — | — |
| assign | yes | — | — |
| status | — | yes | yes |

### See Also

- [../command/](../command/readme.md) — individual command specifications
- [../command_noun/](../command_noun/readme.md) — domain noun documentation
- [../param/](../param/readme.md) — parameter specifications
- [../user_story/](../user_story/readme.md) — user stories exercising these domain verbs
