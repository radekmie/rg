# Translate all non-RG games in the `games` directory to RG.
#   - RBG and GDL games are translated once.
#   - HRG games are translated twice, once with `--reuse-functions` flag enabled
#     and once without.
#
# All previously translated games are removed beforehand.

set -e

timeout=${1:-2s}

cd interpreter_rust
cargo build --release
./target/release/interpreter &> /dev/null || true # Preload.

function translate {
  local source=$1
  local target=$2
  local args=($3)

  echo "$target\c"
  time=$(date +%s%N)

  set +e
  timeout --foreground $timeout ./target/release/interpreter source "${args[@]}" "${source}" > "${target}"
  timeout_code=$?
  set -e

  if [ $timeout_code -eq 124 ] ; then
    echo " \033[01;31mtimeout\033[00m"
  else
    time=$((($(date +%s%N) - $time) / 1000000))
    echo " \033[01;32m${time}ms\033[00m"
  fi
}

rm -f games/**/*.*.rg
for game in ../games/**/*.{hrg,kif,rbg}; do
  translate "${game}" "${game}.rg" ""
  if [[ $game == *.hrg ]] ; then
    translate "${game}" "${game}.reuse.rg" "--reuse-functions"
  fi
done
