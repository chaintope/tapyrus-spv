#! /bin/bash

set -eux

git config --global user.email "${GIT_USER_EMAIL}"
git config --global user.name "bitbucket pipelines"
git remote set-url origin https://${BITBUCKET_USERNAME}:${BITBUCKET_APP_PASSWORD}@bitbucket.org/${BITBUCKET_REPO_OWNER}/${BITBUCKET_REPO_SLUG}.git
