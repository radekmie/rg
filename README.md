[![docs](https://github.com/radekmie/rg/actions/workflows/docs.yml/badge.svg)](https://github.com/radekmie/rg/actions/workflows/docs.yml)

# Regular Games repo

## Setup

### Dependencies

- [Node.js](https://nodejs.org/en/) 18.15.0
- [Rust](https://www.rust-lang.org) 1.68.0
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) 0.10.3

#### Manual installation

```sh
# System-wide tools on Debian-based systems
apt update
apt install curl gcc libssl-dev pkg-config

# Node.js via nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.3/install.sh | bash
source ~/.bashrc
nvm install 18.15.0

# Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.bashrc
rustup install 1.68.0

# wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

#### Nix

Open a shell with all required dependencies:

```sh
nix develop
```

These dependencies are accessible only from the current shell\*, they are not installed for any user. They will be removed during the next garbage collection (`nix-collect-garbage`) (unless the shell is still open or they are referenced in any other way). See [NixOS website](https://nixos.org/) for more information (you can also install it as a secondary package manager).

\*: Obviously you can access them by file paths as long as they are in the filesystem.

### Quick start

```sh
# In interpreter_rust
wasm-pack build --out-dir ../../interpreter_node/src/wasm/interpreter --out-name index --target web interpreter
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp --out-name index --target web lsp_browser

# In interpreter_node
npm clean-install
npm run build
node lib/cli rg-source --compactSkipEdges ../examples/ticTacToe.rg > ../examples/ticTacToe.rg.ll

# In interpreter_rust
cargo run --release ../examples/ticTacToe.rg run 1000
```

## Check everything before commit

```sh
# In interpreter_rust
cargo clippy
cargo fmt
cargo test

# In interpreter_node
npm run check
npm run lint
npm test
```

## Features

| Feature                                         |    `interpreter_node`    |    `interpreter_rust`    |
| :---------------------------------------------- | :----------------------: | :----------------------: |
| Parser of GDL (Game Description Language)       | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Parser of RG (Regular Games)                    | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Parser of HRG (High-level Regular Games)        |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Parser of RBG (Regular Board Games)             |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Interpreter of the IST (Interpreter State Tree) | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Translation of RG into IST                      | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Translation of GDL into RG                      | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Translation of HRG into RG                      |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Translation of RBG into RG                      |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `addBuiltins`                    | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `addExplicitCasts`               | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `calculateSimpleApply`           | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `calculateTagIndexes`            | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `calculateUniques`               | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `compactSkipEdges`               | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `expandGeneratorNodes`           | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `joinForkSuffixes`               |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `mangleSymbols`                  | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `normalizeTypes`                 | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `pruneSingletonTypes`            | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `pruneUnreachableNodes`          | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `skipSelfAssignments`            | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Transformation `skipSelfComparisons`            | :heavy_multiplication_x: |    :heavy_check_mark:    |

## `interpreter_rust`

### Development

```sh
# Format the project.
cargo fmt

# Lint the project.
cargo clippy

# Build the `interpreter` WASM module.
wasm-pack build --out-dir ../../interpreter_node/src/wasm/interpreter --out-name index --target web interpreter

# Build the `interpreter` LSP module.
wasm-pack build --out-dir ../../interpreter_node/src/wasm/lsp --out-name index --target web lsp_browser
```

### Usage

```sh
cargo run --release game.rg run [plays]
cargo run --release game.rg perf [depth]
```

## `interpreter_node`

### Development

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

# Run tests.
npm test
```

### Usage

```
Usage: node lib/cli [options] [command]

Options:
  --addExplicitCasts       add type casts to all expressions
  --calculateSimpleApply   calculate missing @simpleApply pragmas automatically
  --calculateTagIndexes    calculate missing @tagIndex and @tagMaxIndex pragmas automatically
  --calculateUniques       calculate missing @unique pragmas automatically
  --compactSkipEdges       optimize automaton by compacting skip edges
  --expandGeneratorNodes   expand generator nodes
  --inlineReachability     inline reachability when possible
  --joinForkSuffixes       join paths with identical labels leading to the same node
  --mangleSymbols          mangle all user-defined symbols
  --normalizeTypes         normalize all types so Arrow types appear only in type definitions and are at most one level deep
  --pruneSingletonTypes    prune singleton types (i.e., Set types with one element)
  --pruneUnreachableNodes  prune unused nodes
  --reuseFunctions         reuse subautomatons when translating function calls (.hrg only)
  --skipSelfAssignments    replaces all self assignments (e.g., `x = x`) with skip edges
  --skipSelfComparisons    replaces all self comparisons (e.g., `x == x`) with skip edges
  -h, --help               display help for command

Commands:
  help [command]          display help for command
  hrg-ast <file>          print .hrg Abstract Syntax Tree
  hrg-cst <file>          print .hrg Concrete Syntax Tree
  hrg-source <file>       print .hrg source
  rbg-ast <file>          print .rbg Abstract Syntax Tree
  rbg-cst <file>          print .rbg Concrete Syntax Tree
  rbg-source <file>       print .rbg source
  rg-ast <file>           print .rg  Abstract Syntax Tree
  rg-perf <file> <depth>  run   .rg  tree depth check
  rg-run <file> <plays>   run   .rg  simulations
  rg-source <file>        print .rg  source
```
