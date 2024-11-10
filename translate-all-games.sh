# Translate all non-RG games in the `examples` directory to RG.
#   - RBG and GDL games are translated once, using Node.js and Rust respectively.
#   - HRG games are translated twice, once with `--reuseFunctions` flag enabled
#     and once without, using Node.js.
#
# All previously translated games are removed beforehand.

set -e

rm -f examples/*.*.rg

cd interpreter_rust
for game in ../examples/*.hrg; do
  echo "${game}.rg\c"       && time=$(date +%s%N) && timeout --foreground 120 cargo run --quiet --release --bin hrg_to_rg "$game" > "${game}.rg"       && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
  echo "${game}.reuse.rg\c" && time=$(date +%s%N) && timeout --foreground 120 cargo run --quiet --release --bin hrg_to_rg "$game" > "${game}.reuse.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done

cd ../interpreter_node
for game in ../examples/*.rbg; do
  echo "${game}.rg\c" && time=$(date +%s%N) && timeout --foreground 120 node lib/cli rg-source "$game" > "${game}.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done

cd ../interpreter_rust
for game in ../examples/*.kif; do
  echo "${game}.rg\c" && time=$(date +%s%N) && timeout --foreground 120 cargo run --quiet --release --bin gdl_to_rg "$game" > "${game}.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done
