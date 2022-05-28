```sh
# Prepare IST files.
node ../interpreter_node/lib/cli rg-ist ../examples/ticTacToe.rg > ../examples/ticTacToe.ist.json

# Possible operations.
cargo run ../examples/ticTacToe.ist.json run [plays]
cargo run ../examples/ticTacToe.ist.json perf [depth]
```
