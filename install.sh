#!/bin/sh

set -eu

REPOSITORY="${STORYSCRIPT_REPOSITORY:-anggaaryas/storyscript}"
REQUESTED_VERSION="${STORYSCRIPT_VERSION:-latest}"

say() {
  printf '%s\n' "$*"
}

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "required command not found: $1"
}

require_command curl
require_command uname
require_command mktemp
require_command tar
require_command awk
require_command install

case "$(uname -s)" in
  Linux) platform_os="linux" ;;
  Darwin) platform_os="macos" ;;
  *) fail "unsupported operating system: $(uname -s)" ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) platform_arch="x86_64" ;;
  arm64 | aarch64) platform_arch="aarch64" ;;
  *) fail "unsupported CPU architecture: $(uname -m)" ;;
esac

if [ "$REQUESTED_VERSION" = "latest" ]; then
  release_json="$(
    curl --proto '=https' --tlsv1.2 -fsSL \
      -H 'Accept: application/vnd.github+json' \
      -H 'X-GitHub-Api-Version: 2022-11-28' \
      "https://api.github.com/repos/${REPOSITORY}/releases?per_page=1"
  )"
  version="$(printf '%s\n' "$release_json" | awk -F '"' '/"tag_name"[[:space:]]*:/ { print $4; exit }')"
  [ -n "$version" ] || fail "could not determine the latest release from GitHub"
else
  version="$REQUESTED_VERSION"
  case "$version" in
    v*) ;;
    *) version="v${version}" ;;
  esac
fi

if ! printf '%s\n' "$version" | awk \
  '/^v[0-9]+\.[0-9]+\.[0-9]+([.-][0-9A-Za-z.-]+)?$/ { valid = 1 } END { exit !valid }'; then
  fail "invalid release version: $version"
fi

if [ -n "${STORYSCRIPT_INSTALL_DIR:-}" ]; then
  install_dir="$STORYSCRIPT_INSTALL_DIR"
else
  [ -n "${HOME:-}" ] || fail "HOME is not set; provide STORYSCRIPT_INSTALL_DIR"
  install_dir="$HOME/.local/bin"
fi

asset="storyscript-${version}-${platform_os}-${platform_arch}.tar.gz"
download_url="https://github.com/${REPOSITORY}/releases/download/${version}/${asset}"
temp_dir="$(mktemp -d "${TMPDIR:-/tmp}/storyscript-install.XXXXXX")"
trap 'rm -rf "$temp_dir"' 0 1 2 3 15

say "Downloading StoryScript ${version} for ${platform_os}-${platform_arch}..."
curl --proto '=https' --tlsv1.2 -fsSL \
  -o "$temp_dir/$asset" \
  "$download_url"
curl --proto '=https' --tlsv1.2 -fsSL \
  -o "$temp_dir/$asset.sha256" \
  "$download_url.sha256"

expected_checksum="$(awk 'NR == 1 { print $1 }' "$temp_dir/$asset.sha256")"
[ "${#expected_checksum}" -eq 64 ] || fail "release checksum is invalid"
case "$expected_checksum" in
  *[!0-9A-Fa-f]* | '') fail "release checksum is invalid" ;;
esac

if command -v sha256sum >/dev/null 2>&1; then
  actual_checksum="$(sha256sum "$temp_dir/$asset" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
  actual_checksum="$(shasum -a 256 "$temp_dir/$asset" | awk '{ print $1 }')"
else
  fail "sha256sum or shasum is required to verify the download"
fi

[ "$expected_checksum" = "$actual_checksum" ] || fail "checksum verification failed for $asset"

tar -xzf "$temp_dir/$asset" -C "$temp_dir"
extracted_dir="$temp_dir/storyscript-${version}-${platform_os}-${platform_arch}"

for binary in storyscript-parser storyscript-player storyscript-bundle; do
  [ -f "$extracted_dir/$binary" ] || fail "release archive is missing $binary"
done

mkdir -p "$install_dir"
[ -d "$install_dir" ] || fail "install destination is not a directory: $install_dir"
[ -w "$install_dir" ] || fail "install destination is not writable: $install_dir"

for binary in storyscript-parser storyscript-player storyscript-bundle; do
  install -m 0755 "$extracted_dir/$binary" "$install_dir/$binary"
done

say "Installed StoryScript ${version} commands in $install_dir:"
say "  storyscript-parser"
say "  storyscript-player"
say "  storyscript-bundle"

case ":${PATH:-}:" in
  *":$install_dir:"*) ;;
  *)
    say ""
    say "Add $install_dir to PATH, for example:"
    say "  export PATH=\"$install_dir:\$PATH\""
    ;;
esac
