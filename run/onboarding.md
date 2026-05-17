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
test_script: verb/test

# Optional: script path executed by .lint.
lint_script: verb/lint

# Optional: script path executed by .run.
run_script: verb/run

# Optional: extra named build context passed as --build-context to the container build.
# Format: name=relpath  (relpath resolved relative to this config file's directory)
# Use when a Dockerfile stage references FROM <name> pointing outside WORKSPACE_ROOT.
# extra_build_context: shared=../../shared
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

### Step 5 — `verb/test`, `verb/test.d/l0`, `verb/test.d/l1`

`verb/test` is a self-dispatching dispatcher. When `VERB_LAYER=l1` (set by `runbox-run`
before entering the container), it routes to `verb/test.d/l1`; invoked directly on the
host it falls through to `l0`.

```bash
# verb/test — dispatcher (identical for all ecosystems)
#!/usr/bin/env bash
# test — run test suite; dispatches by VERB_LAYER to test.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/test.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l0" "$@"
```

`verb/test.d/l0` is the default host-native layer — runs tests directly without Docker:

```bash
# verb/test.d/l0 — host-native layer (default; no Docker, no runbox)
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/pytest" "$SCRIPT_DIR/../../tests/" -v
```

`verb/test.d/l1` runs inside the container. Inside the container `SCRIPT_DIR`
resolves to `/workspace/verb/test.d`, so `$SCRIPT_DIR/../..` is `/workspace`.

```bash
# verb/test.d/l1 — container execution layer (Python example)
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/pytest" "$SCRIPT_DIR/../../tests/" -v
```

`verb/` has no knowledge of Docker or `run/`. The Docker path is owned entirely by `run/runbox`: it sets `VERB_LAYER=l1`, calls `verb/test.d/l1` directly inside the container, and returns results. `verb/test` never references `run/`.

Not needed for `.test.offline` — that command uses the baked image `CMD` directly.

---

### Step 6 — `verb/lint` and `verb/run` (optional)

Add these when you want `.lint` and `.run` commands. Both follow the same dispatcher + `.d/`
pattern as `verb/test`. Set `lint_script: verb/lint` and `run_script: verb/run` in `runbox.yml`.

```bash
# verb/lint — dispatcher (identical for all ecosystems)
#!/usr/bin/env bash
# lint — run linter; dispatches by VERB_LAYER to lint.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/lint.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l1" "$@"

# verb/lint.d/l1 — container execution layer (Python example)
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/ruff" check --no-cache "$SCRIPT_DIR/../../src/" "$SCRIPT_DIR/../../tests/"

# verb/run — dispatcher (same structure as verb/lint, but for "run.d/")
# verb/run.d/l1 — container execution layer (Python example)
#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
exec "$SCRIPT_DIR/../../.venv/bin/python" -m my_package
```

Like `verb/test`, the lint and run dispatchers default to `l1` — direct execution without Docker.

---

### Finalize

```bash
chmod +x run/runbox run/verb-run
chmod +x verb/test verb/test.d/l0 verb/test.d/l1
chmod +x verb/lint verb/lint.d/l1  # if using lint
chmod +x verb/run  verb/run.d/l1   # if using run
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

A verb can define separate implementations for different execution layers. `verb/` has no knowledge of Docker or `run/` — it only knows two contexts: host-native (l0, default) and container-internal (l1, set by whoever invokes the container). `run/runbox` orchestrates the Docker path independently, calling `verb/test.d/l1` directly inside the container without going through the dispatcher.

**Verb structure — dispatcher + `.d/` layers:**

```
verb/
  test          ← dispatcher file (always executable; reads VERB_LAYER, self-dispatches)
  test.d/
    l0          # default: host-native direct execution (no Docker)
    l1          # container-internal: set by runbox-run via VERB_LAYER=l1
  lint          ← flat file (no .d/ directory; same behavior everywhere)
