#!/usr/bin/env bash

set -eux

exec env RUST_BACKTRACE=1 cargo run --release
