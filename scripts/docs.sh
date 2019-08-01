#! /bin/bash

set -eux

cargo doc --release --no-deps
git clone https://${BITBUCKET_USERNAME}:${BITBUCKET_APP_PASSWORD}@bitbucket.org/${BITBUCKET_REPO_OWNER}/${BITBUCKET_REPO_SLUG}.git target/docwebsite
rm -rvf target/docwebsite/*
cp -rv target/doc/* target/docwebsite/
cd target/docwebsite
git add -A
git commit -m "bitbucket pipelines, ${BITBUCKET_REPO_OWNER}/${BITBUCKET_REPO_SLUG} commit ${BITBUCKET_COMMIT}" || true
git push
