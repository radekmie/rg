[![docs](https://github.com/radekmie/rg/actions/workflows/docs.yml/badge.svg)](https://github.com/radekmie/rg/actions/workflows/docs.yml)

# Regular Games repo

## Quick start (CLI)

```sh
# In interpreter_rust
cargo run --release run ../games/rg/ticTacToe.rg 1000
```

```sh
# cargo run help
Regular Games CLI

Usage: interpreter <COMMAND>

Commands:
  ast     Print RG AST
  perf    Benchmark game tree
  run     Benchmark random playouts
  source  Print RG source
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Quick start (GUI)

```sh
# In interpreter_rust
wasm-pack build --out-dir ../../interpreter_node/src/wasm/interpreter --out-name index --target web interpreter
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp --out-name index --target web lsp_browser

# In interpreter_node
npm clean-install
npm run start
```

## Dependencies

- [Node.js](https://nodejs.org/en/) 20.12.2
- [Rust](https://www.rust-lang.org) 1.81.0
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) 0.13.1

### Manual installation

```sh
# System-wide tools on Debian-based systems
apt update
apt install curl gcc libssl-dev pkg-config

# Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
source ~/.bashrc
nvm install 20.12.2

# Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
rustup install 1.81.0

# wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

### Nix

Open a shell with all required dependencies:

```sh
nix develop
```

These dependencies are accessible only from the current shell\*, they are not installed for any user. They will be removed during the next garbage collection (`nix-collect-garbage`) (unless the shell is still open or they are referenced in any other way). See [NixOS website](https://nixos.org/) for more information (you can also install it as a secondary package manager).

\*: Obviously you can access them by file paths as long as they are in the filesystem.

## Development

### `interpreter_rust`

```sh
# Format the project.
cargo fmt

# Lint the project.
cargo clippy

# Run tests.
cargo test

# Build the `interpreter` WASM module.
wasm-pack build --out-dir ../../interpreter_node/src/wasm/interpreter --out-name index --target web interpreter

# Build the `interpreter` LSP module.
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp --out-name index --target web lsp_browser
```

### `interpreter_node`

```sh
# Install dependencies.
npm ci

# Build project once.
npm run build

# Build project on file change.
npm run watch

# Start server on localhost:1234 and refresh on file change.
npm start

# Lint the project.
npm run lint

# Check the project.
npm run check
```
