#!/usr/bin/env bash
#! test_kind: bug_reproducer(issue-runbox-build-safety)
#
# runbox_build_safety_test.sh — regression tests for two concurrent-build bugs.
#
# ## Root Cause
#
# _build() in runbox-run has two independent defects:
#
# 1. No concurrency lock: Multiple simultaneous invocations of `runbox .build`
#    or any command that calls `_ensure_image` (test, lint, list, shell) can all
#    spawn independent `podman build` processes concurrently. In production this
#    produced 7 simultaneous podman processes competing for disk, oscillating
#    between 9.9 G and 54 G free.
#
# 2. No post-build verification: After `podman build` exits 0, _build() returns
#    immediately without confirming `podman image inspect` works. When storage.conf
#    graphroot is migrated mid-build (triggered by the concurrent chaos), the image
#    is orphaned in the old graphroot; subsequent `image inspect` fails with
#    "image not known" even though BUILD COMPLETE was printed.
#
# ## Why Not Caught Initially
#
# The runbox-run script was designed for serial single-user invocation. No test
# asserted that _build() acquires a mutual-exclusion lock, nor that a successful
# `podman build` exit code guarantees image accessibility.
#
# ## Fix Applied
#
# 1. _build() acquires an atomic `mkdir` lock on /tmp/runbox_build_<IMAGE>.lock
#    before invoking the container engine. Concurrent callers wait (up to 5 min)
#    then proceed once the lock is released.
# 2. _build() calls `image inspect` after `podman build` exits 0; if inspect
#    fails, _build() exits 1 with a diagnostic pointing to the graphroot issue.
#
# ## Prevention
#
# Any future command in runbox-run that wraps `$CONTAINER_CMD build` must acquire
# the same per-IMAGE lock and perform the same post-build inspect verification.
#
# ## Pitfall to Avoid
#
# `podman build` exit 0 only means the build process itself succeeded; it does NOT
# guarantee the image is accessible in the current storage backend. Always follow
# with `image inspect` before returning to callers.
#
# Usage: bash run/tests/runbox_build_safety_test.sh

set -uo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
RUNBOX_RUN="$( cd "$SCRIPT_DIR/.." && pwd )/runbox-run"

PASS_COUNT=0
FAIL_COUNT=0

_pass() { echo "  PASS: $1"; (( PASS_COUNT++ )); }
_fail() { echo "  FAIL: $1"; (( FAIL_COUNT++ )); }

# ── Shared helper: create a minimal runbox.yml ────────────────────────────────

_make_config()
{
  local dir="$1"
  local image="$2"
  cat > "$dir/runbox.yml" << EOF
image: $image
test_user: testuser
cmd_scope: --workspace
cmd_filter: ""
test_script: verb/test.d/l1
EOF
}

# ── T1: post-build image not accessible → _build() must exit non-zero ─────────
#
# Simulates the graphroot-migration scenario: `podman build` exits 0, but the
# image was written to the old (now-unreachable) graphroot, so `image inspect`
# fails immediately after. This is the exact failure mode that produced the
# "image not known" crisis.
#
# Before fix: _build() exits 0 despite inaccessible image → test FAILS.
# After fix:  _build() exits non-zero → test PASSES.
# test_kind: bug_reproducer(issue-runbox-build-safety)

test_t1_build_exits_nonzero_when_image_inaccessible_after_build()
{
  local tmp
  tmp="$(mktemp -d)"

  local mock_dir="$tmp/bin"
  mkdir "$mock_dir"

  # Mock container runtime:
  #   - "build" exits 0 (build "succeeds")
  #   - "image inspect" exits 1 (image not accessible — graphroot migrated)
  cat > "$mock_dir/podman" << 'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  container) exit 0 ;;
  build)     exit 0 ;;
  image)
    [[ "${2:-}" == "inspect" ]] && exit 1
    exit 0
    ;;
esac
EOF
  chmod +x "$mock_dir/podman"

  _make_config "$tmp" "t1_safety_test_image"

  local exit_code=0
  PATH="$mock_dir:$PATH" bash "$RUNBOX_RUN" "$tmp/runbox.yml" .build \
    2>/dev/null || exit_code=$?

  if [[ "$exit_code" -ne 0 ]]; then
    _pass "T1: _build() exits non-zero when image not accessible after build"
  else
    _fail "T1: _build() exits non-zero when image not accessible after build (got exit 0 — post-build verify missing)"
  fi

  rm -rf "$tmp"
}

# ── T2: build lock directory exists while _build() is running ─────────────────
#
# The mock `podman build` checks whether the per-image lock directory exists at
# the moment it runs. Without the lock, _build() invokes `podman build` before
# creating any lock → mock does not see the directory → lock_observed flag is
# absent → test FAILS. With the lock, _build() first creates the directory, then
# calls `podman build` → mock sees it → test PASSES.
#
# Before fix: no lock directory → test FAILS.
# After fix:  lock directory created before podman build call → test PASSES.
# test_kind: bug_reproducer(issue-runbox-build-safety)

test_t2_build_lock_directory_exists_during_build()
{
  local tmp
  tmp="$(mktemp -d)"

  local mock_dir="$tmp/bin"
  mkdir "$mock_dir"

  local image_name="t2_lock_safety_test_image"
  # The fix computes the lock dir as: /tmp/runbox_build_<IMAGE with special-chars→_>.lock
  local lockdir="/tmp/runbox_build_${image_name//[^a-zA-Z0-9_-]/_}.lock"
  local lock_observed="$tmp/lock_observed"
  local build_started="$tmp/started"
  local proceed_flag="$tmp/proceed"

  # Mock: during "build", observe whether the lock dir exists, then wait for release signal.
  cat > "$mock_dir/podman" << EOF
#!/usr/bin/env bash
case "\${1:-}" in
  container) exit 0 ;;
  build)
    touch "${build_started}"
    [[ -d "${lockdir}" ]] && touch "${lock_observed}"
    while [[ ! -f "${proceed_flag}" ]]; do sleep 0.05; done
    exit 0
    ;;
  image)
    [[ "\${2:-}" == "inspect" ]] && exit 0
    ;;
esac
EOF
  chmod +x "$mock_dir/podman"

  _make_config "$tmp" "$image_name"

  # Launch build in background
  PATH="$mock_dir:$PATH" bash "$RUNBOX_RUN" "$tmp/runbox.yml" .build \
    2>/dev/null &
  local build_pid=$!

  # Wait for mock to signal it has started
  local iters=60
  while [[ ! -f "$build_started" ]] && (( iters-- > 0 )); do
    sleep 0.1
  done

  # Signal mock to finish building
  touch "$proceed_flag"
  wait "$build_pid" 2>/dev/null || true

  if [[ -f "$lock_observed" ]]; then
    _pass "T2: lock directory exists while podman build runs (prevents concurrent builds)"
  else
    _fail "T2: lock directory exists while podman build runs (got no lock dir — concurrent-build guard missing)"
  fi

  rm -rf "$tmp"
  rm -rf "$lockdir" 2>/dev/null || true
}

# ── Runner ────────────────────────────────────────────────────────────────────

echo ""
echo "── runbox build safety tests ─────────────────────────────────────────"
test_t1_build_exits_nonzero_when_image_inaccessible_after_build
test_t2_build_lock_directory_exists_during_build
echo ""
echo "  Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed"
echo "──────────────────────────────────────────────────────────────────────"
echo ""

[[ "${FAIL_COUNT}" -eq 0 ]]
