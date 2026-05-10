# Runbox Onboarding

How to integrate runbox test isolation into a new project. One `run/` directory, five files, any ecosystem.

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
```

→ Full parameter reference: `run/docs/parameter/`

---

### Step 3 — `run/runbox.dockerfile` (ecosystem container image)

The dockerfile must:
- Install dependencies into `cache_dir` so the volume seed has them.
- Create `{cache_dir}_seed/` as an empty directory (Docker initialises the volume from it).
- Set `CMD` to run offline tests without arguments.

Python example:

```dockerfile
FROM python:3.12-slim

ARG WORKSPACE_DIR=/workspace
WORKDIR $WORKSPACE_DIR

COPY . .

# Install into .venv so the cache_dir volume contains pre-installed packages.
RUN python -m venv .venv && .venv/bin/pip install --no-cache-dir .[dev]

# Seed mount point for build cache persistence plugin.
RUN mkdir .venv_seed

CMD [".venv/bin/pytest", "tests/", "-v"]
```

---

### Step 4 — `run/plugins.sh` (test lister override)

Override `_plugin_list_cmd` to use your ecosystem's test discovery command.

```bash
#!/usr/bin/env bash
# Sourced by runbox-run after the core plugins — overrides only what differs.

_plugin_list_cmd() {
  list_cmd=".venv/bin/pytest --collect-only -q tests/"
}
```

→ Full plugin reference: `run/docs/plugin/`

---

### Step 5 — `run/test` (online test script)

Executed inside the container by `.test`. Use absolute paths (`/workspace/...`).

```bash
#!/usr/bin/env bash
exec /workspace/.venv/bin/pytest /workspace/tests/ -v
```

Not needed for `.test.offline` — that command uses the baked image `CMD` directly.

---

### Finalize

```bash
chmod +x run/runbox run/test run/plugins.sh
```

### Usage

```bash
./run/runbox .build          # build the container image
./run/runbox .test           # online tests (requires mounted credentials if configured)
./run/runbox .test.offline   # offline tests using seeded .venv volume
./run/runbox .list           # list tests via plugins.sh _plugin_list_cmd
./run/runbox .shell          # interactive shell with cache volume mounted
```
