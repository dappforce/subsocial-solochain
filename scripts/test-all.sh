#!/usr/bin/env bash

cargo test --release -p df-integration-tests
cargo test --release -p pallet-dotsama-claims
cargo test --release -p pallet-faucets
cargo test --release -p pallet-moderation
cargo test --release -p pallet-roles
cargo test --release -p pallet-session-keys
cargo test --release -p pallet-space-multi-ownership
#cargo test --release -p pallet-subscriptions
cargo test --release -p pallet-utils
