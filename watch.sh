#!/usr/bin/env bash

set -eux

exec cargo watch --shell "./run.sh"
