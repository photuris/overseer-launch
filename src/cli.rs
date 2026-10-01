//! Command-line definitions. Help text comes from these doc comments.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Launch an overseer session from a named profile.
///
/// A profile names the commands for each role: the overseer itself, the
/// implementer, and optionally the reviewer and judge. `overseer NAME`
/// exports OVERSEER_IMPLEMENTER, OVERSEER_REVIEWER, and OVERSEER_JUDGE
/// from that profile (plus HERDR_OVERSEER=1 when HERDR_ENV=1) and
/// replaces itself with the profile's overseer command in the current
/// pane. Without NAME it uses the file's `default`.
///
/// Profiles live in $XDG_CONFIG_HOME/overseer/profiles.toml
/// (~/.config/overseer/profiles.toml; %APPDATA%\overseer\profiles.toml
/// on Windows). `overseer init` writes an example there.
///
/// stdout carries JSON. On failure the last stderr line is
/// {"error":{"type":…,"message":…}}.
///
/// Exit codes: 0 ok, 1 unexpected, 2 usage, 4 not_found. After a
/// successful launch the exit code is the overseer's own.
#[derive(Debug, Parser)]
#[command(name = "overseer", version, args_conflicts_with_subcommands = true)]
pub struct Cli {
    /// Profile to launch. Omitted: the file's `default`.
    pub profile: Option<String>,
    /// Profile file to read instead of the default location.
    #[arg(long, global = true, value_name = "PATH")]
    pub config: Option<PathBuf>,
    /// Print {"profile":…,"command":…,"env":{…}} instead of launching.
    #[arg(long)]
    pub print: bool,
    /// Indent JSON output.
    #[arg(long, global = true)]
    pub pretty: bool,
    /// A command other than launching.
    #[command(subcommand)]
    pub command: Option<Command>,
}

/// Subcommands.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Write the example profile file if none exists.
    ///
    /// Prints {"path":…}. An existing file is never overwritten: that
    /// is exit 1 with the path in the message.
    ///
    /// Example: overseer init
    Init,
}
