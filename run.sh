#!/usr/bin/env bash

set -eux

cargo build --release

exec env RUST_BACKTRACE=1 time target/release/raytracer
