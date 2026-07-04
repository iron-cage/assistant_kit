# Version Doc Entity

### Scope

- **Purpose**: Documents the release history of Claude Code through per-version changelog records.
- **Responsibility**: One doc instance per Claude Code version documented in the official changelog, capturing all changes introduced by that release.
- **In Scope**: Claude Code releases with published changelog entries (versions 2.1.74 through 2.1.198 and beyond as new releases appear).
- **Out of Scope**: Claude Code internals, API wire contracts, observed runtime behaviors (see behavior/, endpoint/, and settings/ collections).

### Type Declaration

- **Decision Criteria**: Use `version/` when documenting a specific Claude Code release and the changes it introduced. No standard type fits software release changelog records: `feature/` is a navigational hub for design specifications (not changelog entries); `format/` documents encoding schemas (not software versions).
- **Contrast with feature/**: `feature/` documents design decisions and cross-references for a named product capability; `version/` documents what changed in a specific software release (changelog content, one entry per version number).
- **Required Sections**: Abstract, Changes
- **Overview Table Columns**: `ID`, `Version`, `Summary`, `Status`
- **Quality Checklist**:
  - [ ] Does the Abstract state the version number and its primary significance in one sentence?
  - [ ] Does the Changes section list all official changelog entries for this version verbatim?
  - [ ] Is this the only doc instance for this version number (no duplicate IDs)?

### Organization

Versions are assigned sequential IDs in chronological order (oldest first), so new releases append at the end.

- **001–024**: Versions 2.1.74–2.1.107
- **025–048**: Versions 2.1.108–2.1.138
- **049–072**: Versions 2.1.139–2.1.166
- **073–095**: Versions 2.1.167–2.1.198

### Overview Table

| ID | Version | Summary | Status |
|----|---------|---------|--------|
| 001 | [2.1.74](001_v2_1_74.md) | Added actionable suggestions to `/context` comm... | ✅ |
| 002 | [2.1.75](002_v2_1_75.md) | Added 1M context window for Opus 4.6 by default... | ✅ |
| 003 | [2.1.76](003_v2_1_76.md) | Added MCP elicitation support — MCP servers can... | ✅ |
| 004 | [2.1.77](004_v2_1_77.md) | Increased default maximum output token limits f... | ✅ |
| 005 | [2.1.78](005_v2_1_78.md) | Added `StopFailure` hook event that fires when ... | ✅ |
| 006 | [2.1.79](006_v2_1_79.md) | Added `--console` flag to `claude auth login` f... | ✅ |
| 007 | [2.1.80](007_v2_1_80.md) | Added `rate_limits` field to statusline scripts... | ✅ |
| 008 | [2.1.81](008_v2_1_81.md) | Added `--bare` flag for scripted `-p` calls — s... | ✅ |
| 009 | [2.1.83](009_v2_1_83.md) | Added `managed-settings.d/` drop-in directory a... | ✅ |
| 010 | [2.1.84](010_v2_1_84.md) | Added PowerShell tool for Windows as an opt-in ... | ✅ |
| 011 | [2.1.85](011_v2_1_85.md) | Added `CLAUDE_CODE_MCP_SERVER_NAME` and `CLAUDE... | ✅ |
| 012 | [2.1.86](012_v2_1_86.md) | Added `X-Claude-Code-Session-Id` header to API ... | ✅ |
| 013 | [2.1.87](013_v2_1_87.md) | Fixed messages in Cowork Dispatch not getting d... | ✅ |
| 014 | [2.1.89](014_v2_1_89.md) | Added `"defer"` permission decision to `PreTool... | ✅ |
| 015 | [2.1.90](015_v2_1_90.md) | Added `/powerup` — interactive lessons teaching... | ✅ |
| 016 | [2.1.91](016_v2_1_91.md) | Added MCP tool result persistence override via ... | ✅ |
| 017 | [2.1.92](017_v2_1_92.md) | Added `forceRemoteSettingsRefresh` policy setti... | ✅ |
| 018 | [2.1.94](018_v2_1_94.md) | Added support for Amazon Bedrock powered by Man... | ✅ |
| 019 | [2.1.96](019_v2_1_96.md) | Fixed Bedrock requests failing with `403 "Autho... | ✅ |
| 020 | [2.1.97](020_v2_1_97.md) | Added focus view toggle (`Ctrl+O`) in `NO_FLICK... | ✅ |
| 021 | [2.1.98](021_v2_1_98.md) | Added interactive Google Vertex AI setup wizard... | ✅ |
| 022 | [2.1.101](022_v2_1_101.md) | Added `/team-onboarding` command to generate a ... | ✅ |
| 023 | [2.1.105](023_v2_1_105.md) | Added `path` parameter to the `EnterWorktree` t... | ✅ |
| 024 | [2.1.107](024_v2_1_107.md) | Show thinking hints sooner during long operations | ✅ |
| 025 | [2.1.108](025_v2_1_108.md) | Added `ENABLE_PROMPT_CACHING_1H` env var to opt... | ✅ |
| 026 | [2.1.109](026_v2_1_109.md) | Improved the extended-thinking indicator with a... | ✅ |
| 027 | [2.1.110](027_v2_1_110.md) | Added `/tui` command and `tui` setting — run `/... | ✅ |
| 028 | [2.1.111](028_v2_1_111.md) | Claude Opus 4.7 xhigh is now available! Use /ef... | ✅ |
| 029 | [2.1.112](029_v2_1_112.md) | Fixed "claude-opus-4-7 is temporarily unavailab... | ✅ |
| 030 | [2.1.113](030_v2_1_113.md) | Changed the CLI to spawn a native Claude Code b... | ✅ |
| 031 | [2.1.114](031_v2_1_114.md) | Fixed a crash in the permission dialog when an ... | ✅ |
| 032 | [2.1.116](032_v2_1_116.md) | `/resume` on large sessions is significantly fa... | ✅ |
| 033 | [2.1.117](033_v2_1_117.md) | Forked subagents can now be enabled on external... | ✅ |
| 034 | [2.1.118](034_v2_1_118.md) | Added vim visual mode (`v`) and visual-line mod... | ✅ |
| 035 | [2.1.119](035_v2_1_119.md) | `/config` settings (theme, editor mode, verbose... | ✅ |
| 036 | [2.1.120](036_v2_1_120.md) | Windows: Git for Windows (Git Bash) is no longe... | ✅ |
| 037 | [2.1.121](037_v2_1_121.md) | Added `alwaysLoad` option to MCP server config ... | ✅ |
| 038 | [2.1.122](038_v2_1_122.md) | Added `ANTHROPIC_BEDROCK_SERVICE_TIER` environm... | ✅ |
| 039 | [2.1.123](039_v2_1_123.md) | Fixed OAuth authentication failing with a 401 r... | ✅ |
| 040 | [2.1.126](040_v2_1_126.md) | The `/model` picker now lists models from your ... | ✅ |
| 041 | [2.1.128](041_v2_1_128.md) | Bare `/color` (no args) now picks a random sess... | ✅ |
| 042 | [2.1.129](042_v2_1_129.md) | Added `--plugin-url <url>` flag to fetch a plug... | ✅ |
| 043 | [2.1.131](043_v2_1_131.md) | Fixed VS Code extension failing to activate on ... | ✅ |
| 044 | [2.1.132](044_v2_1_132.md) | Added `CLAUDE_CODE_SESSION_ID` environment vari... | ✅ |
| 045 | [2.1.133](045_v2_1_133.md) | Added `worktree.baseRef` setting (`fresh` | `he... | ✅ |
| 046 | [2.1.136](046_v2_1_136.md) | Added `CLAUDE_CODE_ENABLE_FEEDBACK_SURVEY_FOR_O... | ✅ |
| 047 | [2.1.137](047_v2_1_137.md) | [VSCode] Fixed extension failing to activate on... | ✅ |
| 048 | [2.1.138](048_v2_1_138.md) | Internal fixes | ✅ |
| 049 | [2.1.139](049_v2_1_139.md) | Added agent view (Research Preview): a single l... | ✅ |
| 050 | [2.1.140](050_v2_1_140.md) | Improved Agent tool `subagent_type` matching to... | ✅ |
| 051 | [2.1.141](051_v2_1_141.md) | Added `terminalSequence` field to hook JSON out... | ✅ |
| 052 | [2.1.142](052_v2_1_142.md) | Added new `claude agents` flags: `--add-dir`, `... | ✅ |
| 053 | [2.1.143](053_v2_1_143.md) | Added plugin dependency enforcement: `claude pl... | ✅ |
| 054 | [2.1.144](054_v2_1_144.md) | Added `/resume` support for background sessions... | ✅ |
| 055 | [2.1.145](055_v2_1_145.md) | Added `claude agents --json` to list live Claud... | ✅ |
| 056 | [2.1.147](056_v2_1_147.md) | Pinned background sessions (`Ctrl+T` in `claude... | ✅ |
| 057 | [2.1.148](057_v2_1_148.md) | Fixed the Bash tool returning exit code 127 on ... | ✅ |
| 058 | [2.1.149](058_v2_1_149.md) | `/usage` now shows a per-category breakdown of ... | ✅ |
| 059 | [2.1.150](059_v2_1_150.md) | Internal infrastructure improvements (no user-f... | ✅ |
| 060 | [2.1.152](060_v2_1_152.md) | `/code-review --fix` now applies review finding... | ✅ |
| 061 | [2.1.153](061_v2_1_153.md) | Added `skipLfs` option to `github`/`git` plugin... | ✅ |
| 062 | [2.1.154](062_v2_1_154.md) | Opus 4.8 is here! Now defaults to high effort ·... | ✅ |
| 063 | [2.1.156](063_v2_1_156.md) | Fixed an issue when using Opus 4.8 where thinki... | ✅ |
| 064 | [2.1.157](064_v2_1_157.md) | Plugins in `.claude/skills` directories are now... | ✅ |
| 065 | [2.1.158](065_v2_1_158.md) | Auto mode is now available on Bedrock, Vertex, ... | ✅ |
| 066 | [2.1.159](066_v2_1_159.md) | Internal infrastructure improvements (no user-f... | ✅ |
| 067 | [2.1.160](067_v2_1_160.md) | Added a prompt before writing to shell startup ... | ✅ |
| 068 | [2.1.161](068_v2_1_161.md) | `OTEL_RESOURCE_ATTRIBUTES` values are now inclu... | ✅ |
| 069 | [2.1.162](069_v2_1_162.md) | `claude agents --json` now includes `waitingFor... | ✅ |
| 070 | [2.1.163](070_v2_1_163.md) | Added `requiredMinimumVersion` and `requiredMax... | ✅ |
| 071 | [2.1.165](071_v2_1_165.md) | Bug fixes and reliability improvements | ✅ |
| 072 | [2.1.166](072_v2_1_166.md) | Added `fallbackModel` setting to configure up t... | ✅ |
| 073 | [2.1.167](073_v2_1_167.md) | Bug fixes and reliability improvements | ✅ |
| 074 | [2.1.168](074_v2_1_168.md) | Bug fixes and reliability improvements | ✅ |
| 075 | [2.1.169](075_v2_1_169.md) | Self-hosted runner: added a `post-session` life... | ✅ |
| 076 | [2.1.170](076_v2_1_170.md) | Introducing Claude Fable 5: a Mythos-class mode... | ✅ |
| 077 | [2.1.172](077_v2_1_172.md) | Sub-agents can now spawn their own sub-agents (... | ✅ |
| 078 | [2.1.173](078_v2_1_173.md) | Fixed Fable 5 model names with a `[1m]` suffix ... | ✅ |
| 079 | [2.1.174](079_v2_1_174.md) | Added `wheelScrollAccelerationEnabled` setting ... | ✅ |
| 080 | [2.1.175](080_v2_1_175.md) | Added `enforceAvailableModels` managed setting ... | ✅ |
| 081 | [2.1.176](081_v2_1_176.md) | Session titles are now generated in the languag... | ✅ |
| 082 | [2.1.178](082_v2_1_178.md) | Agent teams: removed the `TeamCreate` and `Team... | ✅ |
| 083 | [2.1.179](083_v2_1_179.md) | Fixed mid-stream connection drops: partial resp... | ✅ |
| 084 | [2.1.181](084_v2_1_181.md) | Added `/config key=value` syntax to set any set... | ✅ |
| 085 | [2.1.183](085_v2_1_183.md) | Improved auto mode safety: destructive git comm... | ✅ |
| 086 | [2.1.185](086_v2_1_185.md) | The stream-stall hint now reads "Waiting for AP... | ✅ |
| 087 | [2.1.186](087_v2_1_186.md) | Added `claude mcp login <name>` and `claude mcp... | ✅ |
| 088 | [2.1.187](088_v2_1_187.md) | Added `sandbox.credentials` setting to block sa... | ✅ |
| 089 | [2.1.190](089_v2_1_190.md) | Bug fixes and reliability improvements | ✅ |
| 090 | [2.1.191](090_v2_1_191.md) | Added `/rewind` support for resuming a conversa... | ✅ |
| 091 | [2.1.193](091_v2_1_193.md) | Added `autoMode.classifyAllShell` setting to ro... | ✅ |
| 092 | [2.1.195](092_v2_1_195.md) | Added `CLAUDE_CODE_DISABLE_MOUSE_CLICKS` to dis... | ✅ |
| 093 | [2.1.196](093_v2_1_196.md) | Added support for organization default models —... | ✅ |
| 094 | [2.1.197](094_v2_1_197.md) | Introducing Claude Sonnet 5: now the default mo... | ✅ |
| 095 | [2.1.198](095_v2_1_198.md) | Claude in Chrome is now generally available | ✅ |
