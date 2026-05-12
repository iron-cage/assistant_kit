# Runbox Onboarding

How to integrate runbox test isolation into a new project. One `run/` directory, a handful of scripts, any ecosystem.

### Prerequisites

- `runbox-run` must be reachable by walking up from the project's `run/` directory (i.e. the project lives somewhere inside a directory tree that contains `run/runbox-run`).
- Docker or Podman installed on the host.

---

### Step 1 — `run/runbox` (copy verbatim, no edits)

This wrapper is identical for every project at any directory depth. The walk-up discovery finds `runbox-run` automatically.

```bash
#!/usr/bin/env bash
# runbox wrapper — auto-discovers runbox-run by walking up the directory tree.
# Copy verbatim to any project's run/ directory; no path calculation needed.
set -euo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

_find_runbox_run() {
  local dir="$1"
  while [[ "$dir" != "/" ]]; do
    [[ -x "$dir/run/runbox-run" ]] && echo "$dir/run/runbox-run" && return
    dir="$( dirname "$dir" )"
  done
  echo "error: runbox-run not found in any parent directory" >&2
  exit 1
}

exec "$(_find_runbox_run "$SCRIPT_DIR")" "$SCRIPT_DIR/runbox.yml" "$@"
```

---

### Step 2 — `run/runbox.yml` (project-specific config)

```yaml
# Unique Docker image tag for this project.
image: my_project_test

# Dockerfile path — resolved relative to this config file's directory.
dockerfile: runbox.dockerfile

# Build artifact directory seeded into a named Docker volume.
# Python: .venv  |  Node: node_modules  |  Rust: target
cache_dir: .venv

# Project root relative to this config file's directory. Almost always "..".
workspace_root: ..

# Script path (relative to /workspace) executed by .test for online tests.
test_script: run/test

# Optional: script path executed by .lint.
lint_script: run/lint

# Optional: script path executed by .run.
run_script: run/run
```

→ Full parameter reference: `run/docs/parameter/`

---

### Step 3 — `run/runbox.dockerfile` (ecosystem container image)

The dockerfile must:
- Install dependencies into `cache_dir` so the volume seed has them.
- Create `{cache_dir}_seed/` as an empty directory (used as mount point for seeding).
- Set `CMD` to run offline tests without arguments.

Python example:

```dockerfile
FROM python:3.12-slim

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

# Install into .venv so the cache_dir volume contains pre-installed packages.
RUN python -m venv .venv && .venv/bin/pip install --no-cache-dir .[dev]

# Seed mount point for build cache persistence.
RUN mkdir .venv_seed && chmod -R a+rwX .venv .venv_seed

CMD [".venv/bin/pytest", "tests/", "-v"]
```

---

### Step 4 — `run/plugins.sh` (test lister override)

Override `_plugin_list_cmd` to use your ecosystem's test discovery command. Always use absolute paths (`/workspace/...`) inside the container.

```bash
#!/usr/bin/env bash
# Sourced by runbox-run after the core plugins — overrides only what differs.

_plugin_list_cmd() {
  list_cmd="/workspace/.venv/bin/pytest --collect-only -q /workspace/tests/"
}
```

→ Full plugin reference: `run/docs/plugin/`

---

### Step 5 — `run/test` (online test script)

Executed inside the container by `.test`. Use `$SCRIPT_DIR`-relative paths — inside the
container `SCRIPT_DIR` resolves to `/workspace/run`, so `$SCRIPT_DIR/..` is `/workspace`.
This also allows calling the script directly on the host when local dev tools are available.

```bash
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../.venv/bin/pytest" "$SCRIPT_DIR/../tests/" -v
```

Not needed for `.test.offline` — that command uses the baked image `CMD` directly.

---

### Step 6 — `run/lint` and `run/run` (optional)

Add these when you want `.lint` and `.run` commands. Use `$SCRIPT_DIR`-relative paths.

```bash
# run/lint
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../.venv/bin/ruff" check --no-cache "$SCRIPT_DIR/../src/" "$SCRIPT_DIR/../tests/"

# run/run
#!/usr/bin/env bash
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../.venv/bin/python" -m my_package
```

---

### Finalize

```bash
chmod +x run/runbox run/test run/lint run/run
```

### Usage

```bash
./run/runbox .build          # build the container image
./run/runbox .run            # run the entry point (requires run_script)
./run/runbox .test           # online tests (requires mounted credentials if configured)
./run/runbox .test.offline   # offline tests using seeded cache volume
./run/runbox .lint           # run linter (requires lint_script)
./run/runbox .list           # list tests via plugins.sh _plugin_list_cmd
./run/runbox .shell          # interactive shell with cache volume mounted
```

