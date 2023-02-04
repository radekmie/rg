[![docs](https://github.com/radekmie/rg/actions/workflows/docs.yml/badge.svg)](https://github.com/radekmie/rg/actions/workflows/docs.yml)

# Regular Games repo

Requirements:

- [Node.js](https://nodejs.org/en/) 18.13.0
- [Rust](https://www.rust-lang.org) 1.65.0
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/) 0.10.3

Quick setup:

```sh
# In interpreter_rust
wasm-pack wasm-pack build --out-dir ../interpreter_node/src/wasm-module --out-name index --target web

# In interpreter_node
npm run build
node lib/cli rg-ist --compactSkipEdges ../examples/ticTacToe.rg > ../examples/ticTacToe.rg.ist.json

# In interpreter_rust
cargo run ../examples/ticTacToe.ist.json run 1000
```

Check everything before commit:

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
| Parser of RG (Regular Games)                    |   :heavy_check_mark:\*   |    :heavy_check_mark:    |
| Parser of HRG (High-level Regular Games)        |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Parser of RBG (Regular Board Games)             |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Interpreter of the IST (Interpreter State Tree) | :heavy_multiplication_x: |    :heavy_check_mark:    |
| Translation of RG into IST                      |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Translation of HRG into RG                      |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Translation of RBG into RG                      |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `addExplicitCasts`               |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `compactSkipEdges`               |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `expandGeneratorNodes`           |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `joinForkSuffixes`               |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `mangleSymbols`                  |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `normalizeTypes`                 |    :heavy_check_mark:    | :heavy_multiplication_x: |
| Transformation `skipSelfAssignments`            |    :heavy_check_mark:    | :heavy_multiplication_x: |

\* It's used only for syntaxt highlighting - other operations use the one from `interpreter_rust`.

## `interpreter_rust`

### Development

```sh
# Format the project.
cargo fmt

# Lint the project.
cargo clippy

# Build WASM for `interpreter_node`.
wasm-pack wasm-pack build --out-dir ../interpreter_node/src/wasm-module --out-name index --target web
```

### Usage

```sh
cargo run game.ist.json run [plays]
cargo run game.ist.json perf [depth]
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
  --addExplicitCasts      add type casts to all expressions
  --compactSkipEdges      optimize automaton by compacting skip edges
  --expandGeneratorNodes  expand generator nodes (.hrg and .rg only)
  --inlineReachability    inline reachability when possible
  --joinForkSuffixes      join paths with identical labels leading to the same node
  --mangleSymbols         mangle all user-defined symbols
  --normalizeTypes        normalize all types so Arrow types appear only in type definitions and are at most one level deep
  --reuseFunctions        reuse subautomatons when translating function calls (.hrg only)
  --skipSelfAssignments   replaces all self assignments (e.g., `x = x`) with skip edges
  -h, --help              display help for command

Commands:
  help [command]          display help for command
  hrg-ast <file>          print .hrg Abstract Syntax Tree
  hrg-cst <file>          print .hrg Concrete Syntax Tree
  hrg-source <file>       print .hrg source
  rbg-ast <file>          print .rbg Abstract Syntax Tree
  rbg-cst <file>          print .rbg Concrete Syntax Tree
  rbg-source <file>       print .rbg source
  rg-ast <file>           print .rg  Abstract Syntax Tree
  rg-ist <file>           print .rg  Interpreter State Tree
  rg-perf <file> <depth>  run   .rg  tree depth check
  rg-run <file> <plays>   run   .rg  simulations
  rg-source <file>        print .rg  source
```
