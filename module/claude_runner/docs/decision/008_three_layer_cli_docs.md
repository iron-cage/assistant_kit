# Decision: Three-Layer CLI Docs

**ID:** D8 · **Category:** Documentation · **Status:** ✅ Adopted

### Scope

- **Purpose**: Record why the flat 42-file `docs/cli/` was replaced by a three-layer reference rather than edited in place.
- **Responsibility**: Rationale for the `command/` + `param/` + `type/` structure and for the later L4 test-doc extension.
- **In Scope**: What the previous structure was, what replaced it, and the two passes that produced the current shape.
- **Out of Scope**: The reference content itself (→ [`../cli/readme.md`](../cli/readme.md)); test case specifications (→ `../../tests/docs/cli/readme.md`).

### Decision

The previous `docs/cli/` contained 42 files documenting `param::value` syntax. It was restored as a proper three-layer reference — `command/`, `param/`, `type/` — with parameter groups, a dictionary, and user stories, adapted to the new `--flag value` syntax.

### Rationale

The old tree documented a syntax the CLI no longer had. Editing 42 files in place would have carried the flat structure forward along with the corrections, and the flat structure was itself the problem: a parameter's type, its per-command availability, and its group membership had no place to live except restated inside every command's file. Three layers give each of those exactly one home.

### Consequence

Two passes produced the current shape:

1. **The redesign pass** — 42 flat files replaced by `command/`, `param/`, `type/`, plus parameter groups, the dictionary, and user stories. The user-story layer was originally `workflow_scenario.md` and was migrated to `user_story/` in a subsequent pass.
2. **The L4 pass** — `tests/docs/cli/` added, with per-command, per-param, per-type, and per-group test case coverage mirroring the reference layers.

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| entity | [readme.md](readme.md) | Decision collection index |
| cli | [`../cli/readme.md`](../cli/readme.md) | The three-layer reference this decision produced |
| test | `../../tests/docs/cli/readme.md` | L4 test case specifications mirroring the reference layers |
