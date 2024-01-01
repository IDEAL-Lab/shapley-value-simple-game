#!/usr/bin/env bash

set -o pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../" &> /dev/null && pwd)"

export DSS_CONFIG="$ROOT_DIR/vendor/tpch-kit/dbgen"
export DSS_PATH="$ROOT_DIR/data/tpch/data"
export PATH="$DSS_CONFIG:$PATH"

usage() {
    echo "Usage: $0 -s <scale>" 1>&2
    exit 1
}

while getopts "s:" o; do
    case "${o}" in
        s)
            scale="${OPTARG}"
            ;;
        *)
            usage
            ;;
    esac
done
shift $((OPTIND-1))

if [[ -z "$scale" ]]; then
    usage
fi

echo "==> scale = ${scale}"
echo "==> run dbgen..."
if [[ ! -d "$DSS_PATH" ]]; then
    mkdir -p "$DSS_PATH"
fi

dbgen -vf -s "$scale"

echo "==> append row id..."
for f in "$DSS_PATH"/*.tbl ; do
    table_name="$(basename "${f%.*}")"
    rm -rf "$f.tmp"
    awk "{ print \$0\"|$table_name.\"NR }" "$f" > "$f.tmp"
    mv "$f.tmp" "$f"
done

# Convert raw data to csv format
python3 $ROOT_DIR/scripts/transform_data.py "tpch" $DSS_PATH $DSS_PATH

# Delete the raw data
cd "$DSS_PATH" || exit
find . -type f -name "*.tbl" -delete