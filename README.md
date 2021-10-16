# nix-fix-rustup

A tools for patching rpath and linker for Rust toolchain installed via rustup.rs under Nix environment.

## Usage

Run binary after download it

```bash
nix-fix-rustup patch ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu
```

or using nix

```bash
nix run github:AtkinsChang/nix-fix-rustup patch ~/.rustup/toolchains/stable-x86_64-unknown-linux-gnu
```
