# Version Doc Entity

### Scope

- **Purpose**: Documents the release history of Claude Code through per-version changelog records.
- **Responsibility**: One doc instance per Claude Code version documented in the official changelog, capturing all changes introduced by that release.
- **In Scope**: Claude Code releases with published changelog entries (versions 2.1.74 through 2.1.220 and beyond as new releases appear).
- **Out of Scope**: Claude Code internals, API wire contracts, observed runtime behaviors (see behavior/, endpoint/, and settings/ collections).

### Type Declaration

- **Decision Criteria**: Use `version/` when documenting a specific Claude Code release and the changes it introduced. No standard type fits software release changelog records: `feature/` is a navigational hub for design specifications (not changelog entries); `format/` documents encoding schemas (not software versions).
- **Contrast with feature/**: `feature/` documents design decisions and cross-references for a named product capability; `version/` documents what changed in a specific software release (changelog content, one entry per version number).
- **Required Sections**: Abstract, Changes
- **Overview Table Columns**: `ID`, `Version`, `Date`, `Summary`, `Status`
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
- **096–098**: Versions 2.1.199–2.1.201
- **099–116**: Versions 2.1.202–2.1.220

### Overview Table

| ID | Version | Date | Summary | Status |
|----|---------|------|---------|--------|
| 001 | [2.1.74](001_v2_1_74.md) | 2026-03-12 | Added actionable suggestions to `/context` comm... | ✅ |
| 002 | [2.1.75](002_v2_1_75.md) | 2026-03-13 | Added 1M context window for Opus 4.6 by default... | ✅ |
| 003 | [2.1.76](003_v2_1_76.md) | 2026-03-14 | Added MCP elicitation support — MCP servers can... | ✅ |
| 004 | [2.1.77](004_v2_1_77.md) | 2026-03-17 | Increased default maximum output token limits f... | ✅ |
| 005 | [2.1.78](005_v2_1_78.md) | 2026-03-17 | Added `StopFailure` hook event that fires when ... | ✅ |
| 006 | [2.1.79](006_v2_1_79.md) | 2026-03-18 | Added `--console` flag to `claude auth login` f... | ✅ |
| 007 | [2.1.80](007_v2_1_80.md) | 2026-03-19 | Added `rate_limits` field to statusline scripts... | ✅ |
| 008 | [2.1.81](008_v2_1_81.md) | 2026-03-20 | Added `--bare` flag for scripted `-p` calls — s... | ✅ |
| 009 | [2.1.83](009_v2_1_83.md) | 2026-03-25 | Added `managed-settings.d/` drop-in directory a... | ✅ |
| 010 | [2.1.84](010_v2_1_84.md) | 2026-03-26 | Added PowerShell tool for Windows as an opt-in ... | ✅ |
| 011 | [2.1.85](011_v2_1_85.md) | 2026-03-26 | Added `CLAUDE_CODE_MCP_SERVER_NAME` and `CLAUDE... | ✅ |
| 012 | [2.1.86](012_v2_1_86.md) | 2026-03-27 | Added `X-Claude-Code-Session-Id` header to API ... | ✅ |
| 013 | [2.1.87](013_v2_1_87.md) | 2026-03-29 | Fixed messages in Cowork Dispatch not getting d... | ✅ |
| 014 | [2.1.89](014_v2_1_89.md) | 2026-04-01 | Added `"defer"` permission decision to `PreTool... | ✅ |
| 015 | [2.1.90](015_v2_1_90.md) | 2026-04-01 | Added `/powerup` — interactive lessons teaching... | ✅ |
| 016 | [2.1.91](016_v2_1_91.md) | 2026-04-02 | Added MCP tool result persistence override via ... | ✅ |
| 017 | [2.1.92](017_v2_1_92.md) | 2026-04-04 | Added `forceRemoteSettingsRefresh` policy setti... | ✅ |
| 018 | [2.1.94](018_v2_1_94.md) | 2026-04-07 | Added support for Amazon Bedrock powered by Man... | ✅ |
| 019 | [2.1.96](019_v2_1_96.md) | 2026-04-08 | Fixed Bedrock requests failing with `403 "Autho... | ✅ |
| 020 | [2.1.97](020_v2_1_97.md) | 2026-04-08 | Added focus view toggle (`Ctrl+O`) in `NO_FLICK... | ✅ |
| 021 | [2.1.98](021_v2_1_98.md) | 2026-04-09 | Added interactive Google Vertex AI setup wizard... | ✅ |
| 022 | [2.1.101](022_v2_1_101.md) | 2026-04-10 | Added `/team-onboarding` command to generate a ... | ✅ |
| 023 | [2.1.105](023_v2_1_105.md) | 2026-04-13 | Added `path` parameter to the `EnterWorktree` t... | ✅ |
| 024 | [2.1.107](024_v2_1_107.md) | 2026-04-14 | Show thinking hints sooner during long operations | ✅ |
| 025 | [2.1.108](025_v2_1_108.md) | 2026-04-14 | Added `ENABLE_PROMPT_CACHING_1H` env var to opt... | ✅ |
| 026 | [2.1.109](026_v2_1_109.md) | 2026-04-15 | Improved the extended-thinking indicator with a... | ✅ |
| 027 | [2.1.110](027_v2_1_110.md) | 2026-04-15 | Added `/tui` command and `tui` setting — run `/... | ✅ |
| 028 | [2.1.111](028_v2_1_111.md) | 2026-04-16 | Claude Opus 4.7 xhigh is now available! Use /ef... | ✅ |
| 029 | [2.1.112](029_v2_1_112.md) | 2026-04-16 | Fixed "claude-opus-4-7 is temporarily unavailab... | ✅ |
| 030 | [2.1.113](030_v2_1_113.md) | 2026-04-17 | Changed the CLI to spawn a native Claude Code b... | ✅ |
| 031 | [2.1.114](031_v2_1_114.md) | 2026-04-18 | Fixed a crash in the permission dialog when an ... | ✅ |
| 032 | [2.1.116](032_v2_1_116.md) | 2026-04-20 | `/resume` on large sessions is significantly fa... | ✅ |
| 033 | [2.1.117](033_v2_1_117.md) | 2026-04-22 | Forked subagents can now be enabled on external... | ✅ |
| 034 | [2.1.118](034_v2_1_118.md) | 2026-04-23 | Added vim visual mode (`v`) and visual-line mod... | ✅ |
| 035 | [2.1.119](035_v2_1_119.md) | 2026-04-23 | `/config` settings (theme, editor mode, verbose... | ✅ |
| 036 | [2.1.120](036_v2_1_120.md) | unknown | Windows: Git for Windows (Git Bash) is no longe... | ✅ |
| 037 | [2.1.121](037_v2_1_121.md) | 2026-04-28 | Added `alwaysLoad` option to MCP server config ... | ✅ |
| 038 | [2.1.122](038_v2_1_122.md) | 2026-04-28 | Added `ANTHROPIC_BEDROCK_SERVICE_TIER` environm... | ✅ |
| 039 | [2.1.123](039_v2_1_123.md) | 2026-04-29 | Fixed OAuth authentication failing with a 401 r... | ✅ |
| 040 | [2.1.126](040_v2_1_126.md) | 2026-05-01 | The `/model` picker now lists models from your ... | ✅ |
| 041 | [2.1.128](041_v2_1_128.md) | 2026-05-04 | Bare `/color` (no args) now picks a random sess... | ✅ |
| 042 | [2.1.129](042_v2_1_129.md) | 2026-05-06 | Added `--plugin-url <url>` flag to fetch a plug... | ✅ |
| 043 | [2.1.131](043_v2_1_131.md) | 2026-05-06 | Fixed VS Code extension failing to activate on ... | ✅ |
| 044 | [2.1.132](044_v2_1_132.md) | 2026-05-06 | Added `CLAUDE_CODE_SESSION_ID` environment vari... | ✅ |
| 045 | [2.1.133](045_v2_1_133.md) | 2026-05-07 | Added `worktree.baseRef` setting (`fresh` | `he... | ✅ |
| 046 | [2.1.136](046_v2_1_136.md) | 2026-05-08 | Added `CLAUDE_CODE_ENABLE_FEEDBACK_SURVEY_FOR_O... | ✅ |
| 047 | [2.1.137](047_v2_1_137.md) | 2026-05-09 | [VSCode] Fixed extension failing to activate on... | ✅ |
| 048 | [2.1.138](048_v2_1_138.md) | 2026-05-09 | Internal fixes | ✅ |
| 049 | [2.1.139](049_v2_1_139.md) | 2026-05-11 | Added agent view (Research Preview): a single l... | ✅ |
| 050 | [2.1.140](050_v2_1_140.md) | 2026-05-12 | Improved Agent tool `subagent_type` matching to... | ✅ |
| 051 | [2.1.141](051_v2_1_141.md) | 2026-05-13 | Added `terminalSequence` field to hook JSON out... | ✅ |
| 052 | [2.1.142](052_v2_1_142.md) | 2026-05-14 | Added new `claude agents` flags: `--add-dir`, `... | ✅ |
| 053 | [2.1.143](053_v2_1_143.md) | 2026-05-15 | Added plugin dependency enforcement: `claude pl... | ✅ |
| 054 | [2.1.144](054_v2_1_144.md) | 2026-05-19 | Added `/resume` support for background sessions... | ✅ |
| 055 | [2.1.145](055_v2_1_145.md) | 2026-05-19 | Added `claude agents --json` to list live Claud... | ✅ |
| 056 | [2.1.147](056_v2_1_147.md) | 2026-05-21 | Pinned background sessions (`Ctrl+T` in `claude... | ✅ |
| 057 | [2.1.148](057_v2_1_148.md) | 2026-05-22 | Fixed the Bash tool returning exit code 127 on ... | ✅ |
| 058 | [2.1.149](058_v2_1_149.md) | 2026-05-22 | `/usage` now shows a per-category breakdown of ... | ✅ |
| 059 | [2.1.150](059_v2_1_150.md) | 2026-05-23 | Internal infrastructure improvements (no user-f... | ✅ |
| 060 | [2.1.152](060_v2_1_152.md) | 2026-05-27 | `/code-review --fix` now applies review finding... | ✅ |
| 061 | [2.1.153](061_v2_1_153.md) | 2026-05-28 | Added `skipLfs` option to `github`/`git` plugin... | ✅ |
| 062 | [2.1.154](062_v2_1_154.md) | 2026-05-28 | Opus 4.8 is here! Now defaults to high effort ·... | ✅ |
| 063 | [2.1.156](063_v2_1_156.md) | 2026-05-29 | Fixed an issue when using Opus 4.8 where thinki... | ✅ |
| 064 | [2.1.157](064_v2_1_157.md) | 2026-05-29 | Plugins in `.claude/skills` directories are now... | ✅ |
| 065 | [2.1.158](065_v2_1_158.md) | 2026-05-30 | Auto mode is now available on Bedrock, Vertex, ... | ✅ |
| 066 | [2.1.159](066_v2_1_159.md) | 2026-05-31 | Internal infrastructure improvements (no user-f... | ✅ |
| 067 | [2.1.160](067_v2_1_160.md) | 2026-06-02 | Added a prompt before writing to shell startup ... | ✅ |
| 068 | [2.1.161](068_v2_1_161.md) | 2026-06-02 | `OTEL_RESOURCE_ATTRIBUTES` values are now inclu... | ✅ |
| 069 | [2.1.162](069_v2_1_162.md) | 2026-06-03 | `claude agents --json` now includes `waitingFor... | ✅ |
| 070 | [2.1.163](070_v2_1_163.md) | 2026-06-04 | Added `requiredMinimumVersion` and `requiredMax... | ✅ |
| 071 | [2.1.165](071_v2_1_165.md) | 2026-06-05 | Bug fixes and reliability improvements | ✅ |
| 072 | [2.1.166](072_v2_1_166.md) | 2026-06-06 | Added `fallbackModel` setting to configure up t... | ✅ |
| 073 | [2.1.167](073_v2_1_167.md) | 2026-06-06 | Bug fixes and reliability improvements | ✅ |
| 074 | [2.1.168](074_v2_1_168.md) | 2026-06-06 | Bug fixes and reliability improvements | ✅ |
| 075 | [2.1.169](075_v2_1_169.md) | 2026-06-08 | Self-hosted runner: added a `post-session` life... | ✅ |
| 076 | [2.1.170](076_v2_1_170.md) | 2026-06-09 | Introducing Claude Fable 5: a Mythos-class mode... | ✅ |
| 077 | [2.1.172](077_v2_1_172.md) | 2026-06-10 | Sub-agents can now spawn their own sub-agents (... | ✅ |
| 078 | [2.1.173](078_v2_1_173.md) | 2026-06-11 | Fixed Fable 5 model names with a `[1m]` suffix ... | ✅ |
| 079 | [2.1.174](079_v2_1_174.md) | 2026-06-12 | Added `wheelScrollAccelerationEnabled` setting ... | ✅ |
| 080 | [2.1.175](080_v2_1_175.md) | 2026-06-12 | Added `enforceAvailableModels` managed setting ... | ✅ |
| 081 | [2.1.176](081_v2_1_176.md) | 2026-06-12 | Session titles are now generated in the languag... | ✅ |
| 082 | [2.1.178](082_v2_1_178.md) | 2026-06-15 | Agent teams: removed the `TeamCreate` and `Team... | ✅ |
| 083 | [2.1.179](083_v2_1_179.md) | 2026-06-16 | Fixed mid-stream connection drops: partial resp... | ✅ |
| 084 | [2.1.181](084_v2_1_181.md) | 2026-06-17 | Added `/config key=value` syntax to set any set... | ✅ |
| 085 | [2.1.183](085_v2_1_183.md) | 2026-06-19 | Improved auto mode safety: destructive git comm... | ✅ |
| 086 | [2.1.185](086_v2_1_185.md) | 2026-06-20 | The stream-stall hint now reads "Waiting for AP... | ✅ |
| 087 | [2.1.186](087_v2_1_186.md) | 2026-06-22 | Added `claude mcp login <name>` and `claude mcp... | ✅ |
| 088 | [2.1.187](088_v2_1_187.md) | 2026-06-23 | Added `sandbox.credentials` setting to block sa... | ✅ |
| 089 | [2.1.190](089_v2_1_190.md) | 2026-06-24 | Bug fixes and reliability improvements | ✅ |
| 090 | [2.1.191](090_v2_1_191.md) | 2026-06-24 | Added `/rewind` support for resuming a conversa... | ✅ |
| 091 | [2.1.193](091_v2_1_193.md) | 2026-06-25 | Added `autoMode.classifyAllShell` setting to ro... | ✅ |
| 092 | [2.1.195](092_v2_1_195.md) | 2026-06-26 | Added `CLAUDE_CODE_DISABLE_MOUSE_CLICKS` to dis... | ✅ |
| 093 | [2.1.196](093_v2_1_196.md) | 2026-06-29 | Added support for organization default models —... | ✅ |
| 094 | [2.1.197](094_v2_1_197.md) | 2026-06-30 | Introducing Claude Sonnet 5: now the default mo... | ✅ |
| 095 | [2.1.198](095_v2_1_198.md) | 2026-07-01 | Claude in Chrome is now generally available | ✅ |
| 096 | [2.1.199](096_v2_1_199.md) | 2026-07-02 | Stacked slash-skill invocations like `/skill-a /... | ✅ |
| 097 | [2.1.200](097_v2_1_200.md) | 2026-07-03 | Changed `AskUserQuestion` dialogs to no longer a... | ✅ |
| 098 | [2.1.201](098_v2_1_201.md) | 2026-07-03 | Claude Sonnet 5 sessions no longer use the mid-c... | ✅ |
| 099 | [2.1.202](099_v2_1_202.md) | 2026-07-06 | Added a "Dynamic workflow size" setting in `/co... | ✅ |
| 100 | [2.1.203](100_v2_1_203.md) | 2026-07-07 | Added a warning when your login is about to exp... | ✅ |
| 101 | [2.1.204](101_v2_1_204.md) | 2026-07-08 | Fixed hook events not streaming during SessionS... | ✅ |
| 102 | [2.1.205](102_v2_1_205.md) | 2026-07-08 | Added an auto mode rule that blocks tampering w... | ✅ |
| 103 | [2.1.206](103_v2_1_206.md) | 2026-07-10 | Added directory path suggestions to `/cd`, matc... | ✅ |
| 104 | [2.1.207](104_v2_1_207.md) | 2026-07-11 | Auto mode is now available without `CLAUDE_CODE... | ✅ |
| 105 | [2.1.208](105_v2_1_208.md) | 2026-07-14 | Added screen reader mode: opt-in plain-text ren... | ✅ |
| 106 | [2.1.209](106_v2_1_209.md) | 2026-07-14 | Fixed /model and other dialogs being blocked in... | ✅ |
| 107 | [2.1.210](107_v2_1_210.md) | 2026-07-14 | Added a live elapsed-time counter to the collap... | ✅ |
| 108 | [2.1.211](108_v2_1_211.md) | 2026-07-15 | Added `--forward-subagent-text` flag and `CLAUD... | ✅ |
| 109 | [2.1.212](109_v2_1_212.md) | 2026-07-17 | `/fork` now copies your conversation into a new... | ✅ |
| 110 | [2.1.214](110_v2_1_214.md) | 2026-07-18 | Fixed single-segment `dir/**` allow rules like ... | ✅ |
| 111 | [2.1.215](111_v2_1_215.md) | 2026-07-19 | Claude no longer runs the `/verify` and `/code-... | ✅ |
| 112 | [2.1.216](112_v2_1_216.md) | 2026-07-20 | Added `sandbox.filesystem.disabled` setting to ... | ✅ |
| 113 | [2.1.217](113_v2_1_217.md) | 2026-07-21 | Added emoji shortcode autocomplete in the promp... | ✅ |
| 114 | [2.1.218](114_v2_1_218.md) | 2026-07-22 | Changed `/code-review` to run as a background s... | ✅ |
| 115 | [2.1.219](115_v2_1_219.md) | 2026-07-24 | Added Claude Opus 5 (`claude-opus-5`), now the ... | ✅ |
| 116 | [2.1.220](116_v2_1_220.md) | 2026-07-25 | Bug fixes and reliability improvements | ✅ |
