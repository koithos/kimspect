#!/bin/bash
set -e # Exit immediately if a command exits with a non-zero status.

TAP_REPO="$1"
FORMULA_PATH="$2"
RELEASE_TAG="$3"
REPOSITORY="$4"
AMD64_SHA="$5"
ARM64_SHA="$6"
COMMIT_MESSAGE="$7"
TAP_TOKEN="$8" # Accept tap_token as an argument

TAP_DIR=$(basename "$TAP_REPO")
AMD64_URL="https://github.com/${REPOSITORY}/releases/download/${RELEASE_TAG}/kelper-x86_64-apple-darwin"
ARM64_URL="https://github.com/${REPOSITORY}/releases/download/${RELEASE_TAG}/kelper-aarch64-apple-darwin"

echo "Cloning tap repository $TAP_REPO..."
# Use the passed TAP_TOKEN for authentication
git clone "https://x-access-token:${TAP_TOKEN}@github.com/${TAP_REPO}.git" "$TAP_DIR"
cd "$TAP_DIR"

echo "Updating formula $FORMULA_PATH for version $RELEASE_TAG..."

# Use '#' as delimiter for sed to avoid conflicts with '/' in URLs
sed -i.bak "s#{{version}}#${RELEASE_TAG}#g" "$FORMULA_PATH"
sed -i.bak "s#{{amd64_url}}#${AMD64_URL}#g" "$FORMULA_PATH"
sed -i.bak "s#{{amd64_sha256}}#${AMD64_SHA}#g" "$FORMULA_PATH"
sed -i.bak "s#{{arm64_url}}#${ARM64_URL}#g" "$FORMULA_PATH"
sed -i.bak "s#{{arm64_sha256}}#${ARM64_SHA}#g" "$FORMULA_PATH"
rm "${FORMULA_PATH}.bak" # Remove backup files created by sed -i

echo "Committing and pushing changes..."
git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add "$FORMULA_PATH"
# Check if there are changes to commit
if git diff --staged --quiet; then
  echo "No changes to the formula file. Skipping commit."
else
  git commit -m "$COMMIT_MESSAGE"
  git push origin HEAD # Pushes to the default branch (usually main or master)
  echo "Changes pushed successfully."
fi
