//! Real-subprocess tests: the 5 cached-accessor control methods — `initializationResult`,
//! `reinitialize`, `supportedCommands`, `supportedModels`, `supportedAgents`, `accountInfo`
//! (task 415 Test Matrix rows IT-15, IT-16, IT-17, IT-18, IT-19, IT-21).
//!
//! Confirmed design (see `src/control.rs` module doc + `tests/fixtures/sdk_control_capture/`):
//! 4 of these 5 issue no wire `control_request` of their own — they read a cache populated
//! only by [`claude_runner_core::ControlSession::reinitialize`] (a real wire round trip, whose
//! subtype is `initialize` again). No mocking anywhere below — every test spawns a real
//! `claude` subprocess via [`claude_runner_core::ClaudeCommand::spawn_control_session`].

mod control_session_common;

/// IT-15: `initializationResult()` is a cached accessor — it errors before any wire round
/// trip has populated the cache, and succeeds once `reinitialize()` has run.
#[ test ]
fn it_15_initialization_result_is_a_cached_accessor()
{
  let ( session, _dir ) = control_session_common::spawn_session();

  assert!(
    session.initialization_result().is_err(),
    "before any wire round-trip, the cache must be empty"
  );

  session.reinitialize().expect( "reinitialize() must succeed to populate the cache" );

  let result = session.initialization_result()
    .expect( "initialization_result() must succeed once the cache is populated" );
  assert!( result.commands.is_array(), "commands field must be a JSON array" );
  assert!( result.account.is_object(), "account field must be a JSON object" );
}

/// IT-16: `reinitialize()` is confirmed a real wire round trip (unlike the other 4 cached
/// accessors) — its own subtype is `initialize` again, and it returns a fresh, fully-shaped
/// [`claude_runner_core::InitializeResult`].
#[ test ]
fn it_16_reinitialize_is_a_real_wire_round_trip()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  let result = session.reinitialize().expect( "reinitialize() must succeed against a real session" );
  assert!( result.commands.is_array() );
  assert!( result.agents.is_array() );
  assert!( result.models.is_array() );
  assert!( result.account.is_object() );
}

/// IT-17: `supportedCommands()` reads the cached `commands` field.
#[ test ]
fn it_17_supported_commands_reads_the_cached_commands_field()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  assert!( session.supported_commands().is_err(), "must error before any cache population" );

  session.reinitialize().expect( "reinitialize() must succeed" );
  let commands = session.supported_commands().expect( "supported_commands() must succeed once cached" );
  assert!( commands.is_array(), "commands must be a JSON array" );
  assert!( !commands.as_array().unwrap().is_empty(), "this environment has at least one slash command available" );
}

/// IT-18: `supportedModels()` reads the cached `models` field.
#[ test ]
fn it_18_supported_models_reads_the_cached_models_field()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.reinitialize().expect( "reinitialize() must succeed" );
  let models = session.supported_models().expect( "supported_models() must succeed once cached" );
  assert!( models.is_array(), "models must be a JSON array" );
  assert!( !models.as_array().unwrap().is_empty(), "this environment has at least one model available" );
}

/// IT-19: `supportedAgents()` reads the cached `agents` field.
#[ test ]
fn it_19_supported_agents_reads_the_cached_agents_field()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  session.reinitialize().expect( "reinitialize() must succeed" );
  let agents = session.supported_agents().expect( "supported_agents() must succeed once cached" );
  assert!( agents.is_array(), "agents must be a JSON array" );
  assert!( !agents.as_array().unwrap().is_empty(), "this environment has at least one subagent type available" );
}

/// IT-21: `accountInfo()` reads the cached `account` field into a typed
/// [`claude_runner_core::AccountInfo`].
#[ test ]
fn it_21_account_info_reads_the_cached_account_field()
{
  let ( session, _dir ) = control_session_common::spawn_session();
  assert!( session.account_info().is_err(), "must error before any cache population" );

  session.reinitialize().expect( "reinitialize() must succeed" );
  let account = session.account_info().expect( "account_info() must succeed once cached" );
  // email and organization are absent for API-key-authenticated sessions (only present for
  // OAuth/subscription auth); when present they must be non-empty, not blank strings.
  if let Some( email ) = &account.email
  {
    assert!( !email.is_empty(), "email, when present, must be non-empty" );
  }
  if let Some( organization ) = &account.organization
  {
    assert!( !organization.is_empty(), "organization, when present, must be non-empty" );
  }
  assert!( !account.subscription_type.is_empty(), "subscription_type must be non-empty" );
}
