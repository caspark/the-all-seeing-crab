#!/usr/bin/env bash

set -eux

exec cargo watch --ignore 'output.png' --shell ./run.sh
