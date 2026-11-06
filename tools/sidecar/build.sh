#!/usr/bin/env bash
# Build the camera sidecar (native/camera-sidecar) and install it where Tauri's externalBin expects it.
#
#   tools/sidecar/build.sh --stub [--test]        stub backend (any Mac, CI); no Sony SDK involved
#   tools/sidecar/build.sh --sdk <root> [--test]  against an installed Sony Camera Remote SDK (build machine)
#
# Builds under target/camera-sidecar/{stub,sdk} (never inside the source tree), --test runs ctest
# verbosely, then copies the binary to apps/desktop/src-tauri/binaries/camera-sidecar-aarch64-apple-darwin
# (gitignored). Nothing here is committed and no Sony file is copied into the repository or the app.
set -euo pipefail

usage() {
  echo "usage: tools/sidecar/build.sh --stub | --sdk <sdk root> [--test]" >&2
  exit 2
}

repo="$(cd "$(dirname "$0")/../.." && pwd)"
mode=""
sdk_root=""
run_tests=0
while [ $# -gt 0 ]; do
  case "$1" in
    --stub) mode="stub" ;;
    --sdk)
      [ $# -ge 2 ] || usage
      mode="sdk"
      sdk_root="$2"
      shift
      ;;
    --test) run_tests=1 ;;
    *) usage ;;
  esac
  shift
done
[ -n "$mode" ] || usage

if ! command -v cmake >/dev/null 2>&1; then
  echo "camera sidecar: cmake is not installed. Install it (for example: brew install cmake) and build again." >&2
  exit 127
fi

src="$repo/native/camera-sidecar"
build="$repo/target/camera-sidecar/$mode"
configure=(-S "$src" -B "$build" -DCMAKE_BUILD_TYPE=Release)
if [ "$mode" = "sdk" ]; then
  [ -d "$sdk_root" ] || { echo "camera sidecar: SDK root not found: $sdk_root" >&2; exit 2; }
  configure+=(-DCAMERA_SIDECAR_WITH_SDK=ON "-DCRSDK_ROOT=$(cd "$sdk_root" && pwd)")
else
  configure+=(-DCAMERA_SIDECAR_WITH_SDK=OFF)
fi

cmake "${configure[@]}"
cmake --build "$build" --parallel

if [ "$run_tests" -eq 1 ]; then
  (cd "$build" && ctest --output-on-failure --verbose)
fi

dest_dir="$repo/apps/desktop/src-tauri/binaries"
mkdir -p "$dest_dir"
cp "$build/camera-sidecar" "$dest_dir/camera-sidecar-aarch64-apple-darwin"
chmod 755 "$dest_dir/camera-sidecar-aarch64-apple-darwin"
echo "camera sidecar ($mode): $dest_dir/camera-sidecar-aarch64-apple-darwin"
