//! Entry point: parse arguments, run, and map the outcome to an exit
//! code.

use std::{
    io::{self, Write},
    process::ExitCode,
};

use clap::{Parser, error::ErrorKind};
use overseer_launch::{Error, cli::Cli, error_record, run};

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => return clap_exit(&e),
    };

    let mut stdout = io::stdout().lock();
    let outcome = run(cli, &mut stdout);
    let _ = stdout.flush(); // nothing useful to do if stdout is gone

    match outcome {
        Ok(code) => ExitCode::from(code),
        Err(err) => report(&err),
    }
}

/// Prints the error record as the last stderr line.
fn report(err: &Error) -> ExitCode {
    eprintln!("{}", error_record(err));

    ExitCode::from(err.exit_code())
}

/// Handles clap's help, version, and usage errors.
fn clap_exit(e: &clap::Error) -> ExitCode {
    let _ = e.print(); // help to stdout, usage text to stderr

    if matches!(e.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
        return ExitCode::SUCCESS;
    }

    report(&Error::Usage(e.to_string().trim().replace('\n', " ")))
}
