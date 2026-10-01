//! Launching the overseer, printing what would be launched, and writing
//! the example profile file.

use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use serde_json::json;

use crate::{
    Error, Result, print_one,
    profiles::{Profile, ROSTER_VARS},
};

/// The example profile file written by `overseer init`.
const EXAMPLE: &str = include_str!("../profiles.example.toml");

/// Prints the resolved launch as JSON instead of performing it.
///
/// # Errors
///
/// Writing to `out` fails.
pub fn print(
    out: &mut dyn Write,
    name: &str,
    profile: &Profile,
    env: &BTreeMap<String, String>,
    pretty: bool,
) -> Result<()> {
    let record = json!({
        "profile": name,
        "command": profile.overseer,
        "env": env,
    });

    print_one(out, &record, pretty)
}

/// Replaces this process with `command` under the platform shell, with
/// `env` as the whole roster environment: inherited roster variables
/// that the profile does not set are removed, so a profile without a
/// reviewer really has none. On Unix this never returns on success; on
/// Windows it waits and returns the command's exit code.
///
/// # Errors
///
/// [`Error::Unexpected`] when the shell cannot be started.
pub fn exec(command: &str, env: &BTreeMap<String, String>) -> Result<u8> {
    let mut shell = shell_command(command);

    for var in ROSTER_VARS {
        shell.env_remove(var);
    }

    shell
        .envs(env)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit());

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;

        let err = shell.exec();

        Err(Error::Unexpected(format!("exec {command:?}: {err}")))
    }

    #[cfg(not(unix))]
    {
        let status = shell.status().map_err(|e| {
            Error::Unexpected(format!("running {command:?}: {e}"))
        })?;
        let code = status.code().unwrap_or(1);

        Ok(u8::try_from(code).unwrap_or(1))
    }
}

/// The platform shell invocation for one command string.
fn shell_command(command: &str) -> Command {
    let mut shell = if cfg!(windows) {
        let mut c = Command::new("cmd");

        c.args(["/C", command]);
        c
    } else {
        let mut c = Command::new("sh");

        c.args(["-c", command]);
        c
    };

    shell.stderr(Stdio::inherit());
    shell
}

/// Writes the example profile file to `path`, creating parent
/// directories. Prints `{"path":…}`.
///
/// # Errors
///
/// [`Error::Unexpected`] when the file already exists or cannot be
/// written.
pub fn init(out: &mut dyn Write, path: &Path, pretty: bool) -> Result<()> {
    let shown = path.display();

    if path.exists() {
        return Err(Error::Unexpected(format!(
            "{shown} already exists; edit it or move it aside"
        )));
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Error::Unexpected(format!("creating {}: {e}", parent.display()))
        })?;
    }

    fs::write(path, EXAMPLE)
        .map_err(|e| Error::Unexpected(format!("writing {shown}: {e}")))?;

    print_one(out, &json!({ "path": path }), pretty)
}
