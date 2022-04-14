#!/bin/bash

cargo check --workspace
cargo test --workspace --verbose
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings -W clippy::all