# Some explanation for Rust+wasm setup: https://www.tomhoule.com/2021/building-rust-wasm-with-nix-flakes/
# (made minor tweaks for newer Nix version)

# Provides all dependencies for development.
# (no build targets because sandboxing npm is hard)

{
  description = "TODO";

  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let overlays = [ rust-overlay.overlays.default ];
          pkgs = import nixpkgs { inherit system overlays; };
          rustWithWasmTarget = pkgs.rust-bin.fromRustupToolchainFile ./interpreter_rust/rust-toolchain.toml;
          rustPlatformWasm = pkgs.makeRustPlatform {
             cargo = rustWithWasmTarget;
             rustc = rustWithWasmTarget;
          };
          nodejs = pkgs.nodejs-18_x;
      in
      {
        devShell = pkgs.mkShell {
          packages = [ pkgs.wasm-pack
                       pkgs.wasm-bindgen-cli # used by wasm-pack
                       pkgs.binaryen # wasm-opt used by wasm-pack
                       rustWithWasmTarget
                       pkgs.pkg-config
                       pkgs.openssl
                       nodejs
                       ];
        };
      }
    );
}
