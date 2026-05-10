# verb

Per-verb reference for the `do` Universal Action Protocol.

### Scope

- **Purpose:** Document every verb in the `do` protocol implemented across workspace modules.
- **Responsibility:** Per-instance reference covering kind, availability, `--dry-run` contract, command, and notes.
- **In Scope:** All 8 protocol verbs (6 canonical + 2 meta); their contracts and module-level variation.
- **Out of Scope:** Verb script implementation (→ `module/*/verb/`); runbox integration (→ `run/docs/`).

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
| `008_verify.md` | `verify` verb: full checks including dependency analysis and audit |

### Overview Table

Canonical verbs implement project actions. Meta verbs implement protocol introspection.

| ID | Verb | Kind | Availability | `--dry-run` | Purpose |
|----|------|------|--------------|-------------|---------|
| [001](001_build.md) | `build` | canonical | universal | yes | Compile project artifacts |
| [002](002_test.md) | `test` | canonical | universal | yes | Run the full test suite |
| [003](003_clean.md) | `clean` | canonical | universal | yes | Remove generated build artifacts |
| [004](004_run.md) | `run` | canonical | binary-only | yes | Execute the project binary |
| [005](005_lint.md) | `lint` | canonical | universal | yes | Static analysis, warnings as errors |
| [008](008_verify.md) | `verify` | canonical | universal | yes | Full checks: tests, deps, audit |
| [006](006_verbs.md) | `verbs` | meta | universal | — | List all verbs with status |
| [007](007_detect.md) | `detect` | meta | universal | — | Report project ecosystem and confidence |

**Availability:** `universal` = present and functional in all modules; `binary-only` = functional only in modules with a binary entry point; library modules exit 3 (unavailable).
