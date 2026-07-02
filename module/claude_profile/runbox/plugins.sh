#!/usr/bin/env bash
# Project-level runbox plugin for claude_profile.
# Sourced by runbox-run after the workspace plugins.sh (runbox/plugins.sh).
#
# Provides NEXTEST_FILTER pass-through for targeted single-test execution
# (verb/test1).  When NEXTEST_FILTER is set, overrides TEST_SCRIPT to
# verb/test1.d/l1 and injects the filter as a container env var.
# The w3 binary plugin and .claude mount are skipped for targeted runs;
# cargo uses the pre-seeded /workspace/target volume for fast first-run.

# ── Compose with workspace _plugin_test_args ──────────────────────────────────
# Rename workspace function before redefining so the normal full-suite path
# still calls all workspace plugin logic unchanged.
if declare -f _plugin_test_args > /dev/null
then
  eval "_ws_plugin_test_args()$(declare -f _plugin_test_args | tail -n +2)"
else
  _ws_plugin_test_args() { :; }
fi

_plugin_test_args()
{
  if [[ -z "${NEXTEST_FILTER:-}" ]]
  then
    # Normal full-suite path — delegate entirely to the workspace plugin.
    _ws_plugin_test_args
    # Fix(BUG-TMP): PersistPaths::new() in container uses $HOME=/workspace to
    #   resolve credential_store → /workspace/.persistent/claude/credential/.
    #   Without this mount that path is empty → host_quota_snapshot_from_cache()
    #   returns None → live_quota_snapshot() returns None → quota cache not
    #   pre-populated → lim_it tests hit live /api/oauth/usage → 429 → failures.
    # Root cause: plugin_mount only mounts ~/.claude; the credential store lives
    #   under $PRO/.persistent (when $PRO is set and is-dir) or $HOME/.persistent.
    #   Mirror PersistPaths::resolve_root() — prefer $PRO, fall back to $HOME.
    # Pitfall: mark optional (false) — absent on CI; absent host silently skips mount.
    local _persist_root="$HOME"
    if [[ -n "${PRO:-}" ]] && [[ -d "${PRO:-}" ]]; then _persist_root="$PRO"; fi
    local persist_spec="$_persist_root/.persistent:/workspace/.persistent:directory"
    local persist_mount
    persist_mount="$(_resolve_mount "$persist_spec" false rw)"
    [[ -n "$persist_mount" ]] && mount_args+=( -v "$persist_mount" )
    return
  fi
  # Targeted single-test path:
  #   - redirect TEST_SCRIPT to test1.d/l1 (reads $NEXTEST_FILTER inside container)
  #   - skip the w3 binary plugin and .claude mount (not needed for unit tests)
  #   - use /workspace/target (pre-seeded from image) for fast incremental builds
  TEST_SCRIPT="module/claude_profile/verb/test1.d/l1"
  bin_args=( -e "NEXTEST_FILTER=$NEXTEST_FILTER" )
  mount_args=()
}
