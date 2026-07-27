#!/usr/bin/env bash
# Workspace plugin configuration for runbox-run.
# Sourced automatically by runbox-run via SCRIPT_DIR — applies to workspace and
# all per-crate configs that use runbox/runbox-run (e.g. module/claude_profile/runbox/).
# Reads $CONFIG (active runbox.yml) via inherited cfg/cfg_or from runbox-run.
#
# Remove this file to operate plugin-free — runbox-run stubs become no-ops.

BIN_PLUGIN="$(cfg bin_plugin)"
BIN_PLUGIN_VOLUME="$(cfg bin_plugin_volume)"
PLUGIN_MOUNT="$(cfg plugin_mount)"
BIN_MOUNT="$(cfg bin_mount)"

# ── Mount resolver ────────────────────────────────────────────────────────────
# Spec format: "host_path:container_path:check_type"  (3 fields from runbox.yml)
#   check_type: file | directory
# Caller passes required (true|false) and mode (ro|rw) as positional args.
# On success: prints "host:container:mode" ready for -v flag.
# On required+missing: prints error and exits.
# On optional+missing: prints nothing (caller checks with -n).

_resolve_mount()
{
  local spec="$1" required="$2" mode="${3:-ro}"
  local host container check rest
  host="${spec%%:*}";      rest="${spec#*:}"
  container="${rest%%:*}"; check="${rest#*:}"
  host="${host/#\~/$HOME}"

  local present=false
  [[ "$check" == "file"      ]] && [[ -f "$host" ]] && present=true
  [[ "$check" == "directory" ]] && [[ -d "$host" ]] && present=true

  if ! $present
  then
    if [[ "$required" == "true" ]]
    then
      echo "error: required mount not found at $host" >&2
      exit 1
    fi
    return 0
  fi

  echo "$host:$container:$mode"
}

# ── Plugin hook overrides ─────────────────────────────────────────────────────
# These override the no-op stubs defined in runbox-run.

_plugin_build_volumes()
{
  if [[ -n "$BIN_PLUGIN_VOLUME" ]]; then extra_volumes=( "${IMAGE}_plugin_targets" ); fi
}

_plugin_test_args()
{
  if [[ -n "$BIN_PLUGIN" ]]
  then
    local bin_name bin_container bin_host
    bin_name="${BIN_PLUGIN%%:*}"
    bin_container="${BIN_PLUGIN#*:}"
    bin_host="$(which "$bin_name" 2>/dev/null)" \
      || { echo "error: binary plugin '$bin_name' not found on PATH" >&2; exit 1; }
    # Fix(BUG-001): Export CARGO_TARGET_DIR so the binary plugin writes build artifacts to the
    # plugin_targets volume instead of /workspace/ (which is :ro in cmd_test).
    # Root cause: workspace :ro mount blocked plugin binary temp-target writes.
    # Pitfall: bin_plugin_volume must be exported as CARGO_TARGET_DIR or the plugin writes to workspace.
    bin_args=( -v "${IMAGE}_plugin_targets:$BIN_PLUGIN_VOLUME" -v "$bin_host:$bin_container:ro"
               -e "CARGO_TARGET_DIR=$BIN_PLUGIN_VOLUME" )
  fi
  _apply_data_mounts
}

# Resolves plugin_mount (.claude-style data dir) and bin_mount (read-only host binary) onto
# mount_args. Factored out of _plugin_test_args so per-crate overrides that skip bin_plugin/w3
# for targeted runs (e.g. verb/test_only) can still opt into these mounts without duplicating
# resolution logic — real-subprocess control tests need plugin_mount/bin_mount even when
# NEXTEST_FILTER narrows the run to a single test.
_apply_data_mounts()
{
  if [[ -n "$PLUGIN_MOUNT" ]]
  then
    local plugin_mount
    plugin_mount="$(_resolve_mount "$PLUGIN_MOUNT" true rw)"
    mount_args+=( -v "$plugin_mount" )
  fi
  # bin_mount: like bin_plugin's `which`-resolved host binary injection, but read-only with no
  # working volume / CARGO_TARGET_DIR side effect — for binaries invoked read-only that don't
  # compile their own output (e.g. claude). Safe to use alongside bin_plugin: independent -v
  # mounts, only bin_plugin touches CARGO_TARGET_DIR.
  if [[ -n "$BIN_MOUNT" ]]
  then
    local bin_mount_name bin_mount_container bin_mount_host
    bin_mount_name="${BIN_MOUNT%%:*}"
    bin_mount_container="${BIN_MOUNT#*:}"
    bin_mount_host="$(which "$bin_mount_name" 2>/dev/null)" \
      || { echo "error: bin_mount '$bin_mount_name' not found on PATH" >&2; exit 1; }
    mount_args+=( -v "$bin_mount_host:$bin_mount_container:ro" )
  fi
}

_plugin_list_args()
{
  if [[ -n "$BIN_PLUGIN_VOLUME" ]]
  then
    vol_args=( -v "${IMAGE}_plugin_targets:$BIN_PLUGIN_VOLUME" )
    cargo_env="CARGO_TARGET_DIR=$BIN_PLUGIN_VOLUME "
  fi
}

_plugin_shell_extra_args()
{
  if [[ -n "$BIN_PLUGIN" ]]
  then
    local bin_name bin_container bin_host
    bin_name="${BIN_PLUGIN%%:*}"
    bin_container="${BIN_PLUGIN#*:}"
    if bin_host="$(which "$bin_name" 2>/dev/null)"
    then
      plugin_extra_args+=( -v "$bin_host:$bin_container:ro" )
    fi
  fi
  if [[ -n "$PLUGIN_MOUNT" ]]
  then
    local shell_mount
    shell_mount="$(_resolve_mount "$PLUGIN_MOUNT" false ro)"
    [[ -n "$shell_mount" ]] && plugin_extra_args+=( -v "$shell_mount" )
  fi
}

_plugin_help_hint()
{
  if [[ -n "$PLUGIN_MOUNT" ]]; then plugin_hint=" (${PLUGIN_MOUNT%%:*} required)"; fi
}
