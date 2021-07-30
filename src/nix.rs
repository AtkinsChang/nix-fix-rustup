use std::{
    ffi::{OsStr, OsString},
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::{Path, PathBuf},
    process::{Command, Output},
};

use snafu::{ensure, ResultExt};

use crate::error;

const NIXPKGS: &str = "<nixpkgs>";

trait CommandExt {
    fn to_command_string(&self) -> String;

    fn run(&mut self) -> error::Result<Output>;

    fn try_into_stdout(&mut self) -> error::Result<Vec<u8>> {
        let output = self.run()?;

        ensure!(
            output.status.success(),
            error::CommandNotSuccess {
                command: self.to_command_string(),
                code: output.status.code(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string()
            }
        );

        Ok(output.stdout)
    }
}

impl CommandExt for Command {
    #[cfg(feature = "unstable")]
    fn to_command_string(&self) -> String {
        std::iter::once(self.get_program().to_string_lossy())
            .chain(self.get_args().map(OsStr::to_string_lossy))
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[cfg(not(feature = "unstable"))]
    fn to_command_string(&self) -> String { format!("{:?}", self) }

    fn run(&mut self) -> error::Result<Output> {
        self.output()
            .with_context(|| error::SpawnChildProcess { command: self.to_command_string() })
    }
}

fn is_executable(path: &Path) -> bool {
    matches!(
        path.metadata(),
        Ok(metadata) if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0
    )
}

fn trim_ending_whitespace(bytes: &'_ [u8]) -> &'_ [u8] {
    let mut bytes = bytes;
    loop {
        match bytes {
            [r @ .., last] if char::from(*last).is_whitespace() => bytes = r,
            _ => break,
        }
    }

    bytes
}

fn nix_path(attr_path: impl AsRef<OsStr>) -> error::Result<PathBuf> {
    let stdout = Command::new("nix-build")
        .args(&[NIXPKGS, "--no-out-link", "-A"])
        .arg(attr_path)
        .try_into_stdout()?;
    let path = Path::new(OsStr::from_bytes(trim_ending_whitespace(&stdout)));

    Ok(path.to_path_buf())
}

fn nix_dynamic_linker() -> error::Result<PathBuf> {
    let stdout = Command::new("nix")
        .args(&["eval", "--raw", "-f", NIXPKGS, "stdenv.cc.bintools.dynamicLinker"])
        .try_into_stdout()?;
    let path = Path::new(OsStr::from_bytes(trim_ending_whitespace(&stdout)));

    ensure!(
        is_executable(path),
        error::Nix { message: format!("Invalid dynamic linker: {:?}", path) }
    );

    Ok(path.to_path_buf())
}

fn apply(dir: &Path, patch: impl Fn(PathBuf) -> error::Result<()>) -> error::Result<()> {
    for entry in
        dir.read_dir().with_context(|| error::Io { actions: format!("`opendir` for {:?}", dir) })?
    {
        let entry =
            entry.with_context(|| error::Io { actions: format!("`readdir` for {:?}", dir) })?;
        if entry
            .file_type()
            .with_context(|| error::Io { actions: format!("`lstat` {:?}", entry.path()) })?
            .is_file()
        {
            patch(entry.path())?;
        }
    }

    Ok(())
}

/// Patch Rust toolchain by auto-detecting
///
/// # Errors
///
/// * if error occuring while detecting `patchelf`, linker or related
/// libraries
/// * if invalid Rust toolchain base path
/// * if error patching toolchain
pub fn patch_toolchain(path: PathBuf) -> error::Result<()> {
    let patchelf = {
        let mut path = nix_path("patchelf")?;
        path.push("bin");
        path.push("patchelf");
        ensure!(
            is_executable(&path),
            error::Nix { message: format!("{:?} is not executable", path) }
        );
        path
    };
    let linker = nix_dynamic_linker()?;
    let zlib = {
        let mut path = nix_path("zlib")?;
        path.push("lib");
        ensure!(path.is_dir(), error::Nix { message: format!("{:?} is not directory", path) });
        path
    };
    let rpath = {
        let mut result = OsString::with_capacity(zlib.as_os_str().len());
        // https://github.com/NixOS/patchelf/blob/7ec8edbe094ee13c91dadca191f92b9dfac8c0f9/src/patchelf.cc#L1331-L1332
        result.push("$ORIGIN/../lib:");
        result.push(zlib);
        result
    };

    eprintln!(
        r#"patch options:
  patchelf = {:?}
  linker   = {:?}
  rpath    = {:?}"#,
        patchelf, linker, rpath
    );

    patch_toolchain_with_options(path, patchelf, linker, rpath)
}

/// Patch Rust toolchain
///
/// # Errors
///
/// * if invalid Rust toolchain base path
/// * if error patching toolchain
pub fn patch_toolchain_with_options(
    path: PathBuf,
    patchelf: impl AsRef<OsStr>,
    linker: impl AsRef<OsStr>,
    rpath: impl AsRef<OsStr>,
) -> error::Result<()> {
    let bin = path.join("bin");
    let lib = path.join("lib");
    if !path.is_dir() || !bin.is_dir() || !lib.is_dir() {
        return error::InvalidToolchainPath { path }.fail();
    }

    eprintln!("patching toolchain {:?}", path);

    apply(&bin, |path| {
        if is_executable(&path) {
            if Command::new(patchelf.as_ref())
                .arg("--set-interpreter")
                .arg(linker.as_ref())
                .arg(&path)
                .run()?
                .status
                .success()
            {
                eprintln!("bin: {:?}", path);
            }
        }
        Ok(())
    })?;
    apply(&lib, |path| {
        if Command::new(patchelf.as_ref())
            .arg("--set-rpath")
            .arg(rpath.as_ref())
            .arg(&path)
            .run()?
            .status
            .success()
        {
            eprintln!("lib: {:?}", path);
        }
        Ok(())
    })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::trim_ending_whitespace;

    #[test]
    fn test_trim_ending_whitespace() {
        let expected = "/nix/store/doge";

        for ch in &[" ", "    ", "\n", "\t", " \n\t"] {
            let s = format!("{}{}", expected, ch);
            let b = trim_ending_whitespace(s.as_bytes());
            assert_eq!(b, expected.as_bytes());
        }
    }
}
