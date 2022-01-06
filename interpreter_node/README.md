```sh
# Install dependencies.
$ npm ci

# Start compilation watcher.
$ npm run build

# Possible operations.
$ node lib ../examples/ticTacToe.rg perf [depth]
$ node lib ../examples/ticTacToe.rg print-ast
$ node lib ../examples/ticTacToe.rg print-cst
$ node lib ../examples/ticTacToe.rg print-ist
$ node lib ../examples/ticTacToe.rg print-source-ll
$ node lib ../examples/ticTacToe.rg run [plays]

# Built-in high-level interoperability.
# All above operations work as well.
$ node lib ../examples/breakthrough.hrg print-source-hl
```
