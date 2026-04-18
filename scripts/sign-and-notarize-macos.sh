#!/bin/sh

set -eu

usage() {
    cat <<'EOF'
Usage:
  sh ./scripts/sign-and-notarize-macos.sh sign
  sh ./scripts/sign-and-notarize-macos.sh notarize
  sh ./scripts/sign-and-notarize-macos.sh all

Environment variables:
  CODESIGN_IDENTITY   Required. Example:
                      Developer ID Application: Noriaki Fukuyori (Q6GG27UYG5)
  NOTARY_PROFILE      Required for notarize/all. Keychain profile name created by:
                      xcrun notarytool store-credentials "<profile>"
  BIN_PATH            Optional. Defaults to target/release/moon
  DIST_DIR            Optional. Defaults to dist/macos
  ZIP_PATH            Optional. Defaults to <DIST_DIR>/moon-<version>-macos-signed.zip

Notes:
  - This script signs the standalone CLI binary and uploads a ZIP archive for notarization.
  - Apple can't staple a notarization ticket to a standalone Mach-O binary or ZIP archive.
    Gatekeeper can still validate notarization online for the signed binary inside the ZIP.
EOF
}

if [ "${1:-}" = "" ]; then
    usage
    exit 1
fi

COMMAND="$1"
BIN_PATH="${BIN_PATH:-target/release/moon}"
DIST_DIR="${DIST_DIR:-dist/macos}"
VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)"

if [ -z "$VERSION" ]; then
    echo "failed to read version from Cargo.toml" >&2
    exit 1
fi

ZIP_PATH="${ZIP_PATH:-$DIST_DIR/moon-$VERSION-macos-signed.zip}"
SIGNED_BIN="$DIST_DIR/moon"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "required command not found: $1" >&2
        exit 1
    fi
}

build_release() {
    require_command cargo
    cargo build --release
}

prepare_dist() {
    mkdir -p "$DIST_DIR"
    cp "$BIN_PATH" "$SIGNED_BIN"
}

sign_binary() {
    if [ -z "${CODESIGN_IDENTITY:-}" ]; then
        echo "CODESIGN_IDENTITY is required" >&2
        exit 1
    fi

    require_command codesign
    codesign --force \
        --sign "$CODESIGN_IDENTITY" \
        --options runtime \
        --timestamp \
        "$SIGNED_BIN"
    codesign --verify --strict --verbose=2 "$SIGNED_BIN"
    codesign -dv "$SIGNED_BIN"
}

package_zip() {
    require_command ditto
    rm -f "$ZIP_PATH"
    ditto -c -k --keepParent "$SIGNED_BIN" "$ZIP_PATH"
}

notarize_zip() {
    if [ -z "${NOTARY_PROFILE:-}" ]; then
        echo "NOTARY_PROFILE is required for notarization" >&2
        exit 1
    fi

    require_command xcrun
    xcrun notarytool submit "$ZIP_PATH" \
        --keychain-profile "$NOTARY_PROFILE" \
        --wait
}

case "$COMMAND" in
    sign)
        build_release
        prepare_dist
        sign_binary
        package_zip
        echo "signed binary: $SIGNED_BIN"
        echo "archive: $ZIP_PATH"
        ;;
    notarize)
        if [ ! -f "$ZIP_PATH" ]; then
            echo "archive not found: $ZIP_PATH" >&2
            echo "run the sign step first" >&2
            exit 1
        fi
        notarize_zip
        echo "notarization submitted successfully for: $ZIP_PATH"
        ;;
    all)
        build_release
        prepare_dist
        sign_binary
        package_zip
        notarize_zip
        echo "signed binary: $SIGNED_BIN"
        echo "archive notarized: $ZIP_PATH"
        ;;
    -h|--help|help)
        usage
        ;;
    *)
        echo "unknown command: $COMMAND" >&2
        usage
        exit 1
        ;;
esac
