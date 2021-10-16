{ pkgs ? import <nixpkgs> { } }:

with pkgs;

rustPlatform.buildRustPackage {
  name = "nix-fix-rustup";
  src = ./.;

  cargoSha256 = "sha256-zw6O58w+ted5add/BBk4Tzir0s9wKin4NuRqxmN5R8o=";

  meta = with lib; {
    description = "A tools for patching rpath and linker for Rust toolchain installed via rustup.rs under Nix environment.";
    homepage = "https://github.com/AtkinsChang/nix-fix-rustup";
    license = licenses.mit;
    maintainers = with maintainers; [ atkinschang ];
    platforms = platforms.unix;
  };
}
