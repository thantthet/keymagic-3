# How to Deploy updates.json to GitHub Pages

## Prerequisites
- GitHub Pages must be enabled for your repository
- You need push access to the repository

## Option 1: Using the Deploy Script (Recommended)

```bash
# 1. Create updates.json from the sample
cp updates.sample.json updates.json

# 2. Edit updates.json with current version info
# Update version numbers, URLs, release notes, etc.

# 3. Run the deploy script
./scripts/deploy-updates.sh
```

## Option 2: Manual Git Commands

```bash
# 1. Create updates.json from the sample
cp updates.sample.json updates.json

# 2. Edit updates.json with current version info

# 3. Create or switch to gh-pages branch
git checkout gh-pages || git checkout -b gh-pages

# 4. If creating new gh-pages branch, clean it
git rm -rf .
echo "# KeyMagic Updates" > README.md

# 5. Copy updates.json from main branch
git checkout main -- updates.json

# 6. Commit and push
git add updates.json README.md
git commit -m "Update version info"
git push origin gh-pages

# 7. Switch back to main branch
git checkout main
```

## Option 3: GitHub Actions (Automated)

The repository includes a GitHub Action that can automatically deploy updates.json:

1. **Manual trigger**: Go to Actions tab → "Deploy Updates JSON to GitHub Pages" → Run workflow
2. **Automatic on release**: Creates/updates updates.json when you publish a new release

## Setting up GitHub Pages

If GitHub Pages is not yet enabled:

1. Go to Settings → Pages in your GitHub repository
2. Under "Source", select "Deploy from a branch"
3. Select "gh-pages" branch and "/ (root)" folder
4. Click Save

## Verifying Deployment

After deployment, your updates.json will be available at:
```
https://thantthet.github.io/keymagic-3/updates.json
```

Note: It may take 5-10 minutes for changes to appear on GitHub Pages.

## Best Practices

1. **Test locally first**: Validate your JSON before deploying
   ```bash
   python -m json.tool updates.json
   ```

2. **Version consistency**: Ensure version numbers match your actual releases

3. **Keep history**: The gh-pages branch will maintain a history of all updates

4. **Automate**: Use GitHub Actions for consistent deployments

## Troubleshooting

- **404 Error**: Ensure GitHub Pages is enabled and pointing to gh-pages branch
- **Old content**: GitHub Pages can cache for up to 10 minutes
- **JSON errors**: Validate JSON syntax before deploying