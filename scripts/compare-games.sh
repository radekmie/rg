# Compares different implementations of the same game (e.g., between languages).

set -e

timeout=${1:-10s}
plays=${2:-100}
arguments=(${@:3})

cd ../interpreter_rust
cargo build --release
./target/release/cli &> /dev/null || true # Preload.

function compare {
  local game=${1%%:*}
  local files=${1#*:}

  echo "${game}"
  for file in ${files//,/ }; do
    echo "  ${file}\c"
    local time=$(date +%s%N)
    local output=$(timeout --foreground $timeout ./target/release/cli run --log-every-play ${arguments[@]} "../games/${file}" ${plays})
    local time=$((($(date +%s%N) - $time) / 1000000))

    # There was no output.
    if [ -z "${output}" ] ; then
      echo " \033[01;31mtimeout\033[00m"
    else
      # Take the output after the last terminal clear (`\x1bc`).
      local output=$(echo "${output}" | sed 's/\x1bc//' | tac | sed '/after/q' | tac | sed 's/^/    /')

      # The number of plays matches the requested one, i.e., there was no timeout.
      if [[ $(echo "${output}" | head -n 1) =~ $plays ]] ; then
        echo " \033[01;32m${time}ms\033[00m"
      else
        echo " \033[01;33m${time}ms\033[00m"
      fi

      echo "${output}"
    fi
  done
}

games=(
  "Breakthrough:hrg/breakthrough.hrg,kif/breakthrough.kif,rbg/breakthrough.rbg"
  "Connect Four:hrg/connect4.hrg,kif/connect4.kif,rbg/connect4.rbg"
  "Gomoku (standard):hrg/gomoku_standard.hrg,kif/gomoku_15x15.kif,rbg/gomoku_standard.rbg"
  "Knightthrough:hrg/knightthrough.hrg,kif/knightthrough.kif,rbg/knightthrough.rbg"
  "Reversi:hrg/reversi.hrg,kif/reversi.kif,rbg/reversi.rbg"
  "Tic Tac Toe:hrg/ticTacToe.hrg,kif/ticTacToe.kif,rbg/ticTacToe.rbg"
)

for game in "${games[@]}"; do
  compare "${game}"
done
