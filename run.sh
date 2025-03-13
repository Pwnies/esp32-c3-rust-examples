#!/usr/bin/env bash

set -eu -o pipefail

if [ $# -lt 1 ]; then
  echo "Usage: $0 [crate] [args..]"
  exit 1
fi

CRATE="$1"; shift

if [ -z ${ESP_LOG+x} ]; then
  export ESP_LOG=debug
fi

if [ -z ${DEBUG+x} ]; then
  cargo run -p $CRATE --release "$@"
else
  cargo run -p $CRATE "$@"
fi
