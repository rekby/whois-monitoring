#!/bin/bash

set -eu

VERSION=""
[ -n "${TRAVIS_TAG#v}" ] && VERSION="${TRAVIS_TAG#v}"

[ -n "$VERSION" ] && sed -i -e "s/version = \"0.0.0\"/version = \"$VERSION\"/" Cargo.toml

cat Cargo.toml

cargo build --release
