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
