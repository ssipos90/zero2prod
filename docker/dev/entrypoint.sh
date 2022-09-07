#!/bin/sh

set -eu

. /usr/local/cargo/env

cargo install cargo-watch sqlx-cli

case "$1" in
  serve)
    shift
    cargo run -- $@
    ;;
  dev)
    shift
    cargo watch --watch-when-idle -x "run $@"
    ;;
  test)
    shift
    cargo watch --watch-when-idle -x "test -- --nocapture $@"
    ;;
  *) exec "$@";;
esac
