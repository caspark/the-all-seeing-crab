#!/usr/bin/env bash

set -eux

cargo build

exec env RUST_BACKTRACE=1 time target/debug/raytracer "$1"
