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

# ── T3: stale content hash → _ensure_image rebuilds ──────────────────────────
#
# Simulates an image whose runbox.content_hash label is stale (old hash from a
# previous build, before Cargo.toml or runbox.yml was changed on the host).
# _ensure_image() must detect the mismatch via _is_stale() and call _build().
#
# Before staleness detection: _ensure_image() returns without rebuild → FAILS.
# After staleness detection:  _ensure_image() calls _build() → PASSES.

test_t3_stale_hash_triggers_rebuild()
{
  local tmp
  tmp="$(mktemp -d)"

  local mock_dir="$tmp/bin"
  mkdir "$mock_dir"

  local rebuild_triggered="$tmp/rebuild_triggered"

  # Mock: image exists but carries a known-wrong hash.
  # build: touch flag + exit 0; image inspect: return stale hash for --format queries.
  cat > "$mock_dir/podman" << EOF
#!/usr/bin/env bash
case "\${1:-}" in
  container) exit 0 ;;
  build)
    touch "${rebuild_triggered}"
    exit 0
    ;;
  image)
    if [[ "\${2:-}" == "inspect" ]]
    then
      [[ "\${4:-}" == "--format" ]] && echo "000deadbeef000" && exit 0
      exit 0
    fi
    ;;
  volume)
    [[ "\${2:-}" == "inspect" ]] && exit 1  # volume absent — needs seed
    exit 0
    ;;
  run) exit 0 ;;
esac
EOF
  chmod +x "$mock_dir/podman"

  _make_config "$tmp" "t3_stale_hash_test_image"

  PATH="$mock_dir:$PATH" bash "$RUNBOX_RUN" "$tmp/runbox.yml" .test \
    2>/dev/null || true

  if [[ -f "$rebuild_triggered" ]]
  then
    _pass "T3: stale content hash triggers automatic image rebuild"
  else
    _fail "T3: stale content hash triggers automatic image rebuild (no rebuild detected — staleness check missing)"
  fi

  rm -rf "$tmp"
}

# ── T4: matching content hash → _ensure_image skips rebuild ──────────────────
#
# Simulates an image whose runbox.content_hash label matches the current host
# state.  _ensure_image() must skip the rebuild and proceed directly to test.
#
# Before staleness detection (no label stored): image appears stale → rebuild.
# After staleness detection with correct hash:  no rebuild → PASSES.

test_t4_matching_hash_skips_rebuild()
{
  local tmp
  tmp="$(mktemp -d)"

  local mock_dir="$tmp/bin"
  mkdir "$mock_dir"

  local rebuild_triggered="$tmp/rebuild_triggered"

  _make_config "$tmp" "t4_matching_hash_test_image"

  # Compute the exact hash _content_hash() will produce for this config+workspace.
  # Inside runbox-run: SCRIPT_DIR = dirname(runbox-run), WORKSPACE_ROOT = SCRIPT_DIR/..
  # Must replicate all five inputs added by Fix(BUG-002): Cargo + config + $0 +
  # plugins.sh + DOCKERFILE (in the same order as _content_hash()).
  local runbox_script_dir
  runbox_script_dir="$( cd "$( dirname "$RUNBOX_RUN" )" && pwd )"
  local workspace_root
  workspace_root="$( cd "$runbox_script_dir/.." && pwd )"
  local current_hash
  current_hash=$( {
    find "$workspace_root" \( -name Cargo.toml -o -name Cargo.lock \) -print0 \
      | sort -z | xargs -0 sha256sum 2>/dev/null
    sha256sum "$tmp/runbox.yml"                              2>/dev/null
    sha256sum "$RUNBOX_RUN"                                  2>/dev/null
    sha256sum "$runbox_script_dir/plugins.sh"                2>/dev/null
    sha256sum "$runbox_script_dir/runbox.dockerfile"         2>/dev/null
  } | sha256sum | cut -d' ' -f1 )

  # Mock: image exists with the correct (current) hash label → no rebuild expected.
  cat > "$mock_dir/podman" << EOF
#!/usr/bin/env bash
case "\${1:-}" in
  container) exit 0 ;;
  build)
    touch "${rebuild_triggered}"
    exit 0
    ;;
  image)
    if [[ "\${2:-}" == "inspect" ]]
    then
      [[ "\${4:-}" == "--format" ]] && echo "${current_hash}" && exit 0
      exit 0
    fi
    ;;
  volume)
    [[ "\${2:-}" == "inspect" ]] && exit 1
    exit 0
    ;;
  run) exit 0 ;;
