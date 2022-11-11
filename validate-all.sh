set -e

(cd interpreter_node && npm run --silent build)

function perf_test {
  # Arguments
  local flags="$1"
  local name=$2

  # Local variables
  local game=$(realpath "examples/$name")

  # Actual script
  echo "\033[0;33m${name}\033[0m\033[0;36m${flags}\033[0m\c"
  (cd interpreter_node && time=$(date +%s%N) && timeout --foreground 120 node lib/cli $flags rg-source "$game" > /dev/null && time=$((($(date +%s%N) - $time) / 1000000)) && echo " analyse time=${time}ms")
}

function combine() {
  eval echo $(printf "{,%s}_" $(echo "$@ " | tac -s ' '))
}

for game in ./examples/*.{hrg,rbg,rg}; do
  for flags in $(combine 'compactSkipEdges' 'expandGeneratorNodes' 'mangleSymbols' 'reuseFunctions'); do
    flags=( $(echo "${flags//_/ }" | xargs) )
    flags=${flags[@]/#/--}
    if [ ! -z "$flags" ]; then
      flags=" $flags"
    fi

    perf_test "$flags" "${game/\.\/examples\//}"
  done
done
