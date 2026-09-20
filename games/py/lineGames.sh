# Generates HRG sources for all LineGames.

set -e

mkdir -p hrg

for script in lineGames/*.py; do
  game=$(basename "$script" .py)
  if [[ "$game" == "lineGames" ]]; then
    continue
  fi

  python3 "$script" > "hrg/${game}.hrg"
  echo "Saved ${game}.hrg"
done
