#!/bin/bash
set -e

TAP_DIR=$(basename "$TAP_REPO")
AMD64_URL="https://github.com/${REPOSITORY}/releases/download/${RELEASE_TAG}/${AMD64_NAME}"
ARM64_URL="https://github.com/${REPOSITORY}/releases/download/${RELEASE_TAG}/${ARM64_NAME}"
TEMPLATE_FILE_PATH="$(dirname "$0")/kimspect.rb.tpl"

echo "Cloning tap repository $TAP_REPO..."
# Use the passed TAP_TOKEN for authentication
git clone "https://x-access-token:${TAP_TOKEN}@github.com/${TAP_REPO}.git" --branch main "$TAP_DIR"

cd "$TAP_DIR"

echo "Generating formula $FORMULA_PATH from template for version $RELEASE_TAG..."

# Export variables for envsubst with names matching the template placeholders
export version="$RELEASE_TAG"
export amd64_url="$AMD64_URL"
export amd64_sha256="$AMD64_SHA"
export arm64_url="$ARM64_URL"
export arm64_sha256="$ARM64_SHA"

# Check if template file exists
if [ ! -f "$TEMPLATE_FILE_PATH" ]; then
    echo "Error: Template file not found at $TEMPLATE_FILE_PATH"
    exit 1
fi

# Use envsubst to substitute variables in the template and output to the formula path
# Specify the variables to substitute explicitly
envsubst '${version} ${amd64_url} ${amd64_sha256} ${arm64_url} ${arm64_sha256}' < "$TEMPLATE_FILE_PATH" > "$FORMULA_PATH"

echo "Formula file $FORMULA_PATH generated."

# Unset exported variables
unset version amd64_url amd64_sha256 arm64_url arm64_sha256

echo "Committing and pushing changes..."
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git status
git add "$FORMULA_PATH"

if git diff --staged --quiet; then
  echo "No changes detected in the formula file $FORMULA_PATH after generation. Exiting."
  # Exit cleanly if no changes are detected
  exit 0
else
  git commit -m "$COMMIT_MESSAGE"
  git push
  echo "Changes pushed successfully."
fi
