//! Real-subprocess tests: MCP-server-related control methods — `mcpServerStatus`,
//! `reconnectMcpServer`, `toggleMcpServer`, `setMcpServers`, `setMcpPermissionModeOverride`
//! (task 415 Test Matrix rows IT-20, IT-22, IT-23, IT-24, IT-27).
//!
//! No mocking anywhere below — every test spawns a real `claude` subprocess via
//! [`claude_runner_core::ClaudeCommand::spawn_control_session`], with no MCP servers
//! configured (so the "target server" in each test is always genuinely unregistered).

mod control_session_common;

use claude_runner_core::McpPermissionOverrideMode;

/// IT-20: `mcpServerStatus()` is a real wire round trip returning a well-formed status list.
/// Which servers (if any) are connected depends on this environment's own global MCP
/// configuration (e.g. a browser-automation server tied to a Chrome extension, independent
/// of the session's working directory) — not something this test can assume either way — so
/// it asserts structural correctness of whatever comes back rather than emptiness.
#[ test ]
fn it_20_mcp_server_status_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let servers = session.mcp_server_status().expect( "mcp_server_status() must succeed against a real session" );
  for server in &servers
  {
    assert!( !server.name.is_empty(), "each reported server must have a non-empty name" );
    assert!( !server.status.is_empty(), "each reported server must have a non-empty status" );
  }
}

/// IT-22: `reconnectMcpServer(serverName)` surfaces a clean typed error for an unknown server
/// — not a hang, not a panic. (Confirmed real-subprocess message: "Server not found: ...".)
#[ test ]
fn it_22_reconnect_mcp_server_surfaces_a_clean_error_for_an_unknown_server()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let err = session.reconnect_mcp_server( "clr-test-nonexistent-server" )
    .expect_err( "reconnecting an unregistered server must return Err, not hang or panic" );
  assert!( !err.to_string().is_empty(), "error message must be non-empty" );
}

/// IT-23: `toggleMcpServer(serverName, enabled)` surfaces a clean typed error for an unknown
/// server, same as [`it_22_reconnect_mcp_server_surfaces_a_clean_error_for_an_unknown_server`].
#[ test ]
fn it_23_toggle_mcp_server_surfaces_a_clean_error_for_an_unknown_server()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let err = session.toggle_mcp_server( "clr-test-nonexistent-server", false )
    .expect_err( "toggling an unregistered server must return Err, not hang or panic" );
  assert!( !err.to_string().is_empty(), "error message must be non-empty" );
}

/// IT-24: `setMcpServers(servers)` is a real wire round trip; an empty server map applies
/// cleanly with nothing added, removed, or errored.
#[ test ]
fn it_24_set_mcp_servers_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let result = session.set_mcp_servers( serde_json::json!( {} ) )
    .expect( "set_mcp_servers() must succeed against a real session" );
  assert!( result.added.is_empty() );
  assert!( result.removed.is_empty() );
  assert!( result.errors.is_empty() );
}

/// IT-27: `setMcpPermissionModeOverride(serverName, mode)` pins or clears a per-server
/// override; an unmatched server name produces a `{warning}` response either way, rather than
/// an error.
#[ test ]
fn it_27_set_mcp_permission_mode_override_warns_for_an_unmatched_server()
{
  let ( session, _dir ) = control_session_common::spawn_session();

  let pinned = session.set_mcp_permission_mode_override(
    "clr-test-nonexistent-server", Some( McpPermissionOverrideMode::Auto )
  ).expect( "set_mcp_permission_mode_override() must succeed (with a warning) for an unmatched server name" );
  assert!( pinned.warning.is_some(), "an unmatched server name must produce a warning when pinning an override" );

  let cleared = session.set_mcp_permission_mode_override( "clr-test-nonexistent-server", None )
    .expect( "clearing an override (mode: None) must also succeed" );
  assert!( cleared.warning.is_some(), "an unmatched server name must produce a warning when clearing too" );
}
