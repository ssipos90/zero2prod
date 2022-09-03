#!/bin/sh

set -eu

if [[ "$PWD" == */docker/prod ]]; then
  echo 'You have to run this from the root dir, ex: "./docker/prod/build.sh"' >&2
  exit 1
fi

full_image=zero2prod:latest

docker build \
  --file docker/prod/Dockerfile \
  $PWD \
  --tag "$full_image"
