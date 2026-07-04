# Feature: Scaffold Command

### Scope

- **Purpose**: Document the `.init` command of the `crb`/`runbox` binary that scaffolds container runner integration files in a project directory.
- **Responsibility**: Describe command signature, required and optional parameters, generated file contents per ecosystem, and exit behavior.
- **In Scope**: `.init` command; `image::`, `ecosystem::`, `test_script::` parameters; generated `runbox/runbox`, `runbox/runbox.yml`, `runbox/runbox.dockerfile` file contents; ecosystem variants (rust, nodejs, python, none); idempotency guard.
- **Out of Scope**: Container runner execution internals (governed by the container runner system), unilang pipeline internals.

### Design

**Commands:** `runbox` exposes one command `.init` registered via `register_commands()`:

| Command | Purpose | Required args | Optional args |
|---------|---------|---------------|---------------|
| `.init` | Scaffold container runner integration files in current directory | `image::` | `ecosystem::`, `test_script::` |

**`.init` behavior:** Creates a `runbox/` directory (if absent) inside the current working directory and writes three files:

1. `runbox/runbox` — standard walk-up discovery wrapper script; set to executable (mode 0o755).
2. `runbox/runbox.yml` — project config with `image`, `dockerfile`, `cache_dir`, `workspace_root`, and `test_script` fields.
3. `runbox/runbox.dockerfile` — Dockerfile template appropriate for the specified ecosystem.

**Parameters:**

| Parameter | Kind | Default | Values |
|-----------|------|---------|--------|
| `image::` | required | — | any string (Docker image tag) |
| `ecosystem::` | optional | `none` | `rust`, `nodejs`, `python`, `none` |
| `test_script::` | optional | `verb/test.d/l1` | any path string |

**Ecosystem cache directories:**

| Ecosystem | `cache_dir` value | Base image |
|-----------|-------------------|------------|
| `rust` | `target` | `rust:latest` |
| `nodejs` | `node_modules` | `node:22-slim` |
| `python` | `.venv` | `python:3.12-slim` |
| `none` | `.cache` | `ubuntu:22.04` |

**Idempotency guard:** If `runbox/` already exists at the target path, `.init` prints `error: runbox/ already exists` and exits 1 without writing any file. This prevents accidental overwrites of customized container runner configurations.

**Help rendering:** When `needs_help` is true (empty argv, `.help`, `--help`, `-h`), `print_usage()` renders grouped command output via `cli_fmt::CliHelpTemplate` to stdout and exits 0.

**Dual binary:** Both `runbox` and the `crb` alias binary call `run_cli()` from `lib.rs` — the CLI function is compiled once, not twice.

**Acceptance Criteria:**

| ID | Criterion |
|----|-----------|
| AC-001 | `.init` without `image::` exits 1 and prints `error: missing required argument: image::` |
| AC-002 | `ecosystem::` with unknown value exits 1 and prints `error: unknown ecosystem: <value>` |
| AC-003 | `.init image::my_img` creates `runbox/` directory, `runbox/runbox`, `runbox/runbox.yml`, `runbox/runbox.dockerfile` |
| AC-004 | Generated `runbox/runbox` contains the walk-up discovery wrapper script verbatim |
| AC-005 | `runbox/runbox` is created with executable permission (mode 0o755) |
| AC-006 | Generated `runbox/runbox.yml` contains `image`, `dockerfile`, `cache_dir`, `workspace_root`, `test_script` fields |
| AC-007 | `cache_dir` in generated `runbox.yml` is `target` for rust, `node_modules` for nodejs, `.venv` for python, `.cache` for none |
| AC-008 | `test_script` in generated `runbox.yml` defaults to `verb/test.d/l1` when `test_script::` not provided |
| AC-009 | `test_script::custom/path` overrides the default in the generated `runbox.yml` |
| AC-010 | `.init` exits 1 with `error: runbox/ already exists` when `runbox/` directory already exists |
| AC-011 | `--help`, `-h`, `.help`, and empty argv print usage to stdout and exit 0 |
| AC-012 | Both `crb` and `runbox` binaries invoke the same `run_cli()` entry point |

### Cross-References

| Type | File | Responsibility |
|------|------|----------------|
| source | `src/lib.rs` | `register_commands()`, `run_cli()` |
| source | `src/commands.rs` | `init_routine` — file generation logic |
| source | `src/templates.rs` | Ecosystem-specific file content templates |
| source | `unilang.commands.yaml` | CLI command metadata (names, arguments, examples) |
| doc | [`feature/001_workspace_design.md`](../../../../docs/feature/001_workspace_design.md) | Workspace crate inventory (runbox member) |
