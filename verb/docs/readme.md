# verb

Per-verb reference for the `do` Universal Action Protocol.

### Scope

- **Purpose:** Document every verb in the `do` protocol implemented across workspace modules.
- **Responsibility:** Per-instance reference covering kind, availability, `--dry-run` contract, command, and notes.
- **In Scope:** All 11 documented protocol verbs (9 canonical + 2 meta); their contracts, module-level variation, and workspace-level behavior.
- **Out of Scope:** Verb script implementation (→ `module/*/verb/` for modules, `verb/` for workspace).

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `procedure.md` | Add a verb/ dir to a module or workspace; update a verb command |
| [001_build.md](001_build.md) | `build` verb: compile project artifacts |
| [002_test.md](002_test.md) | `test` verb: run the full test suite |
| [003_clean.md](003_clean.md) | `clean` verb: remove generated artifacts |
| [004_run.md](004_run.md) | `run` verb: execute the project entry point |
| [005_lint.md](005_lint.md) | `lint` verb: static analysis and style checking |
| [006_verbs.md](006_verbs.md) | `verbs` meta-verb: list available verbs |
| [007_package_info.md](007_package_info.md) | `package_info` meta-verb: report deterministic package metadata as JSON |
| [008_verify.md](008_verify.md) | `verify` verb: full checks including dependency analysis and audit |
| [009_test1.md](009_test1.md) | `test1` verb: run a single nextest filter inside container |
| [010_install.md](010_install.md) | `install` verb: install module binaries to `~/.cargo/bin` |
| [011_test_only.md](011_test_only.md) | `test_only` verb: run a targeted nextest filter at module scope |

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
| [009](009_test1.md) | `test1` | canonical | workspace-only | yes | Targeted nextest filter in container |
| [010](010_install.md) | `install` | canonical | binary-only, module-only | yes | Install module binaries to `~/.cargo/bin` |
| [011](011_test_only.md) | `test_only` | canonical | module-only | yes | Targeted nextest filter at module scope |
| [006](006_verbs.md) | `verbs` | meta | universal | — | List all verbs with status |
| [007](007_package_info.md) | `package_info` | meta | universal | — | Deterministic flat JSON: name, version, edition, ecosystem |

**Availability:** `universal` = present and functional in all modules and at workspace scope; `binary-only` = functional only in modules with a binary entry point; library modules and workspace scope exit 3 (unavailable) for `run`. `workspace-only` = script exists at the workspace root and not in modules (`test1`); `module-only` = script exists in every module and not at the workspace root (`test_only`, `install`). `test1` and `test_only` are the same capability at their respective scopes, as are the workspace and module forms of `test`.
