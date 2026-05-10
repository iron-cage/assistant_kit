# verb

Per-verb reference for the `do` Universal Action Protocol.

### Scope

- **Purpose:** Document every verb in the `do` protocol implemented across workspace modules.
- **Responsibility:** Per-instance reference covering kind, availability, `--dry-run` contract, command, and notes.
- **In Scope:** All 7 protocol verbs (5 canonical + 2 meta); their contracts and module-level variation.
- **Out of Scope:** Verb script implementation (→ `module/*/verb/`); runbox integration (→ `docs/runbox/`).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `procedure.md` | Add a verb/ dir to a module; update a verb command |
| `001_build.md` | `build` verb: compile project artifacts |
| `002_test.md` | `test` verb: run the full test suite |
| `003_clean.md` | `clean` verb: remove generated artifacts |
| `004_run.md` | `run` verb: execute the project entry point |
| `005_lint.md` | `lint` verb: static analysis and style checking |
| `006_verbs.md` | `verbs` meta-verb: list available verbs |
| `007_detect.md` | `detect` meta-verb: identify the project ecosystem |

### Overview Table

Canonical verbs implement project actions. Meta verbs implement protocol introspection.

| ID | Verb | Kind | Availability | `--dry-run` |
|----|------|------|--------------|-------------|
| [001](001_build.md) | `build` | canonical | universal | yes |
| [002](002_test.md) | `test` | canonical | universal | yes |
| [003](003_clean.md) | `clean` | canonical | universal | yes |
| [004](004_run.md) | `run` | canonical | binary-only | yes |
| [005](005_lint.md) | `lint` | canonical | universal | yes |
| [006](006_verbs.md) | `verbs` | meta | universal | — |
| [007](007_detect.md) | `detect` | meta | universal | — |

**Availability:** `universal` = present and functional in all modules; `binary-only` = functional only in modules with a binary entry point; library modules exit 3 (unavailable).
