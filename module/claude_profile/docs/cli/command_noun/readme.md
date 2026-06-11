# Command Nouns

Domain object documentation for the `clp` CLI. Each noun represents a domain object that CLI commands operate on, following the `noun.verb` command naming pattern.

| File | Noun | Commands | Purpose |
|------|------|---------|---------|
| [001_account.md](001_account.md) | account | 10 | Saved credential profile in the per-machine account store |
| [002_token.md](002_token.md) | token | 1 | OAuth access token for the active Claude Code session |
| [003_credentials.md](003_credentials.md) | credentials | 1 | Live OAuth credential metadata independent of the account store |

**Total:** 3 domain nouns

### Entity Relationship Map

```
[account] --has--> [token]
[account] --has--> [credentials]
[token]   --owned-by--> [account]
[credentials] --reflects-active--> [account]
```

- `account` is the primary domain object; all mutating operations target accounts.
- `token` is a derived view: the OAuth access token embedded in the active account's credential file.
- `credentials` is a live view: the identity metadata for the current session read directly from `~/.claude/.credentials.json`, independent of the account store.

### See Also

- [../command/](../command/readme.md) — individual command specifications
- [../command_verb/](../command_verb/readme.md) — domain verb documentation
- [../param/](../param/readme.md) — parameter specifications
- [../user_story/](../user_story/readme.md) — user stories operating on these domain nouns
