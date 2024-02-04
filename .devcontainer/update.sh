#!/bin/bash
echo "Update docker image"     ; sudo apt update ; sudo apt upgrade -y
echo "Install dev library"     ; sudo apt install libudev-dev -y
echo "Updates to rustup"       ; rustup self update
echo "Update Rust toolchain"   ; rustup update
echo "Add rustfmt"             ; rustup component add rustfmt
echo "Check Rust version"      ; rustc --version
