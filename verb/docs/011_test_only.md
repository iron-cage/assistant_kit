# Verb: `test_only`

- **Kind:** canonical
- **Availability:** module scope only, universal across all 24 modules (workspace root uses [`test1`](009_test1.md))
- **`--dry-run`:** `runbox .live -- ./module/<name>/verb/test_only.d/l1 <filter>`

### Command

```bash
./verb/test_only <nextest-filter>
```

Runs only the matching tests inside the container: the globally-installed `runbox` engine executes the module's own `verb/test_only.d/l1` with the filter forwarded as a positional argument. The layer runs `cargo nextest run --all-features "$@"`, cwd-scoped to the module directory so the run never widens to the whole workspace. No doc tests and no clippy — use [`test`](002_test.md) for the module's full suite.

### Notes

`test_only` is the **mandatory** choice for targeted verification at module scope. Running `./verb/test` (full suite) to check a single test wastes the full nextest + doc-test + clippy cycle. The container-only invariant (`module/claude_profile/docs/invariant/009_container_only_test_execution.md`) applies to both forms.

The filter is a **positional name substring**, not the `-E` filter-expression syntax that workspace-scope [`test1`](009_test1.md) takes. `cargo nextest run --all-features <substring>` matches on test name; there is no `-E` involved and no environment variable — the argument travels straight through the payload to nextest.

`test_only` and `test1` are the same capability at two scopes, and neither exists at the other's scope: modules have `test_only` and no `test1`; the workspace root has `test1` and no `test_only`. The scope difference drives the flag difference — `test1` passes `--workspace --no-fail-fast -E "<filter>"` because it must select across every crate, while `test_only` relies on the module's own working directory to bound the run.

Two guard conditions exit 1 before any container work:
- **No filter given** — the verb refuses to run rather than silently falling back to the full suite (`ERROR: test_only requires a test name filter.`).
- **`VERB_LAYER` set on the host** — that variable belongs to the container side only; setting it host-side is a misinvocation (`ERROR: VERB_LAYER is not valid on the host side.`).

The `l1` payload exports `CARGO_NET_OFFLINE=true`, `NO_COLOR=1`, `RUNBOX_CONTAINER=1`, and `RUSTFLAGS="-D warnings"`. `CARGO_TARGET_DIR` is supplied by the engine (persistent working volume) and overrides the host `.cargo/config.toml` target-dir redirect inside the container.

`--dry-run` (first argument) prints the delegated `runbox .live` command and exits 0 — no tests run, container not started.

### Example

```bash
cd module/claude_storage_core

# Run every test whose name contains the substring:
./verb/test_only encode_path

# Dry run (prints the delegated command, no execution):
./verb/test_only --dry-run encode_path
# → runbox .live -- ./module/claude_storage_core/verb/test_only.d/l1 encode_path

# Missing filter — exits 1, does not fall back to the full suite:
./verb/test_only
# → ERROR: test_only requires a test name filter.
```

Scope comparison with [`test1`](009_test1.md):
```bash
# module scope — positional substring
cd module/claude_storage && ./verb/test_only session_stats

# workspace scope — nextest -E filter expression
./verb/test1 'package(claude_storage)'
```
