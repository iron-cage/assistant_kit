//! Integration tests: `.` (Dot) and `.help` — help output.
//!
//! Tests invoke the compiled `clp` binary as a subprocess via `CARGO_BIN_EXE_clp`.
//! All tests run without TTY (subprocess stdout is not a terminal), so ANSI colour
//! codes are suppressed at the source and must not appear in captured output.
//!
//! ## Test Matrix
//!
//! | ID    | Test Function                           | Condition                                     | P/N |
//! |-------|-----------------------------------------|-----------------------------------------------|-----|
//! | dot01 | `dot01_dot_and_help_byte_identical`     | `.` and `.help` stdout byte-identical         | P   |
//! | dot02 | `dot02_dot_exits_0`                     | `.` exits 0                                   | P   |
//! | dot03 | `dot03_dot_hidden_from_listing`         | no bare `.` command row in listing            | P   |
//! | dot04 | `dot04_all_visible_commands_present`    | 9 commands present; removed commands absent   | P   |
//! | dot05 | `dot05_exactly_nine_command_rows`       | exactly 9 lines starting with `"    ."`       | P   |
//! | dot06 | `dot06_usage_line_present`              | stdout contains `"Usage: clp <command>"`      | P   |
//! | dot07 | `dot07_unknown_param_ignored`           | `. foo::bar` output identical to bare `.`     | P   |
//! | dot08 | `dot08_output_stable_across_invocations`| 3 invocations all byte-identical              | P   |
//! | dot09 | `dot09_grouped_section_headers`         | "Account management" and "Status & info"      | P   |
//! | dot10 | `dot10_no_per_command_param_syntax`     | no `[`/`]` in command rows                   | P   |
//! | dot11 | `dot11_options_section_hints`           | format/dry/name hints in Options section      | P   |
//! | dot12 | `dot12_no_ansi_in_subprocess_output`    | zero ESC (`0x1b`) bytes in stdout             | P   |

use crate::helpers::{ run_cs, stdout, assert_exit };

// ── dot01 — `.` and `.help` produce identical stdout ─────────────────────────

#[ test ]
fn dot01_dot_and_help_byte_identical()
{
  let dot      = run_cs( &[ "." ] );
  let dot_help = run_cs( &[ ".help" ] );
  assert_eq!(
    dot.stdout,
    dot_help.stdout,
    "`.` and `.help` stdout must be byte-identical",
  );
}

// ── dot02 — `.` exits 0 ───────────────────────────────────────────────────────

#[ test ]
fn dot02_dot_exits_0()
{
  let out = run_cs( &[ "." ] );
  assert_exit( &out, 0 );
}

// ── dot03 — `.` is hidden from the command listing ───────────────────────────

#[ test ]
fn dot03_dot_hidden_from_listing()
{
  // A bare `.` command row would appear as "    . " (4 spaces + dot + space)
  // in the padded 20-char name column.  Confirm it is absent.
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );
  assert!(
    !text.contains( "    . " ),
    "`.` must not appear as a command row in the listing",
  );
}

// ── dot04 — all 9 visible commands present; removed names absent ──────────────

#[ test ]
fn dot04_all_visible_commands_present()
{
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );

  let visible = [
    ".accounts",
    ".account.save",
    ".account.use",
    ".account.delete",
    ".account.limits",
    ".credentials.status",
    ".token.status",
    ".paths",
    ".usage",
  ];
  for name in &visible
  {
    assert!( text.contains( name ), "output must contain command {name:?}" );
  }

  assert!( !text.contains( ".account.list"   ), ".account.list must not appear (removed)" );
  assert!( !text.contains( ".account.status" ), ".account.status must not appear (removed)" );
}

// ── dot05 — exactly 9 command rows in listing ─────────────────────────────────

#[ test ]
fn dot05_exactly_nine_command_rows()
{
  let out   = run_cs( &[ "." ] );
  let text  = stdout( &out );
  let count = text.lines().filter( |l| l.starts_with( "    ." ) ).count();
  assert_eq!( count, 9, "expected 9 command rows starting with '    .', got {count}" );
}

// ── dot06 — usage line includes `<command>` syntax ───────────────────────────

#[ test ]
fn dot06_usage_line_present()
{
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );
  assert!(
    text.contains( "Usage: clp <command>" ),
    "output must contain usage line 'Usage: clp <command>'",
  );
}

// ── dot07 — trailing unknown param is silently ignored ───────────────────────

#[ test ]
fn dot07_unknown_param_ignored()
{
  // `.` triggers the help path before any param processing; `foo::bar` is never parsed.
  let bare  = run_cs( &[ "." ] );
  let extra = run_cs( &[ ".", "foo::bar" ] );
  assert_eq!(
    bare.stdout,
    extra.stdout,
    "trailing unknown param must not change help output",
  );
}

// ── dot08 — output stable across repeated invocations ────────────────────────

#[ test ]
fn dot08_output_stable_across_invocations()
{
  let out1 = run_cs( &[ "." ] );
  let out2 = run_cs( &[ "." ] );
  let out3 = run_cs( &[ "." ] );
  assert_eq!( out1.stdout, out2.stdout, "first and second invocations must match" );
  assert_eq!( out2.stdout, out3.stdout, "second and third invocations must match" );
}

// ── dot09 — grouped section headers present ──────────────────────────────────

#[ test ]
fn dot09_grouped_section_headers()
{
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );
  assert!( text.contains( "Account management" ), "output must contain 'Account management' header" );
  assert!( text.contains( "Status & info" ),      "output must contain 'Status & info' header"      );
}

// ── dot10 — no per-command parameter syntax in command rows ──────────────────

#[ test ]
fn dot10_no_per_command_param_syntax()
{
  // Command rows (lines starting with "    .") must not carry bracket-style param lists.
  // The usage line itself contains "[key::value ...]" which is intentional — only
  // command rows are checked here.
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );
  for line in text.lines()
  {
    if line.starts_with( "    ." )
    {
      assert!( !line.contains( '[' ), "command row must not contain '[': {line:?}" );
      assert!( !line.contains( ']' ), "command row must not contain ']': {line:?}" );
    }
  }
}

// ── dot11 — Options section lists cross-command hints ────────────────────────

#[ test ]
fn dot11_options_section_hints()
{
  let out  = run_cs( &[ "." ] );
  let text = stdout( &out );
  assert!( text.contains( "format::text|json" ), "Options must include format hint" );
  assert!( text.contains( "dry::bool"          ), "Options must include dry hint"    );
  assert!( text.contains( "name::EMAIL"         ), "Options must include name hint"   );
}

// ── dot12 — no ANSI escape sequences in subprocess stdout ────────────────────

#[ test ]
fn dot12_no_ansi_in_subprocess_output()
{
  // `clp` detects non-TTY stdout and suppresses all ANSI colour codes at the source.
  // Confirm by checking raw bytes for ESC (0x1b) — the universal ANSI escape prefix.
  let out = run_cs( &[ "." ] );
  assert!(
    !out.stdout.contains( &0x1b_u8 ),
    "subprocess stdout must not contain ESC bytes (ANSI suppressed in non-TTY)",
  );
}
