#!/usr/bin/env bash
# Write the camera support package manifest for one Sony SDK version: the runtime files and their
# SHA-256, hashes only (R-I36). Run on the build machine that holds the SDK; the output never holds
# a file's bytes.
#
#   tools/sidecar/manifest.sh <sdk root> <version> [<out file>]
#
# Lists every *.dylib under <sdk root>: a file whose parent directory is CrAdapter as
# "CrAdapter/<name>", any other as its basename; sorted bytewise by path.
# Default output: native/camera-sidecar/manifest/sony-<version>.json
# Tip: point <sdk root> at the folder holding libCr_Core.dylib and CrAdapter/ (for SDK 2.02:
# CrSDK_v2.02.00_Mac/RemoteCli/external/crsdk), so sample-app libraries are not listed.
set -euo pipefail

if [ $# -lt 2 ] || [ $# -gt 3 ]; then
  echo "usage: tools/sidecar/manifest.sh <sdk root> <version> [<out file>]" >&2
  exit 2
fi
repo="$(cd "$(dirname "$0")/../.." && pwd)"
root="$1"
version="$2"
out="${3:-$repo/native/camera-sidecar/manifest/sony-$version.json}"
[ -d "$root" ] || { echo "manifest: not a directory: $root" >&2; exit 2; }

json_string() {
  local s="${1//\\/\\\\}"
  s="${s//\"/\\\"}"
  printf '"%s"' "$s"
}

entries="$(
  find "$root" -name '*.dylib' -type f -print0 |
    while IFS= read -r -d '' file; do
      name="$(basename "$file")"
      parent="$(basename "$(dirname "$file")")"
      if [ "$parent" = "CrAdapter" ]; then path="CrAdapter/$name"; else path="$name"; fi
      sha="$(shasum -a 256 "$file" | awk '{print tolower($1)}')"
      printf '%s\t%s\n' "$path" "$sha"
    done | LC_ALL=C sort -t "$(printf '\t')" -k1,1
)"

mkdir -p "$(dirname "$out")"
{
  printf '{"version": %s, "files": [' "$(json_string "$version")"
  first=1
  while IFS="$(printf '\t')" read -r path sha; do
    [ -n "$path" ] || continue
    if [ $first -eq 0 ]; then printf ','; fi
    first=0
    printf '\n  {"path": %s, "sha256": %s}' "$(json_string "$path")" "$(json_string "$sha")"
  done <<<"$entries"
  printf '\n]}\n'
} >"$out"
echo "manifest: $out"
