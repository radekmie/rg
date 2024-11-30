# Translate all non-RG games in the `examples` directory to RG.
#   - RBG and GDL games are translated once.
#   - HRG games are translated twice, once with `--reuse-functions` flag enabled
#     and once without.
#
# All previously translated games are removed beforehand.

set -e

cd interpreter_rust
cargo build --release
./target/release/interpreter &> /dev/null || true # Preload.

rm -f examples/*.*.rg
for game in ../examples/*.hrg; do
  echo "${game}.rg\c"       && time=$(date +%s%N) && timeout --foreground 120 ./target/release/interpreter source                   "$game" > "${game}.rg"       && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
  echo "${game}.reuse.rg\c" && time=$(date +%s%N) && timeout --foreground 120 ./target/release/interpreter source --reuse-functions "$game" > "${game}.reuse.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done

for game in ../examples/*.kif; do
  echo "${game}.rg\c" && time=$(date +%s%N) && timeout --foreground 120 ./target/release/interpreter source "$game" > "${game}.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done

for game in ../examples/*.rbg; do
  echo "${game}.rg\c" && time=$(date +%s%N) && timeout --foreground 120 ./target/release/interpreter source "$game" > "${game}.rg" && time=$((($(date +%s%N) - $time) / 1000000)) && echo " ${time}ms"
done
