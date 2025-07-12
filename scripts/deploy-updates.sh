#!/bin/bash

# Script to deploy updates.json to GitHub Pages

# Check if updates.json exists
if [ ! -f "updates.json" ]; then
    echo "Error: updates.json not found in project root"
    echo "Create updates.json from updates.sample.json first"
    exit 1
fi

# Set worktree directory
WORKTREE_DIR=".gh-pages-worktree"

# Ensure we have the latest gh-pages branch
git fetch origin gh-pages:gh-pages

# Create gh-pages branch if it doesn't exist
if ! git show-ref --verify --quiet refs/heads/gh-pages; then
    echo "Creating gh-pages branch..."
    git checkout --orphan gh-pages
    echo "# KeyMagic Updates" > README.md
    git add README.md
    git commit -m "Initial gh-pages commit"
    git push origin gh-pages
    git checkout -
fi

# Remove existing worktree if it exists
if [ -d "$WORKTREE_DIR" ]; then
    git worktree remove --force "$WORKTREE_DIR"
fi

# Create worktree for gh-pages branch
echo "Creating worktree for gh-pages branch..."
git worktree add "$WORKTREE_DIR" gh-pages

# Copy updates.json to worktree
cp updates.json "$WORKTREE_DIR/"

# Commit and push changes
cd "$WORKTREE_DIR"
git add updates.json
git commit -m "Update version info $(date +%Y-%m-%d)"
git push origin gh-pages
cd ..

# Remove worktree
git worktree remove --force "$WORKTREE_DIR"

echo "Successfully deployed updates.json to GitHub Pages"
echo "It will be available at: https://thantthet.github.io/keymagic-3/updates.json"
echo "Note: It may take a few minutes for GitHub Pages to update"