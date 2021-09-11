#!/usr/bin/env bash

set -eux

exec cargo watch --shell "cargo test && ./run.sh target/output.png"
