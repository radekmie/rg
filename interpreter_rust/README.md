```sh
# Prepare IST files.
$ node ../interpreter_node/lib ../examples/ticTacToe.rg print-ist > ../examples/ticTacToe.ist.json

# Possible operations.
$ cargo run ../examples/ticTacToe.ist.json run [plays]
$ cargo run ../examples/ticTacToe.ist.json perf [depth]
```
