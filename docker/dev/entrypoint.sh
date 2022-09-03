#!/bin/sh

set -eu

. /usr/local/cargo/env

cargo install cargo-watch sqlx-cli

case "$1" in
  serve)
    shift
    exec cargo watch --watch-when-idle -x "run $@"
    ;;
  test)
    shift
    exec cargo watch -x "test -- --nocapture $@"
    ;;
  *) exec "$@";;
esac
