#[cfg(test)]
pub mod common;

#[cfg(test)]
pub mod conventions;

#[cfg(test)]
pub mod event_emitter_tests;

#[cfg(test)]
pub mod status_endpoint_tests;

#[cfg(test)]
pub mod phase6_regression_tests;

#[cfg(test)]
pub mod plugin_hook_tests;

#[cfg(test)]
pub mod security_tests;

#[cfg(test)]
pub mod permission_integration_tests;
#[cfg(test)]
pub mod session_lifecycle_tests;
#[cfg(test)]
pub mod session_storage_tests;
#[cfg(test)]
pub mod tool_registry_audit_tests;

#[cfg(test)]
pub mod ws_streaming_tests;

#[cfg(test)]
pub mod test_websocket;