---

### Multi-Layer Verbs

A verb can define separate implementations for different execution layers. This is the mechanism for expressing that the same logical operation — "test this module" — means different things depending on where it runs: on the host, it means "orchestrate Docker"; inside the container, it means "run the test suite directly."

**Two verb forms:**

| Form | Structure | Layer-aware? |
|------|-----------|-------------|
| Flat file | `verb/X` (executable) | No — `VERB_LAYER` ignored; runs as-is everywhere |
| Directory | `verb/X/` (directory of executables) | Yes — dispatches to layer-specific file |

Most verbs are flat. Use the directory form only when a verb genuinely has different behavior at different execution layers.

---

**Directory structure:**

```
verb/
  test/
    default     # REQUIRED — entry point when VERB_LAYER is not set
    l1          # innermost: bare execution (direct test runner call)
    l2          # next: orchestration wrapper (delegates to runbox or equivalent)
    l3          # (optional) further wrapping — CI runner, remote host, etc.
```

**Layer naming is positional, not semantic.** `l1` = most direct execution (no wrappers). Higher numbers = more orchestration layers around it. What each layer does is documented inside the layer file itself, not in its name.

---

**`VERB_LAYER` — the layer identity convention:**

`runbox-run` sets `VERB_LAYER=l1` before invoking any verb inside the container. On the host with no explicit layer, `VERB_LAYER` is unset. Verbs never detect their environment — they are told their layer by whoever invokes them. Layer boundary crossers (runbox, CI runners, SSH executors) are solely responsible for setting this variable.

---

**`default` — the developer entry point:**

`default` is invoked when `VERB_LAYER` is not set (direct host invocation). It delegates to whichever layer is the natural developer entry point — often `l2`, not `l1`. The innermost layer (`l1`) is too bare to be the default: on the host it would run tests without Docker isolation. `l2` (Docker-orchestrated) is reproducible and safe. `default` makes that choice explicit:

```bash
#!/usr/bin/env bash
# verb/test/default — delegates to l2 (Docker-orchestrated; developer-standard entry point)
exec "$(dirname "$0")/l2" "$@"
```

Because `default` always exists and `runbox-run` always sets `VERB_LAYER=l1` inside the container, the two layers never collide.

---

**Dispatch — `run/verb-run`:**

A universal dispatcher `run/verb-run` resolves the verb form and selects the right implementation:

```
resolve(verb_name, VERB_LAYER):
  if verb/X is a FILE  →  exec directly (flat form; VERB_LAYER ignored)
  if verb/X/ is a DIR  →  exec verb/X/$VERB_LAYER if set and exists
                        →  else exec verb/X/default
```

Callers use `verb-run test` instead of `bash verb/test` directly. `runbox-run` passes `VERB_LAYER=l1` and points `test_script` at `verb/test` (the directory); `verb-run` resolves to `verb/test/l1` inside the container.

---

**Example — a module using Docker for test isolation:**

```
verb/
  test/
    default   → exec "$(dirname "$0")/l2" "$@"
    l1        → exec w3 .test level::3          # bare execution inside container
    l2        → exec bash run/runbox .test       # host orchestration via Docker
  lint        → exec cargo clippy ...            # flat: same everywhere
```

Invocation flows:

```
Host (VERB_LAYER unset):
  verb-run test → verb/test/default → l2 → bash run/runbox .test
    runbox sets VERB_LAYER=l1 → verb-run test inside container
      → verb/test/l1 → w3 .test level::3 → nextest ✓

Host (VERB_LAYER unset):
  verb-run lint → verb/lint (flat file) → cargo clippy ✓

Container (VERB_LAYER=l1, set by runbox):
  verb-run test → verb/test/l1 → w3 .test level::3 ✓
```

---

**Configuration in `runbox.yml`:**

With multi-layer verbs, point `test_script` (and `lint_script`, `run_script`) at the verb directory. `runbox-run` detects the directory form and resolves to the `l1` implementation automatically:

```yaml
# Points at the directory; runbox-run resolves to verb/test/l1 inside container
test_script: module/my_module/verb/test
```

No change needed in `runbox.yml` format — the path is the same whether `verb/test` is a file or a directory.

---

**Key policies:**

1. Flat file verbs never need updating — they remain valid for single-behavior operations.
2. `default` is mandatory in every directory verb. Missing `default` is a protocol violation.
3. Configuration controlling verb behavior lives at the verb's layer — not at the orchestration layer. (A nextest filter belongs in `nextest.toml`, not in `runbox.yml cmd_filter`.)
4. `--dry-run` propagates through layer delegation — each layer file honors it.
