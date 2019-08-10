#!/usr/bin/env bash
set -e

cargo run --bin interpreter -- "$@"
