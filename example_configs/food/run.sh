# change into directory of this script
cd "$(dirname "$0")"
python open_food_facts.py &
cargo run -- --config config.yaml