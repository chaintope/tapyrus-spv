#! /bin/bash

set -eux

rustup component add rustfmt-preview
cargo fmt -v -- -v --emit files

if git commit -a -m "bitbucket pipelines, rustfmt"; then
    git push
    echo "Pushed formatting changes. Another build will start."
    exit 1
fi
