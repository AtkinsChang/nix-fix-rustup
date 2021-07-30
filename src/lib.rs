#![cfg_attr(feature = "unstable", feature(command_access))]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic)]

pub mod error;
mod nix;

pub use self::nix::{patch_toolchain, patch_toolchain_with_options};
