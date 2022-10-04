set -e

rm -f /tmp/rg-test.*

(cd interpreter_node && npm run --silent build)
(cd interpreter_rust && cargo build --release --quiet)

function perf_test {
  # Arguments
  local flags="$1"
  local name=$2
  local results=( ${@:3} )

  # Local variables
  local game=$(realpath "examples/$name")
  local ist=$(mktemp /tmp/rg-test.XXXXXXXX.ist.json)
  local out=$(mktemp /tmp/rg-test.XXXXXXXX.txt)

  # Actual script
  echo -e "\033[0;33m${name}\033[0m \033[0;36m${flags}\033[0m"
  (
    (cd interpreter_node && time=$(date +%s%N) && timeout --foreground 120 node lib/cli $flags rg-ist "$game" > $ist && time=$((($(date +%s%N) - $time) / 1000000)) && echo "  analyse time=${time}ms")
    (cd interpreter_rust && time=$(date +%s%N) && timeout --foreground 120 ./target/release/interpreter_rust $ist perf ${#results[@]} > $out && time=$((($(date +%s%N) - $time) / 1000000)) && echo "  perform time=${time}ms")
  )

  for (( depth=0; depth<${#results[@]}; ++depth )); do
    local nodes=$(grep "perf(depth: $depth)" "$out" | awk '{print $4}')
    if [ $nodes != ${results[depth]} ]; then
      echo -e "  Expected \033[0;32m${results[depth]}\033[0m nodes at depth ${depth}, but got \033[0;31m${nodes}\033[0m"
    fi
  done

  # Cleanup
  rm -f $ist $out
}

function combine() {
  eval echo $(printf "{,%s}_" $(echo "$@ " | tac -s ' '))
}

games[0]='amazons.hrg 1 2176'
#games[0]='amazons.hrg 1 2176 4307152'
#games[0]='amazons.hrg 1 80 2176' # split2 option
games[1]='breakthrough.hrg 1 22 484 11132' # 256036'
games[2]='breakthrough.rg 1 22 484 11132' # 256036'
games[3]='connect4.hrg 1 7 49 343 2401 16807' # 117649 823536 5673234'
games[4]='ticTacToe.rg 1 9 72 504 3024 15120 54720' # 148176 200448 127872'

for game in "${games[@]}"; do
  game=( $game )
  for flags in $(combine 'compactSkipEdges' 'expandGeneratorNodes' 'mangleSymbols' 'reuseFunctions'); do
    flags=( $(echo "${flags//_/ }" | xargs) )
    flags=${flags[@]/#/--}
    perf_test "$flags" "${game[0]}" ${game[@]:1}
  done
done
