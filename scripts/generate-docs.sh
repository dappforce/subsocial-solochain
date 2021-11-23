#!/usr/bin/env bash

set -e

cargo doc --frozen --release --workspace \
  --exclude pallet-moderation
