#!/usr/bin/env bash

set -o pipefail

ROOT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../" &> /dev/null && pwd)"

if [[ "$(uname -s)" = Darwin ]]; then
    OS=MACOS
else
    OS=LINUX
fi

cd "$ROOT_DIR/vendor/tpch-kit/dbgen"
make MACHINE="$OS" DATABASE=POSTGRESQL