[![docs](https://github.com/radekmie/rbg-2.0/actions/workflows/docs.yml/badge.svg)](https://github.com/radekmie/rbg-2.0/actions/workflows/docs.yml)

# Setup

```sh
# Install dependencies.
npm ci

# Build project once.
npm run build

# Build project on file change.
npm run build -- --watch
```

# CLI

```
Usage: node lib/cli [options] [command] <file>

Arguments:
  file                path to game description file (.hrg or .rg)

Options:
  --compactSkipEdges  optimize automaton by compacting skip edges
  -h, --help          display help for command

Commands:
  help [command]      display help for command
  hrg-ast             print high-level Abstract Syntax Tree
  hrg-cst             print high-level Concrete Syntax Tree
  hrg-source          print high-level source
  rg-ast              print  low-level Abstract Syntax Tree
  rg-cst              print  low-level Concrete Syntax Tree
  rg-ist              print  low-level Interpreter State Tree
  rg-perf <depth>     run    low-level tree depth check
  rg-run <plays>      run    low-level simulations
  rg-source           print  low-level source
```

# UI

```sh
# Start local server on localhost:1234.
npm run ui:start
```
