# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    cp README.md LICENSE.txt config-default.yaml customers-example.yaml $stage/

    cd $stage
    case "$TARGET" in
      *-windows-*)
        cp "$src/target/$TARGET/release/$CRATE_NAME.exe" $stage/

        unix2dos -n README.md README.txt
        rm -f README.md
        unix2dos -n LICENSE.txt LICENSE.txt_
        mv -f LICENSE.txt_ LICENSE.txt
        unix2dos -n config-default.yaml config-default.yaml_
        mv -f config-default.yaml_ config-default.yaml
        unix2dos -n customers-example.yaml customers-example.yaml_
        mv -f customers-example.yaml_ customers-example.yaml
        zip "$src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.zip" *
        ;;
      *)
        cp "$src/target/$TARGET/release/$CRATE_NAME" $stage/
        tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
        ;;
    esac

    cd $src

    rm -rf $stage
}

main
