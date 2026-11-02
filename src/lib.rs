//! `overseer`: launch an overseer session from a named profile.
//!
//! A profile is a set of role commands in a TOML file. Launching one
//! exports the roster environment the overseer skill reads and execs
//! the profile's overseer command. See [`cli::Cli`] for the interface.

pub mod cli;
pub mod launch;
pub mod profiles;

use std::{env, fmt, io::Write};

use serde::Serialize;
use serde_json::json;

use cli::{Cli, Command};
use profiles::Profiles;

/// Every failure the CLI reports, each with a fixed exit code.
#[derive(Debug)]
pub enum Error {
    /// Anything that should not happen; exit 1.
    Unexpected(String),
    /// Bad arguments or an invalid profile file; exit 2.
    Usage(String),
    /// A missing file or unknown profile; exit 4.
    NotFound(String),
}

impl Error {
    /// The process exit code for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Unexpected(_) => 1,
            Self::Usage(_) => 2,
            Self::NotFound(_) => 4,
        }
    }

    /// The `error.type` string for this error.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Unexpected(_) => "unexpected",
            Self::Usage(_) => "usage",
            Self::NotFound(_) => "not_found",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected(m) | Self::Usage(m) | Self::NotFound(m) => {
                f.write_str(m)
            }
        }
    }
}

impl std::error::Error for Error {}

/// Result alias over [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The JSON error record printed as the last stderr line.
pub fn error_record(err: &Error) -> String {
    json!({
        "error": { "type": err.kind(), "message": err.to_string() }
    })
    .to_string()
}

/// Runs one parsed command. Returns the exit code to finish with.
///
/// # Errors
///
/// Any [`Error`] from loading the profile file, resolving the profile,
/// launching, or writing to `out`.
pub fn run(cli: Cli, out: &mut dyn Write) -> Result<u8> {
    let path = match cli.config {
        Some(path) => path,
        None => profiles::default_path()?,
    };

    if let Some(Command::Init) = cli.command {
        launch::init(out, &path, cli.pretty)?;

        return Ok(0);
    }

    let profiles = Profiles::load(&path)?;
    let (name, profile) = profiles.resolve(cli.profile.as_deref())?;
    let herdr = in_herdr(
        env::var("HERDR_ENV").ok().as_deref(),
        env::var("HC_TERMINAL_ID").ok().as_deref(),
    );
    let env = profile.env(herdr);

    if cli.print {
        launch::print(out, name, profile, &env, cli.pretty)?;

        return Ok(0);
    }

    launch::exec(&profile.overseer, &env)
}

/// Whether the process runs under Herdr: the TUI sets `HERDR_ENV=1`,
/// the desktop app's core sets `HC_TERMINAL_ID` in each terminal.
fn in_herdr(herdr_env: Option<&str>, hc_terminal: Option<&str>) -> bool {
    herdr_env == Some("1") || hc_terminal.is_some_and(|id| !id.is_empty())
}

/// Prints one JSON value on stdout, compact unless `pretty`.
///
/// # Errors
///
/// Writing to `out` fails.
pub fn print_one(
    out: &mut dyn Write,
    value: &impl Serialize,
    pretty: bool,
) -> Result<()> {
    let text = if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .map_err(|e| Error::Unexpected(format!("encoding output: {e}")))?;

    writeln!(out, "{text}")
        .map_err(|e| Error::Unexpected(format!("writing stdout: {e}")))
}

#[cfg(test)]
mod tests {
    mod in_herdr {
        use crate::in_herdr;

        #[test]
        fn should_be_false_when_neither_variable_is_set() {
            assert!(!in_herdr(None, None));
        }

        #[test]
        fn should_be_true_when_herdr_env_is_1() {
            assert!(in_herdr(Some("1"), None));
        }

        #[test]
        fn should_be_false_when_herdr_env_is_0() {
            assert!(!in_herdr(Some("0"), None));
        }

        #[test]
        fn should_be_true_when_hc_terminal_id_is_set() {
            assert!(in_herdr(None, Some("t7")));
        }

        #[test]
        fn should_be_false_when_hc_terminal_id_is_empty() {
            assert!(!in_herdr(None, Some("")));
        }

        #[test]
        fn should_be_true_when_both_variables_are_set() {
            assert!(in_herdr(Some("1"), Some("t7")));
        }
    }
}
