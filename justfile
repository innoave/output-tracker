#!/usr/bin/env just --justfile

set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

alias b := build
alias c := check
alias cc := code-coverage
alias d := doc
alias l := lint
alias t := test
alias tl := test-lib

# list recipies
default:
    just --list

# build the crate for debugging
build:
    cargo build --all-features

# check syntax in all targets
check:
    cargo check --all-targets --all-features

# linting code using Clippy
lint:
    cargo clippy --all-targets --all-features

# run all tests
test:
    cargo test --all-features

# run unit tests only
test-lib:
    cargo test --all-features --lib --bins

# run code coverage (does not include doc-tests)
code-coverage:
    cargo +nightly llvm-cov clean --workspace
    cargo +nightly llvm-cov --branch --all-features --no-report
    cargo +nightly llvm-cov report --html --open --ignore-filename-regex "tests|test_dsl"

# build the crate for release
build-release:
    cargo build --release

# clean the workspace
clean:
    cargo clean

# generate and open docs locally
doc:
    cargo +nightly doc --all-features --no-deps --open