```

`verb/test` is always a regular executable file — never a directory. The layers live in the adjacent `verb/test.d/` directory. Callers always invoke `bash verb/test` directly; the dispatcher selects the appropriate layer internally.

**Layer naming is positional, not semantic.** `l0` = host-native direct execution (default, no Docker). `l1` = container-internal execution (set by runbox-run). What each layer does is documented inside the layer file itself, not in its name.

---

**`VERB_LAYER` — the layer identity convention:**

`runbox-run` sets `VERB_LAYER=l1` before invoking any verb inside the container. On the host with no explicit layer, `VERB_LAYER` is unset. Verbs never detect their environment — they are told their layer by whoever invokes them. Layer boundary crossers (runbox, CI runners, SSH executors) are solely responsible for setting this variable.

---

**Dispatcher implementation:**

Each multi-layer verb file reads `VERB_LAYER` and self-dispatches to the correct layer file in its `.d/` directory. The default layer (when `VERB_LAYER` is unset) is encoded directly in the dispatcher:

```bash
#!/usr/bin/env bash
# test — run full test suite; dispatches by VERB_LAYER to test.d/ layer.
set -euo pipefail
DIR="$(dirname "${BASH_SOURCE[0]}")/test.d"
LAYER="${VERB_LAYER:-}"
[[ -n "$LAYER" && -f "$DIR/$LAYER" ]] && exec "$DIR/$LAYER" "$@"
exec "$DIR/l0" "$@"   # default → l0 (host-native; developer-standard entry point)
```

The last `exec` line encodes the default: `l0` for modules with host-native and container layers, `l1` for simple modules that use a single execution layer. No separate `default` file is needed.

---

**Dispatch — `run/verb-run`:**

`run/verb-run` is simplified under the `.d/` convention. Since `verb/X` is always a file, dispatch is always direct:

```
resolve(verb_name):
  if verb/X is a FILE  →  exec directly (VERB_LAYER already in environment)
  else                 →  error
```

Callers use `verb-run test` or `bash verb/test` — both work identically since `verb/test` is always a file. `runbox-run` sets `VERB_LAYER=l1` and points `test_script` at `verb/test`; the dispatcher inside routes to `test.d/l1`.

---

**Example — a module with host-native and container layers:**

```
verb/
  test          → dispatcher (default → l0)
  test.d/
    l0          → exec w3 .test level::3          # host-native (default)
    l1          → exec w3 .test level::3          # container-internal (VERB_LAYER=l1)
  lint          → exec cargo clippy ...            # flat: same everywhere
```

Invocation flows:

```
Host (VERB_LAYER unset):
  bash verb/test → dispatcher → test.d/l0 → w3 .test level::3 → nextest ✓

Host (VERB_LAYER unset):
  bash verb/lint → verb/lint (flat file) → cargo clippy ✓

Container (VERB_LAYER=l1, set by runbox):
  bash verb/test → dispatcher → test.d/l1 → w3 .test level::3 ✓

Via Docker (run/runbox .test — run/ is fully independent of verb/):
  ./run/runbox .test → docker run ... verb/test.d/l1 directly → w3 .test level::3 ✓
```

---

**Configuration in `runbox.yml`:**

Point `test_script` (and `lint_script`, `run_script`) at the verb file. Since `verb/test` is always a file, `runbox-run` uses `test -f` throughout — no directory detection needed:

```yaml
# Points at the verb file; executed with VERB_LAYER=l1 set inside the container
test_script: module/my_module/verb/test
```

---

**Key policies:**

1. Flat file verbs remain flat — no `.d/` directory needed for single-behavior operations.
2. `verb/X` is always a file. Layers always live in `verb/X.d/`. Never make `verb/X` a directory.
3. The default layer is encoded in the dispatcher's final `exec` line — no separate `default` file.
4. Configuration controlling verb behavior lives at the verb's layer — not at the orchestration layer. (A nextest filter belongs in `nextest.toml`, not in `runbox.yml cmd_filter`.)
5. `--dry-run` propagates through layer delegation — each layer file honors it.
