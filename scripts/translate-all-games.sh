# Translate all non-RG games in the `games` directory to RG.
# All previously translated games are removed beforehand.

set -e

timeout=${1:-10s}
arguments=(${@:2})

cd ../interpreter_rust
cargo build --release
./target/release/cli &> /dev/null || true # Preload.

function translate {
  local game="$1"

  echo "${game}\c"
  local time=$(date +%s%N)

  set +e
  timeout --foreground $timeout ./target/release/cli source ${arguments[@]} "${game}" > "${game}.rg"
  local status=$?
  local time=$((($(date +%s%N) - $time) / 1000000))
  set -e

  if [ $status -eq 124 ] ; then
    echo " \033[01;31mtimeout\033[00m"
  else
    echo " \033[01;32m${time}ms\033[00m"
  fi
}

rm -f ../games/**/*.*.rg
for game in ../games/**/*.{hrg,kif,rbg}; do
  translate "${game}"
done
