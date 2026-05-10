# Plugin Analysis Operations

- **Actor:** Developer
- **Trigger:** A runbox plugin slot is added, removed, or its status, mechanism, or behavior changes.
- **Emits:** —

## Add Plugin (✅ hook-based)

1. Assign the next available ID (check `readme.md` Overview Table for current highest ID, increment by 1)
2. Add `_plugin_<name>()` stub (no-op) to the hook stubs block in `runbox-run`
3. Create `NNN_{snake_case_name}.md` with Status `✅`, Controls, Mechanism, and Notes
4. Add hook override to `run/plugins.sh` — read config via inherited `cfg`, override the stub
5. Register in `readme.md` Overview Table: add row with ID, Plugin, Status (`✅`), Category, Controls, Parameterizable, Affects

## Add Plugin (🔧 param-based)

Param-based plugins are configured via `runbox.yml` and `cfg_or` in `runbox-run`. Use this path when the behavioral slot is controlled by selecting an external resource (e.g., a different dockerfile) rather than injecting logic at runtime.

1. Add `cfg_or` read in `runbox-run` config block (e.g., `DOCKERFILE="$(cfg_or dockerfile ...)"`)
2. Use the new variable in the relevant `runbox-run` function(s)
3. Create `NNN_{snake_case_name}.md` with Status `🔧`, Controls, Mechanism, and Notes
4. Create `run/docs/parameter/NNN_{param_name}.md` documenting the new `runbox.yml` key
5. Register in both `plugin/readme.md` and `parameter/readme.md` Overview Tables

## Update Plugin

1. Edit the target `NNN_*.md` file and, if hook logic changed, update `run/plugins.sh`
2. If Status changed: update `readme.md` Overview Table Status column

## Example

Adding plugin `007_git_plugin` (bind-mounts `gh` CLI into container):

1. Check `readme.md` Overview Table — current highest ID is `006`
2. Create `007_git_plugin.md` with Status `✅`, Controls, Mechanism, and Notes sections
3. In `run/plugins.sh`: add `GH_PLUGIN="$(cfg gh_plugin)"` and extend `_plugin_test_args` to resolve `which gh` and add the bind-mount
4. Add row: `| [007](007_git_plugin.md) | \`gh_plugin\` | ✅ | VCS Integration |`
