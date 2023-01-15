[![docs](https://github.com/radekmie/rbg-2.0/actions/workflows/docs.yml/badge.svg)](https://github.com/radekmie/rbg-2.0/actions/workflows/docs.yml)

# Setup

```sh
# Install dependencies.
npm ci

# Build project once.
npm run build

# Build project on file change.
npm run build -- --watch

# Run tests.
npm test
```

# CLI

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
  rg-cst <file>           print .rg  Concrete Syntax Tree
  rg-ist <file>           print .rg  Interpreter State Tree
  rg-perf <file> <depth>  run   .rg  tree depth check
  rg-run <file> <plays>   run   .rg  simulations
  rg-source <file>        print .rg  source
```

# UI

```sh
# Start local server on localhost:1234.
npm run ui:start
```
