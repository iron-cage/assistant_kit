#!/usr/bin/env bash
# Workspace plugin configuration for runbox-run.
# Sourced automatically by runbox-run via SCRIPT_DIR — applies to workspace and
# all per-crate configs that use run/runbox-run (e.g. module/claude_profile/run/).
# Reads $CONFIG (active runbox.yml) via inherited cfg/cfg_or from runbox-run.
#
# Remove this file to operate plugin-free — runbox-run stubs become no-ops.

BIN_PLUGIN="$(cfg bin_plugin)"
BIN_PLUGIN_VOLUME="$(cfg bin_plugin_volume)"
PLUGIN_MOUNT="$(cfg plugin_mount)"

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
  [[ -n "$BIN_PLUGIN_VOLUME" ]] && extra_volumes=( "${IMAGE}_plugin_targets" )
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
    bin_args=( -v "${IMAGE}_plugin_targets:$BIN_PLUGIN_VOLUME" -v "$bin_host:$bin_container:ro" )
  fi
  if [[ -n "$PLUGIN_MOUNT" ]]
  then
    local plugin_mount
    plugin_mount="$(_resolve_mount "$PLUGIN_MOUNT" true rw)"
    mount_args=( -v "$plugin_mount" )
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
  [[ -n "$PLUGIN_MOUNT" ]] && plugin_hint=" (${PLUGIN_MOUNT%%:*} required)"
}
