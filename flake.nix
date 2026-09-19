{
  description = "Klor keyboard firmware development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" "llvm-tools-preview" ];
          targets = [ "thumbv6m-none-eabi" ];
        };
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            rustToolchain
            pkgs.taplo
            pkgs.cargo-binutils
            pkgs.elf2uf2-rs
            pkgs.flip-link
          ];
        };
      });
}
