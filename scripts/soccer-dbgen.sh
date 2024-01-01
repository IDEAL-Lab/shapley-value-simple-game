set -o pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../" &> /dev/null && pwd)"

# Preprocess data
export DSS_PATH="$ROOT_DIR/data/soccer/data"
python3 $ROOT_DIR/scripts/transform_data.py "soccer" $DSS_PATH $DSS_PATH