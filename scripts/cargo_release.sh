#!/bin/bash
set -e

# Check if version argument is provided
if [ -z "$1" ]; then
  echo "Error: Version argument is required."
  echo "Usage: $0 <version>"
  exit 1
fi

VERSION=$1

# Use the provided version in the cargo release command
cargo release "$VERSION" --execute --no-publish --no-tag --no-push
