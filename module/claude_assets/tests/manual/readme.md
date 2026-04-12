# tests/manual/

| File | Responsibility |
|------|----------------|
| `readme.md` | Manual testing plan: real $PRO_CLAUDE asset operations. |

## Manual Testing Plan — Symlink Install/Uninstall

**Trigger:** After any change to `install()`, `uninstall()`, `list_all()`,
`ArtifactKind`, or the adapter layer.

### Prerequisites

- `$PRO_CLAUDE` set to a real claude-assets directory (e.g., `$PRO/genai/claude/`)
- At least one source artifact per kind present (rule `.md`, command `.md`, hook `.yaml`, skill dir, etc.)
- `cla` binary compiled with `--features enabled`

### M-01: Full install/list/uninstall cycle (file-layout kind)

```
cla .list kind::rule
cla .install kind::rule name::<pick-a-real-rule>
cla .list kind::rule installed::1
cla .uninstall kind::rule name::<same>
cla .list kind::rule installed::1
```

Verify:
- First `.list` shows `○` for the rule
- After `.install`: symlink exists at `.claude/rules/<name>.md`, points to `$PRO_CLAUDE/rules/<name>.md`
- After `.uninstall`: symlink is gone, `.list installed::1` shows nothing for that rule

### M-02: Full install/list/uninstall cycle (directory-layout kind)

```
cla .install kind::skill name::<pick-a-real-skill>
ls -la .claude/skills/<name>
cla .uninstall kind::skill name::<same>
```

Verify:
- Symlink target is a directory in `$PRO_CLAUDE/skills/<name>/`
- After uninstall, the symlink is removed

### M-03: Data-loss guard — regular file in target

```
mkdir -p .claude/rules/
echo "not a symlink" > .claude/rules/fake.md
cla .uninstall kind::rule name::fake
```

Verify: Exit 2, error says "not a symlink — refusing to remove (data-loss guard)".
The regular file is NOT deleted.

### M-04: All 6 kinds shown in .kinds

```
cla .kinds
```

Verify: Output lists all 6 kinds (rule, command, agent, skill, plugin, hook) with
correct source and target path mappings reflecting `$PRO_CLAUDE`.

### M-05: Idempotent reinstall

```
cla .install kind::rule name::<name>
cla .install kind::rule name::<name>
```

Verify: Second call exits 0, says "Reinstalled", symlink still points correctly.

### Expected Outcome

All steps succeed without panics. No regular files are ever deleted.
Symlinks always point to the real source in `$PRO_CLAUDE`.
