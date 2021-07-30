{
  description = "A tools for patching rpath and linker for Rust toolchain installed via rustup.rs under Nix environment.";

  inputs =
    {
      nixpkgs.url = github:NixOS/nixpkgs/nixos-unstable;
      flake-utils.url = github:numtide/flake-utils;
    };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        rec {
          packages.nix-fix-rustup = pkgs.callPackage ./default.nix { };
          defaultPackage = packages.nix-fix-rustup;
          apps.nix-fix-rustup = flake-utils.lib.mkApp { drv = packages.nix-fix-rustup; };
          defaultApp = apps.nix-fix-rustup;
          devShell = pkgs.callPackage ./shell.nix { };
        }) // {
      overlay = final: prev: {
        nix-fix-rustup = final.callPackage ./default.nix { };
      };
    };
}

