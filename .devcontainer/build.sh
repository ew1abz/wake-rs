#!/bin/bash
echo "Check code format"       ; cargo fmt --all -- --check
echo "Build project"           ; cargo build --release
echo "Build examples"          ; cargo build --examples
echo "Run unit tests"          ; cargo test --lib --release -v --no-fail-fast -- --nocapture --test
echo "Run documentation tests" ; cargo test --doc --release -v --no-fail-fast -- --nocapture --test
