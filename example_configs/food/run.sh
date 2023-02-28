# change into directory of this script
cd "$(dirname "$0")"
trap 'kill $(jobs -p)' EXIT
python open_food_facts.py &
cargo run -- --config config.yaml