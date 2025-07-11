#!/bin/bash

# Script to deploy updates.json to GitHub Pages

# Check if updates.json exists
if [ ! -f "updates.json" ]; then
    echo "Error: updates.json not found in project root"
    echo "Create updates.json from updates.sample.json first"
    exit 1
fi

# Store current branch
CURRENT_BRANCH=$(git branch --show-current)

# Ensure we have the latest gh-pages branch
git fetch origin gh-pages:gh-pages

# Create gh-pages branch if it doesn't exist
if ! git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "Creating gh-pages branch..."
    git checkout --orphan gh-pages
    git rm -rf .
    echo "# KeyMagic Updates" > README.md
    git add README.md
    git commit -m "Initial gh-pages commit"
    git push origin gh-pages
fi

# Stash any changes
git stash

# Switch to gh-pages branch
git checkout gh-pages

# Copy updates.json from stashed files
git checkout stash@{0} -- updates.json

# Commit and push
git add updates.json
git commit -m "Update version info $(date +%Y-%m-%d)"
git push origin gh-pages

# Switch back to original branch
git checkout $CURRENT_BRANCH

# Pop stash if there were changes
git stash pop || true

echo "Successfully deployed updates.json to GitHub Pages"
echo "It will be available at: https://thantthet.github.io/keymagic-3/updates.json"
echo "Note: It may take a few minutes for GitHub Pages to update"