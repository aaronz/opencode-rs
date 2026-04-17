#![expect(
    clippy::expect_used,
    reason = "CLI entry points where failure should panic with clear error messages"
)]

pub(crate) mod account;
pub(crate) mod acp;
pub(crate) mod agent;
pub(crate) mod attach;
pub(crate) mod bash;
pub(crate) mod completion;
pub(crate) mod config;
pub(crate) mod db;
pub(crate) mod debug;
pub(crate) mod desktop;
pub(crate) mod export;
pub(crate) mod files;
pub(crate) mod generate;
pub(crate) mod github;
pub(crate) mod gitlab;
pub(crate) mod import;
pub(crate) mod list;
pub(crate) mod mcp;
pub(crate) mod mcp_auth;
pub(crate) mod models;
pub(crate) mod palette;
pub(crate) mod plugin;
pub(crate) mod pr;
pub(crate) mod project;
pub(crate) mod prompt;
pub(crate) mod providers;
pub(crate) mod quick;
pub(crate) mod run;
pub(crate) mod serve;
pub(crate) mod session;
pub(crate) mod shortcuts;
pub(crate) mod stats;
pub(crate) mod terminal;
pub(crate) mod thread;
pub(crate) mod ui;
pub(crate) mod uninstall;
pub(crate) mod upgrade;
pub(crate) mod web;
pub(crate) mod workspace;
pub(crate) mod workspace_serve;
