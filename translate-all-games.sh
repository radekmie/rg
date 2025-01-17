# Translate all non-RG games in the `games` directory to RG.
# All previously translated games are removed beforehand.

set -e

timeout=${1:-2s}

cd interpreter_rust
cargo build --release
./target/release/interpreter &> /dev/null || true # Preload.

function translate {
  local game="$1"

  echo "${game}\c"
  time=$(date +%s%N)

  set +e
  timeout --foreground $timeout ./target/release/interpreter source "${game}" > "${game}.rg"
  timeout_code=$?
  set -e

  if [ $timeout_code -eq 124 ] ; then
    echo " \033[01;31mtimeout\033[00m"
  else
    time=$((($(date +%s%N) - $time) / 1000000))
    echo " \033[01;32m${time}ms\033[00m"
  fi
}

rm -f ../games/**/*.*.rg
for game in ../games/**/*.{hrg,kif,rbg}; do
  translate "${game}"
done
