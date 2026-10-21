#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 prepare|load DIRECTORY SOURCE_SHA IMAGE_TAG [EXPECTED_IMAGE_ID]" >&2
  exit 2
}

require_source_sha() {
  if [[ ! "$1" =~ ^[0-9a-f]{40}$ ]]; then
    echo "invalid source SHA: $1" >&2
    exit 1
  fi
}

require_image_id() {
  if [[ ! "$1" =~ ^sha256:[0-9a-f]{64}$ ]]; then
    echo "invalid image ID: $1" >&2
    exit 1
  fi
}

prepare() {
  local directory="$1" source_sha="$2" image_tag="$3"
  local archive="$directory/grenadiers-app.tar"
  local archive_sha image_id

  require_source_sha "$source_sha"
  mkdir -p "$directory"
  image_id="$(docker image inspect --format '{{.Id}}' "$image_tag")"
  require_image_id "$image_id"
  docker save --output "$archive" "$image_tag"
  (
    cd "$directory"
    sha256sum grenadiers-app.tar > grenadiers-app.tar.sha256
  )
  read -r archive_sha _ < "$directory/grenadiers-app.tar.sha256"
  printf 'source_sha=%s\nimage_tag=%s\nimage_id=%s\n' \
    "$source_sha" "$image_tag" "$image_id" > "$directory/image-metadata.txt"
  printf 'BROWSER_IMAGE source=%s image=%s archive=%s\n' \
    "$source_sha" "$image_id" "$archive_sha"
}

metadata_value() {
  local metadata="$1" key="$2"
  sed -n "s/^${key}=//p" "$metadata"
}

load() {
  local directory="$1" source_sha="$2" image_tag="$3" expected_image_id="$4"
  local metadata="$directory/image-metadata.txt"
  local archive_sha recorded_source recorded_tag recorded_image_id loaded_image_id

  require_source_sha "$source_sha"
  require_image_id "$expected_image_id"
  test "$(wc -l < "$metadata")" -eq 3
  recorded_source="$(metadata_value "$metadata" source_sha)"
  recorded_tag="$(metadata_value "$metadata" image_tag)"
  recorded_image_id="$(metadata_value "$metadata" image_id)"
  test "$recorded_source" = "$source_sha"
  test "$recorded_tag" = "$image_tag"
  test "$recorded_image_id" = "$expected_image_id"
  test "$(wc -l < "$directory/grenadiers-app.tar.sha256")" -eq 1
  grep -Eq '^[0-9a-f]{64}  grenadiers-app\.tar$' \
    "$directory/grenadiers-app.tar.sha256"
  (
    cd "$directory"
    sha256sum --check --strict grenadiers-app.tar.sha256
  )
  read -r archive_sha _ < "$directory/grenadiers-app.tar.sha256"
  printf 'BROWSER_IMAGE source=%s image=%s archive=%s\n' \
    "$source_sha" "$expected_image_id" "$archive_sha"
  docker load --input "$directory/grenadiers-app.tar"
  loaded_image_id="$(docker image inspect --format '{{.Id}}' "$image_tag")"
  test "$loaded_image_id" = "$expected_image_id"
}

if [ "$#" -lt 4 ]; then
  usage
fi

case "$1" in
  prepare)
    [ "$#" -eq 4 ] || usage
    prepare "$2" "$3" "$4"
    ;;
  load)
    [ "$#" -eq 5 ] || usage
    load "$2" "$3" "$4" "$5"
    ;;
  *) usage ;;
esac
