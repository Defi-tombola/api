#!/bin/bash

set -e

export PATH="$HOME/.cargo/bin:$PATH"

git checkout stage
git fetch --all --tags
OLD_RELEASE_VERSION=$(cargo ws list --json | jq -r '.[] | select(.name == "graphql") | .version')
RELEASE_VERSION=$(echo $OLD_RELEASE_VERSION | awk -F. '{print $1"."$2+1".0"}')

read -p "Current version: $OLD_RELEASE_VERSION. Are you sure you want to create a release? This will reset stage branch to origin (y/n) " CONT
if [ "$CONT" != "y" ]; then
  echo "Aborted"
  exit 1
fi

git reset origin/stage # Reset to origin/stage
PR_BODY=$(git log --pretty=format:"%h %s" master..stage)

echo -e "## Release v$RELEASE_VERSION \n $PR_BODY\n\n$(cat CHANGELOG.md)" > CHANGELOG.md

# Add commit and push
git add CHANGELOG.md
git commit -m "[skip build] Update changelog for v$RELEASE_VERSION"
git push origin stage

# Create pull request
gh pr create -B master -b "### Release v$RELEASE_VERSION \n $PR_BODY" -t "Update prod v$RELEASE_VERSION"
gh pr view --web
