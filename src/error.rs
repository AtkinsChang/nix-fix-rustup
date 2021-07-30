use std::{borrow::Cow, io, path::Path};

use snafu::Snafu;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub")]
pub enum Error {
    #[snafu(display("Could not spawn command `{}`: {}", command, source))]
    SpawnChildProcess { command: Cow<'static, str>, source: io::Error },

    #[snafu(display("Command {}\n{}\n{}", fmt_code(*code), command, stderr))]
    CommandNotSuccess { command: Cow<'static, str>, code: Option<i32>, stderr: Cow<'static, str> },

    #[snafu(display("{}", message))]
    Nix { message: Cow<'static, str> },

    #[snafu(display("Invalid toolchain path {:?}", path))]
    InvalidToolchainPath { path: Cow<'static, Path> },

    #[snafu(display("Failed to {}: {}", actions, source))]
    Io { actions: Cow<'static, str>, source: io::Error },
}

fn fmt_code(code: Option<i32>) -> String {
    match code {
        Some(code) => format!("exited with status code: {}", code),
        None => format!("terminated by signal"),
    }
}
