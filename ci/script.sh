# This script takes care of testing your crate

set -eu

build() {
    VERSION=""
    [ -n "${TRAVIS_TAG#v}" ] && VERSION="${TRAVIS_TAG#v}"
    [ -n "$VERSION" ] && sed -i -e "s/version = \"0.0.0\"/version = \"$VERSION\"/" Cargo.toml
    cat Cargo.toml

    cross build --target $TARGET
    cross build --target $TARGET --release
}

runtest () {
    if [ ! -z ${DISABLE_TESTS+x} ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release
}

build

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    runtest
fi

#echo "!!!rekby-debug"
#ls -la target/release
#
#echo "!!! 2"
#ls -la target/release/*
#
#echo "!!! find"
#echo "$PWD"
#find / -iname "*whois*monitoring*" 2>/dev/null || true