esac
EOF
  chmod +x "$mock_dir/podman"

  PATH="$mock_dir:$PATH" bash "$RUNBOX_RUN" "$tmp/runbox.yml" .test \
    2>/dev/null || true

  if [[ ! -f "$rebuild_triggered" ]]
  then
    _pass "T4: matching content hash skips rebuild (image reused as-is)"
  else
    _fail "T4: matching content hash skips rebuild (unexpected rebuild — hash comparison broken)"
  fi

  rm -rf "$tmp"
}

# ── T5: Dockerfile pre-creates /workspace/.claude bind-mount target ───────────
#
# Checks statically that runbox.dockerfile contains a RUN mkdir for
# $WORKSPACE_DIR/.claude.  Without this mkdir the OCI runtime (runc) tries to
# create the directory at container init time, but the workspace overlay is
# read-only (parent mounted :ro) at that point — mkdirat returns EROFS and the
# container never starts.
#
# Before fix: mkdir absent from Dockerfile → test FAILS.
# After fix:  RUN mkdir $WORKSPACE_DIR/.claude present → test PASSES.
# test_kind: bug_reproducer(BUG-001)

test_t5_dockerfile_creates_claude_dir()
{
  # Root Cause: runbox.dockerfile never created /workspace/.claude; OCI init EROFS.
  # Why Not Caught: runc silently created missing dirs on writable overlay; :ro mount exposed gap.
  # Fix Applied: RUN mkdir $WORKSPACE_DIR/.claude added to dockerfile (BUG-001).
  # Prevention: Any directory-type plugin_mount target must have a corresponding RUN mkdir.
  # Pitfall: OCI bind-mount targets absent from image fail EROFS when parent is :ro mounted.
  #
  # Scope: checks every runbox.dockerfile that configures plugin_mount: ~/.claude and has
  # had BUG-001 applied.  Add entries to the array below when fixing additional modules.
  local runbox_root repo_root
  runbox_root="$( cd "$SCRIPT_DIR/.." && pwd )"
  repo_root="$( cd "$runbox_root/../.." && pwd )"

  local -a checks=(
    "default:$runbox_root/runbox.dockerfile"
    "claude_profile:$repo_root/claude_profile/runbox/runbox.dockerfile"
    "claude_runner:$repo_root/claude_runner/runbox/runbox.dockerfile"
    "claude_storage:$repo_root/claude_storage/runbox/runbox.dockerfile"
  )

  local fail_count=0
  for entry in "${checks[@]}"; do
    local label dockerfile
    label="${entry%%:*}"
    dockerfile="${entry#*:}"
    if [[ ! -f "$dockerfile" ]]; then
      _fail "T5[$label]: dockerfile not found at $dockerfile"
      (( fail_count++ ))
      continue
    fi
    if ! grep -qE 'RUN[[:space:]]+mkdir\b.*/\.claude([[:space:]]|$)' "$dockerfile"; then
      _fail "T5[$label]: $dockerfile missing RUN mkdir .claude (plugin_mount will fail with EROFS)"
      (( fail_count++ ))
    fi
  done

  if [[ "$fail_count" -eq 0 ]]; then
    _pass "T5: all ${#checks[@]} dockerfile(s) with plugin_mount .claude have RUN mkdir .claude"
  fi
}

# ── T6: _content_hash() changes when runbox-run script is modified ────────────
#
# Verifies that _content_hash() includes $0 (the runbox-run script itself) in
# its hash input set.  The test runs the real runbox-run twice — once unmodified,
# once with a one-line prepend — captures the runbox.content_hash label argument
# passed to `podman build` via a mock, and confirms the two hashes differ.
#
# Before fix: $0 excluded from hash → both invocations produce identical hashes
#             → test FAILS.
# After fix:  $0 included in hash → hashes differ → test PASSES.
# test_kind: bug_reproducer(BUG-002)

