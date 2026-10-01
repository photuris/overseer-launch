//! The profile file: its shape, where it lives, and how a profile turns
//! into a roster environment.

use std::{collections::BTreeMap, env, fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Roster variable for the implementer command.
const IMPLEMENTER: &str = "OVERSEER_IMPLEMENTER";
/// Roster variable for the reviewer command.
const REVIEWER: &str = "OVERSEER_REVIEWER";
/// Roster variable for the judge command.
const JUDGE: &str = "OVERSEER_JUDGE";
/// Herdr's overseer-mode signal.
const HERDR_OVERSEER: &str = "HERDR_OVERSEER";
/// Every variable a launch owns. A launch clears these before adding
/// the profile's own, so nothing leaks in from the parent shell.
pub const ROSTER_VARS: [&str; 4] =
    [IMPLEMENTER, REVIEWER, JUDGE, HERDR_OVERSEER];

/// The whole profile file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profiles {
    /// Profile used when none is named.
    pub default: Option<String>,
    /// Profiles by name.
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

/// One profile: a command per role.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Command that runs the overseer itself.
    pub overseer: String,
    /// Command that launches an implementer.
    pub implementer: String,
    /// Command that launches the reviewer roles; none means no review.
    pub reviewer: Option<String>,
    /// Command for the judge; none means no judge.
    pub judge: Option<String>,
}

impl Profiles {
    /// Reads and parses the file at `path`.
    ///
    /// # Errors
    ///
    /// [`Error::NotFound`] when the file does not exist,
    /// [`Error::Usage`] when it is not valid, and [`Error::Unexpected`]
    /// for any other read failure.
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let text = fs::read_to_string(path).map_err(|e| {
            let shown = path.display();

            match e.kind() {
                io::ErrorKind::NotFound => Error::NotFound(format!(
                    "no profile file at {shown}; run `overseer init`"
                )),
                _ => Error::Unexpected(format!("reading {shown}: {e}")),
            }
        })?;

        Self::parse(&text)
            .map_err(|e| Error::Usage(format!("{}: {e}", path.display())))
    }

    /// Parses the file's text.
    ///
    /// # Errors
    ///
    /// The TOML error when the text is not a valid profile file.
    pub fn parse(text: &str) -> std::result::Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Finds the profile called `name`, or the default when `name` is
    /// `None`.
    ///
    /// # Errors
    ///
    /// [`Error::Usage`] when nothing is named and the file has no
    /// `default`; [`Error::NotFound`] when the name is unknown.
    pub fn resolve(&self, name: Option<&str>) -> Result<(&str, &Profile)> {
        let name = name.or(self.default.as_deref()).ok_or_else(|| {
            Error::Usage("no profile named and the file has no default".into())
        })?;
        let (name, profile) =
            self.profiles.get_key_value(name).ok_or_else(|| {
                let known: Vec<&str> =
                    self.profiles.keys().map(String::as_str).collect();

                Error::NotFound(format!(
                    "unknown profile {name:?}; known: {}",
                    known.join(", ")
                ))
            })?;

        Ok((name.as_str(), profile))
    }
}

impl Profile {
    /// The roster variables this profile exports. `herdr` adds the
    /// Herdr overseer-mode signal.
    pub fn env(&self, herdr: bool) -> BTreeMap<String, String> {
        let mut env = BTreeMap::new();

        env.insert(IMPLEMENTER.into(), self.implementer.clone());

        if let Some(reviewer) = &self.reviewer {
            env.insert(REVIEWER.into(), reviewer.clone());
        }

        if let Some(judge) = &self.judge {
            env.insert(JUDGE.into(), judge.clone());
        }

        if herdr {
            env.insert(HERDR_OVERSEER.into(), "1".into());
        }

        env
    }
}

/// The platform's default profile file location.
///
/// # Errors
///
/// [`Error::Unexpected`] when neither the XDG nor the home variable is
/// set, so no location can be derived.
pub fn default_path() -> Result<PathBuf> {
    let base = if cfg!(windows) {
        env::var_os("APPDATA").map(PathBuf::from)
    } else {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| {
                env::var_os("HOME").map(|h| PathBuf::from(h).join(".config"))
            })
    };
    let base = base.ok_or_else(|| {
        Error::Unexpected(
            "no config directory: set XDG_CONFIG_HOME, HOME, or APPDATA"
                .into(),
        )
    })?;

    Ok(base.join("overseer").join("profiles.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A file with one full profile and one minimal one.
    const TEXT: &str = r#"
default = "full"

[profiles.full]
overseer = "claude"
implementer = "claude --model x"
reviewer = "codex"
judge = "overseer-judge"

[profiles.bare]
overseer = "pi"
implementer = "pi --model y"
"#;

    mod parse {
        use super::*;

        #[test]
        fn should_reject_unknown_key_when_present() {
            let err = Profiles::parse(
                "[profiles.a]\noverseer='x'\nimplementer='y'\ncolour='z'\n",
            )
            .err()
            .map(|e| e.to_string())
            .unwrap_or_default();

            assert!(err.contains("colour"), "error was {err:?}");
        }

        #[test]
        fn should_reject_profile_when_implementer_missing() {
            let err = Profiles::parse("[profiles.a]\noverseer='x'\n")
                .err()
                .map(|e| e.to_string())
                .unwrap_or_default();

            assert!(err.contains("implementer"), "error was {err:?}");
        }
    }

    mod resolve {
        use super::*;

        #[test]
        fn should_use_default_when_no_name() {
            let profiles = Profiles::parse(TEXT).unwrap();
            let (name, _) = profiles.resolve(None).unwrap();

            assert_eq!(name, "full");
        }

        #[test]
        fn should_exit_usage_when_no_name_and_no_default() {
            let profiles = Profiles::parse(
                "[profiles.a]\noverseer='x'\nimplementer='y'\n",
            )
            .unwrap();
            let err = profiles.resolve(None).unwrap_err();

            assert!(matches!(err, Error::Usage(_)), "error was {err:?}");
        }

        #[test]
        fn should_list_known_names_when_name_unknown() {
            let profiles = Profiles::parse(TEXT).unwrap();
            let err = profiles.resolve(Some("nope")).unwrap_err();

            assert_eq!(
                err.to_string(),
                "unknown profile \"nope\"; known: bare, full"
            );
        }
    }

    mod env {
        use super::*;

        #[test]
        fn should_omit_optional_roles_when_unset() {
            let profiles = Profiles::parse(TEXT).unwrap();
            let (_, bare) = profiles.resolve(Some("bare")).unwrap();
            let keys: Vec<String> = bare.env(false).into_keys().collect();

            assert_eq!(keys, vec!["OVERSEER_IMPLEMENTER"]);
        }

        #[test]
        fn should_add_herdr_signal_when_asked() {
            let profiles = Profiles::parse(TEXT).unwrap();
            let (_, full) = profiles.resolve(Some("full")).unwrap();
            let env = full.env(true);

            assert_eq!(
                env.get("HERDR_OVERSEER").map(String::as_str),
                Some("1")
            );
        }
    }
}
