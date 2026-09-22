#!/bin/sh

set -eu

root_dir="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
test_dir="$(mktemp -d "${TMPDIR:-/tmp}/storyscript-install-test.XXXXXX")"
trap 'rm -rf "$test_dir"' 0 1 2 3 15

fixture_dir="$test_dir/fixture/storyscript-v9.8.7-linux-x86_64"
mock_bin="$test_dir/mock-bin"
mkdir -p "$fixture_dir" "$mock_bin"

for binary in storyscript-parser storyscript-player storyscript-bundle; do
  printf '#!/bin/sh\nprintf "fixture %s\\n"\n' "$binary" > "$fixture_dir/$binary"
  chmod 0755 "$fixture_dir/$binary"
done

archive="$test_dir/storyscript-v9.8.7-linux-x86_64.tar.gz"
tar -C "$test_dir/fixture" -czf "$archive" storyscript-v9.8.7-linux-x86_64
if command -v sha256sum >/dev/null 2>&1; then
  archive_checksum="$(sha256sum "$archive" | awk '{ print $1 }')"
else
  archive_checksum="$(shasum -a 256 "$archive" | awk '{ print $1 }')"
fi
checksum="$test_dir/storyscript-v9.8.7-linux-x86_64.tar.gz.sha256"
printf '%s  %s\n' "$archive_checksum" "storyscript-v9.8.7-linux-x86_64.tar.gz" > "$checksum"

cat > "$mock_bin/uname" <<'EOF'
#!/bin/sh
case "${1:-}" in
  -s) printf '%s\n' "${MOCK_UNAME_S:-Linux}" ;;
  -m) printf '%s\n' "${MOCK_UNAME_M:-x86_64}" ;;
  *) exit 2 ;;
esac
EOF

cat > "$mock_bin/curl" <<'EOF'
#!/bin/sh
output=''
url=''
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o)
      output="$2"
      shift 2
      ;;
    -H | --proto)
      shift 2
      ;;
    --tlsv1.2 | -fsSL)
      shift
      ;;
    *)
      url="$1"
      shift
      ;;
  esac
done

case "$url" in
  */releases?per_page=1)
    printf '%s\n' '[{"tag_name":"v9.8.7","prerelease":true}]'
    ;;
  *.tar.gz.sha256)
    cp "$TEST_CHECKSUM" "$output"
    ;;
  *.tar.gz)
    cp "$TEST_ARCHIVE" "$output"
    ;;
  *)
    printf 'unexpected URL: %s\n' "$url" >&2
    exit 2
    ;;
esac
EOF
chmod 0755 "$mock_bin/uname" "$mock_bin/curl"

export TEST_ARCHIVE="$archive"
export TEST_CHECKSUM="$checksum"
export PATH="$mock_bin:$PATH"

install_dir="$test_dir/install/bin"
output="$(STORYSCRIPT_INSTALL_DIR="$install_dir" sh "$root_dir/install.sh")"
for binary in storyscript-parser storyscript-player storyscript-bundle; do
  [ -x "$install_dir/$binary" ] || {
    printf 'FAIL: %s was not installed\n' "$binary" >&2
    exit 1
  }
done
case "$output" in
  *'Installed StoryScript v9.8.7 commands'*) ;;
  *)
    printf 'FAIL: installer did not report successful latest-version installation\n' >&2
    exit 1
    ;;
esac
printf 'ok - installs latest release binaries\n'

bad_checksum="$test_dir/bad.sha256"
printf '%064d  %s\n' 0 "storyscript-v9.8.7-linux-x86_64.tar.gz" > "$bad_checksum"
export TEST_CHECKSUM="$bad_checksum"
if STORYSCRIPT_VERSION=9.8.7 STORYSCRIPT_INSTALL_DIR="$test_dir/bad-install" \
  sh "$root_dir/install.sh" >"$test_dir/bad-checksum.log" 2>&1; then
  printf 'FAIL: installer accepted an invalid checksum\n' >&2
  exit 1
fi
if ! awk '/checksum verification failed/ { found = 1 } END { exit !found }' "$test_dir/bad-checksum.log"; then
  printf 'FAIL: checksum failure did not produce a useful error\n' >&2
  exit 1
fi
printf 'ok - rejects checksum mismatch\n'

export TEST_CHECKSUM="$checksum"
if MOCK_UNAME_S=Plan9 STORYSCRIPT_VERSION=v9.8.7 STORYSCRIPT_INSTALL_DIR="$test_dir/unsupported" \
  sh "$root_dir/install.sh" >"$test_dir/unsupported.log" 2>&1; then
  printf 'FAIL: installer accepted an unsupported operating system\n' >&2
  exit 1
fi
if ! awk '/unsupported operating system/ { found = 1 } END { exit !found }' "$test_dir/unsupported.log"; then
  printf 'FAIL: unsupported OS did not produce a useful error\n' >&2
  exit 1
fi
printf 'ok - rejects unsupported operating system\n'

if STORYSCRIPT_VERSION='../unsafe' STORYSCRIPT_INSTALL_DIR="$test_dir/invalid-version" \
  sh "$root_dir/install.sh" >"$test_dir/invalid-version.log" 2>&1; then
  printf 'FAIL: installer accepted an invalid version\n' >&2
  exit 1
fi
if ! awk '/invalid release version/ { found = 1 } END { exit !found }' "$test_dir/invalid-version.log"; then
  printf 'FAIL: invalid version did not produce a useful error\n' >&2
  exit 1
fi
printf 'ok - rejects invalid release version\n'
