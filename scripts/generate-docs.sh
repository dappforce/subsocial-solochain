#!/usr/bin/env bash

set -e

cargo doc --frozen --release --workspace \
  --exclude pallet-donations \
  --exclude pallet-moderation \
  --exclude pallet-session-keys \
  --exclude pallet-space-multi-ownership \
  --exclude pallet-subscriptions