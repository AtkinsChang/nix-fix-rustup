use std::{io, path::PathBuf, process};

use structopt::{clap::Shell, StructOpt};

use nix_fix_rustup::{error, patch_toolchain};

#[derive(Debug, StructOpt)]
pub struct PathConfig {
    /// The path of toolchain
    #[structopt(parse(from_os_str))]
    toolchain_path: PathBuf,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "fix-nixpkgs-rustup",
    about = "A tools for patching rpath and linker for Rust toolchain installed via rustup.rs \
             under Nix environment."
)]
pub enum Command {
    #[structopt(about = "Shows current version")]
    Version,

    #[structopt(about = "Shows shell completions")]
    Completions { shell: Shell },

    #[structopt(about = "Fix Rust toolchain")]
    Patch(Box<PathConfig>),
}

impl Command {
    #[inline]
    pub fn app_name() -> String { Command::clap().get_name().to_string() }

    pub fn run(self) -> error::Result<()> {
        match self {
            Command::Version => {
                Command::clap()
                    .write_version(&mut io::stdout())
                    .expect("failed to write to stdout");
                Ok(())
            }
            Command::Completions { shell } => {
                let app_name = Command::app_name();
                Command::clap().gen_completions_to(app_name, shell, &mut io::stdout());
                Ok(())
            }
            Command::Patch(config) => patch_toolchain(config.toolchain_path),
        }
    }
}

fn main() {
    if let Err(err) = Command::from_args().run() {
        eprintln!("{}", err);
        process::exit(-87);
    }
}
