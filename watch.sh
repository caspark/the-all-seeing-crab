#!/usr/bin/env bash

set -eux

exec cargo watch --ignore '*.ppm' --shell ./run.sh
