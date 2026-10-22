#!/usr/bin/env bash

server_mask_resolve_context() {
  local repository_root="$1"
  local canonical_root git_root canonical_git_root status
  canonical_root="$(cd "$repository_root" && pwd -P)"
  server_mask_source=''
  if git_root="$(git -C "$canonical_root" rev-parse --show-toplevel 2>/dev/null)"; then
    canonical_git_root="$(cd "$git_root" && pwd -P)"
    if [ "$canonical_git_root" = "$canonical_root" ]; then
      status="$(git -C "$canonical_root" status --porcelain --untracked-files=normal)"
      if [ -z "$status" ]; then
        server_mask_source="$(git -C "$canonical_root" rev-parse HEAD)"
      fi
    fi
  fi

  local index_set="${GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX+x}"
  local count_set="${GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT+x}"
  local report_set="${GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR+x}"
  local plan_set="${GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN_FILE+x}"
  local ci_set="${GRENADIERS_SERVER_REPLAY_ITEM_MASK_CI_SOURCE+x}"
  if [ -n "$index_set$count_set$report_set$plan_set$ci_set" ]; then
    if [ -z "$index_set" ] || [ -z "$count_set" ] || [ -z "$report_set" ] \
      || [ -z "$plan_set" ] || [ -z "$ci_set" ] \
      || [ -z "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_INDEX" ] \
      || [ -z "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PARTITION_COUNT" ] \
      || [ -z "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_REPORT_DIR" ] \
      || [ -z "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN_FILE" ] \
      || [ -z "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_CI_SOURCE" ]; then
      echo 'server item-mask partition selectors require complete nonempty CI report context' >&2
      return 1
    fi
    if [ ! -f "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_PLAN_FILE" ]; then
      echo 'server item-mask CI plan file is missing' >&2
      return 1
    fi
    if [ -z "$server_mask_source" ] \
      || [ "$server_mask_source" != "$GRENADIERS_SERVER_REPLAY_ITEM_MASK_CI_SOURCE" ]; then
      echo 'clean canonical checked-out HEAD does not match the CI server-mask source' >&2
      return 1
    fi
  fi
}