test_t6_hash_changes_when_runbox_run_changes()
{
  # Root Cause: _content_hash() excluded $0/plugins.sh/dockerfile from hash scope.
  # Why Not Caught: Hash scoped to application inputs only; infra scripts treated as stable.
  # Fix Applied: $0, $SCRIPT_DIR/plugins.sh, $DOCKERFILE added to _content_hash() (BUG-002).
  # Prevention: Every file affecting container behavior must be in _content_hash() inputs.
  # Pitfall: Content hashes scoped to app inputs silently miss infrastructure changes.
  local tmp
  tmp="$(mktemp -d)"

  local mock_dir="$tmp/bin"
  mkdir "$mock_dir"

  # Create a mirror runbox directory so SCRIPT_DIR resolves identically for both
  # the original and the modified script (same plugins.sh + dockerfile paths).
  # The ONLY difference between the two runs is the $0 script content.
  local mirror_dir="$tmp/runbox"
  local runbox_dir
  runbox_dir="$( dirname "$RUNBOX_RUN" )"
  mkdir "$mirror_dir"
  cp "$RUNBOX_RUN" "$mirror_dir/runbox-run"
  # Copy the other infrastructure files so sha256sum in _content_hash() succeeds
  # for all inputs (missing files exit 1 under set -e and abort the script).
  cp "$runbox_dir/plugins.sh"       "$mirror_dir/plugins.sh"       2>/dev/null || true
  cp "$runbox_dir/runbox.dockerfile" "$mirror_dir/runbox.dockerfile" 2>/dev/null || true
  chmod +x "$mirror_dir/runbox-run"

  # Modified script: one prepended comment changes the content so $0 sha256 differs.
  local modified="$mirror_dir/runbox-run-modified"
  { echo "# T6 modification marker"; cat "$mirror_dir/runbox-run"; } > "$modified"
  chmod +x "$modified"

  # One config shared by both runs — identical path + content → same CONFIG hash.
  mkdir -p "$tmp/cfg"
  _make_config "$tmp/cfg" "t6_hash_test_image"

  # Mock podman: captures the runbox.content_hash label emitted by `podman build`.
  cat > "$mock_dir/podman" << 'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  container) exit 0 ;;
  build)
    prev=""
    for arg in "$@"; do
      if [[ "$prev" == "--label" && "$arg" == runbox.content_hash=* ]]; then
        echo "${arg#runbox.content_hash=}" > "$HASH_CAPTURE_FILE"
        break
      fi
      prev="$arg"
    done
    exit 0
    ;;
  image)
    [[ "${2:-}" == "inspect" ]] && exit 0
    exit 0
    ;;
esac
EOF
  chmod +x "$mock_dir/podman"

  local hash_file_orig="$tmp/hash_orig.txt"
  local hash_file_mod="$tmp/hash_mod.txt"

  # Run original script → capture hash.
  # SCRIPT_DIR = $mirror_dir; WORKSPACE_ROOT = $tmp (no Cargo files → same for both)
  HASH_CAPTURE_FILE="$hash_file_orig" PATH="$mock_dir:$PATH" \
    bash "$mirror_dir/runbox-run" "$tmp/cfg/runbox.yml" .build 2>/dev/null || true

  # Run modified script → same SCRIPT_DIR, same config, different $0 content.
  HASH_CAPTURE_FILE="$hash_file_mod" PATH="$mock_dir:$PATH" \
    bash "$modified" "$tmp/cfg/runbox.yml" .build 2>/dev/null || true

  local hash_orig hash_mod
  hash_orig="$( cat "$hash_file_orig" 2>/dev/null )"
  hash_mod="$( cat "$hash_file_mod"   2>/dev/null )"

  if [[ -z "$hash_orig" || -z "$hash_mod" ]]; then
    _fail "T6: _content_hash() changes when runbox-run script is modified (hash not captured)"
  elif [[ "$hash_orig" != "$hash_mod" ]]; then
    _pass "T6: _content_hash() changes when runbox-run script is modified (infrastructure file included in hash)"
  else
    _fail "T6: _content_hash() changes when runbox-run script is modified (hashes identical — \$0 not in hash input)"
  fi

  rm -rf "$tmp"
}

# ── Runner ────────────────────────────────────────────────────────────────────

echo ""
echo "── runbox build safety tests ─────────────────────────────────────────"
test_t1_build_exits_nonzero_when_image_inaccessible_after_build
test_t2_build_lock_directory_exists_during_build
test_t3_stale_hash_triggers_rebuild
test_t4_matching_hash_skips_rebuild
test_t5_dockerfile_creates_claude_dir
test_t6_hash_changes_when_runbox_run_changes
echo ""
echo "  Results: ${PASS_COUNT} passed, ${FAIL_COUNT} failed"
echo "──────────────────────────────────────────────────────────────────────"
echo ""

[[ "${FAIL_COUNT}" -eq 0 ]]
