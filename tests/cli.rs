//! CLI tests: exit codes, error records, and output shape. They run the
//! built binary with `--print` and never launch an agent.

use std::{
    env, fs,
    path::PathBuf,
    process::{Command, Output},
};

/// The fixture profile file.
const FIXTURE: &str = "tests/fixtures/profiles.toml";

/// Runs the binary with `args` and a clean roster environment.
fn overseer(args: &[&str], herdr_env: Option<&str>) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_overseer"));

    command.args(args).env_remove("HERDR_ENV");

    if let Some(value) = herdr_env {
        command.env("HERDR_ENV", value);
    }

    command
        .output()
        .unwrap_or_else(|e| panic!("running the binary: {e}"))
}

/// Parses stdout as one JSON value.
fn stdout_json(output: &Output) -> serde_json::Value {
    let text = String::from_utf8_lossy(&output.stdout);

    serde_json::from_str(&text)
        .unwrap_or_else(|e| panic!("stdout {text:?}: {e}"))
}

/// The error record on the last stderr line.
fn error_record(output: &Output) -> serde_json::Value {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let last = stderr.lines().last().unwrap_or_default();

    serde_json::from_str(last)
        .unwrap_or_else(|e| panic!("last stderr line {last:?}: {e}"))
}

#[test]
fn should_print_named_profile_when_name_given() {
    let output = overseer(&["open", "--config", FIXTURE, "--print"], None);
    let value = stdout_json(&output);

    assert_eq!(
        (
            output.status.code(),
            value["profile"].as_str(),
            value["command"].as_str(),
            value["env"]["OVERSEER_IMPLEMENTER"].as_str(),
        ),
        (
            Some(0),
            Some("open"),
            Some("pi --model code/kimi-k3"),
            Some("pi --model code/glm-5.3"),
        )
    );
}

#[test]
fn should_print_default_profile_when_no_name() {
    let output = overseer(&["--config", FIXTURE, "--print"], None);
    let value = stdout_json(&output);

    assert_eq!(value["profile"].as_str(), Some("claude"));
}

#[test]
fn should_omit_herdr_signal_when_not_in_herdr() {
    let output = overseer(&["--config", FIXTURE, "--print"], None);
    let value = stdout_json(&output);

    assert!(value["env"].get("HERDR_OVERSEER").is_none(), "env: {value}");
}

#[test]
fn should_add_herdr_signal_when_herdr_env_is_1() {
    let output = overseer(&["--config", FIXTURE, "--print"], Some("1"));
    let value = stdout_json(&output);

    assert_eq!(value["env"]["HERDR_OVERSEER"].as_str(), Some("1"));
}

#[test]
fn should_exit_4_listing_names_when_profile_unknown() {
    let output = overseer(&["nope", "--config", FIXTURE, "--print"], None);
    let record = error_record(&output);

    assert_eq!(
        (
            output.status.code(),
            record["error"]["type"].as_str(),
            record["error"]["message"].as_str(),
        ),
        (
            Some(4),
            Some("not_found"),
            Some("unknown profile \"nope\"; known: claude, codex, open"),
        )
    );
}

#[test]
fn should_exit_4_when_profile_file_missing() {
    let output =
        overseer(&["--config", "tests/fixtures/none.toml", "--print"], None);
    let record = error_record(&output);

    assert_eq!(
        (output.status.code(), record["error"]["type"].as_str()),
        (Some(4), Some("not_found"))
    );
}

#[test]
fn should_exit_2_when_flag_unknown() {
    let output = overseer(&["--bogus"], None);
    let record = error_record(&output);

    assert_eq!(
        (output.status.code(), record["error"]["type"].as_str()),
        (Some(2), Some("usage"))
    );
}

#[test]
fn should_write_example_then_refuse_when_init_run_twice() {
    let dir =
        env::temp_dir().join(format!("overseer-init-{}", std::process::id()));
    let path: PathBuf = dir.join("nested").join("profiles.toml");
    let path_str = path.to_string_lossy().into_owned();
    let first = overseer(&["init", "--config", &path_str], None);
    let written = fs::read_to_string(&path).unwrap_or_default();
    let second = overseer(&["init", "--config", &path_str], None);

    let _ = fs::remove_dir_all(&dir); // best effort, the assert matters

    assert_eq!(
        (
            first.status.code(),
            stdout_json(&first)["path"].as_str(),
            written == include_str!("../profiles.example.toml"),
            second.status.code(),
            error_record(&second)["error"]["type"].as_str(),
        ),
        (
            Some(0),
            Some(path_str.as_str()),
            true,
            Some(1),
            Some("unexpected")
        )
    );
}

/// A profile whose overseer is a shell line that reports the roster it
/// sees and exits 7. Unix only: it relies on `sh`.
#[cfg(unix)]
#[test]
fn should_clear_inherited_roster_and_pass_exit_code_when_launching() {
    let dir =
        env::temp_dir().join(format!("overseer-exec-{}", std::process::id()));
    let path = dir.join("profiles.toml");
    let profile = concat!(
        "default = \"t\"\n[profiles.t]\n",
        "overseer = \"echo J=${OVERSEER_JUDGE:-unset} R=$OVERSEER_REVIEWER; exit 7\"\n",
        "implementer = \"impl\"\nreviewer = \"rev\"\n",
    );

    fs::create_dir_all(&dir).unwrap();
    fs::write(&path, profile).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_overseer"))
        .args(["--config", &path.to_string_lossy()])
        .env_remove("HERDR_ENV")
        .env("OVERSEER_JUDGE", "leaked")
        .output()
        .unwrap();

    let _ = fs::remove_dir_all(&dir); // best effort, the assert matters

    assert_eq!(
        (
            output.status.code(),
            String::from_utf8_lossy(&output.stdout).trim(),
        ),
        (Some(7), "J=unset R=rev")
    );
}

#[test]
fn should_document_exit_codes_in_top_level_help() {
    let output = overseer(&["--help"], None);
    let help = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success()
            && help.contains("Exit codes:")
            && help.contains("init"),
        "help was {help:?}"
    );
}
