#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

mode=runtime
if [[ "$#" -eq 1 && "${1:-}" == --cpu ]]; then
  mode=cpu
elif [[ "$#" -ne 0 ]]; then
  echo 'usage: scripts/measure-game.sh [--cpu]' >&2
  exit 2
fi

output="$root/target/m7-measurements"
if [[ "$mode" == cpu ]]; then
  output="$root/target/cpu-measurements"
fi
mkdir -p "$output"
: > "$output/results.jsonl"

node24="${NODE24_BIN:-$(command -v node || true)}"
if [[ ! -x "$node24" ]]; then
  echo "pinned Node 24 executable is unavailable: $node24" >&2
  exit 1
fi

node_version="$("$node24" --version)"
if [[ "$node_version" != v24.21.0 ]]; then
  echo "measurement helper requires Node v24.21.0; found $node_version" >&2
  exit 1
fi

cargo_messages="$output/cargo-artifacts.jsonl"
cargo test --release --locked -p grenadiers-server --lib --no-run \
  --message-format=json-render-diagnostics > "$cargo_messages"
test_binary="$("$node24" scripts/measure-game-artifact.mjs select < "$cargo_messages")"
if [[ ! -x "$test_binary" ]]; then
  echo "Cargo reported a non-executable libtest artifact: $test_binary" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  binary_sha256="$(sha256sum "$test_binary" | awk '{print $1}')"
else
  binary_sha256="$(shasum -a 256 "$test_binary" | awk '{print $1}')"
fi

git_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -n "$git_root" ]] && [[ "$(cd "$git_root" && pwd -P)" == "$(pwd -P)" ]]; then
  source_commit="$(git rev-parse --verify HEAD)"
  if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
    source_state=dirty
  else
    source_state=clean
  fi
else
  source_commit="${MEASUREMENT_SOURCE_COMMIT:-}"
  if [[ ! "$source_commit" =~ ^[0-9a-fA-F]{40}$ ]]; then
    echo 'a Git-free export requires MEASUREMENT_SOURCE_COMMIT with exactly 40 hexadecimal characters' >&2
    exit 1
  fi
  source_commit="$(printf '%s' "$source_commit" | tr '[:upper:]' '[:lower:]')"
  source_state='export'
fi
image_identity="${MEASUREMENT_IMAGE_IDENTITY:-none-native}"
if [[ -z "$image_identity" ]]; then
  echo 'MEASUREMENT_IMAGE_IDENTITY must not be empty' >&2
  exit 1
fi

{
  echo "node_version=$node_version"
  echo "source_commit=$source_commit"
  echo "source_state=$source_state"
  echo "binary_sha256=$binary_sha256"
  echo "image_identity=$image_identity"
  echo "test_binary=$test_binary"
  echo 'profile=release'
  rustc -Vv
  uname -a
  echo "process_visible_logical_cpus=$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo unknown)"
  if command -v sysctl >/dev/null 2>&1; then
    sysctl -n machdep.cpu.brand_string 2>/dev/null || true
    sysctl -n hw.logicalcpu 2>/dev/null || true
    sysctl -n hw.memsize 2>/dev/null || true
  fi
  if [[ "$(uname -s)" == Darwin ]] && command -v system_profiler >/dev/null 2>&1; then
    system_profiler SPHardwareDataType 2>/dev/null \
      | sed -n -e '/^[[:space:]]*Chip:/p' -e '/^[[:space:]]*Total Number of Cores:/p' -e '/^[[:space:]]*Memory:/p'
    echo 'cgroup=not-applicable'
    echo 'process_affinity=not-constrained-by-harness'
    echo 'memory_limit=not-constrained-by-harness'
  fi
  if [[ -r /proc/cpuinfo ]]; then
    grep -m1 'model name' /proc/cpuinfo || true
  fi
  if [[ -r /proc/self/status ]]; then
    grep -E 'Cpus_allowed_list|Mems_allowed_list' /proc/self/status || true
  fi
  for file in /sys/fs/cgroup/cpu.max /sys/fs/cgroup/memory.max /sys/fs/cgroup/cpuset.cpus.effective; do
    if [[ -r "$file" ]]; then
      echo "$file=$(cat "$file")"
    fi
  done
} > "$output/environment.txt"

if /usr/bin/time -v true >/dev/null 2>&1; then
  time_mode=gnu
else
  time_mode=bsd
fi

run_case() {
  local name="$1"
  local filter="$2"
  local log="$output/$name.log"
  local usage="$output/$name.time.txt"
  if [[ "$time_mode" == gnu ]]; then
    /usr/bin/time -v -o "$usage" \
      "$test_binary" "$filter" --ignored --nocapture --test-threads=1 | tee "$log"
  else
    /usr/bin/time -l -o "$usage" \
      "$test_binary" "$filter" --ignored --nocapture --test-threads=1 | tee "$log"
  fi
  local records
  records="$(grep -c 'M7_MEASUREMENT_JSON=' "$log")"
  if [[ "$records" != 1 ]]; then
    echo "$name produced $records measurement records; expected one" >&2
    exit 1
  fi
  local record
  record="$(sed -n 's/^.*M7_MEASUREMENT_JSON=//p' "$log")"
  MEASUREMENT_COMMIT="$source_commit" \
    MEASUREMENT_SOURCE_STATE="$source_state" \
    MEASUREMENT_BINARY_SHA256="$binary_sha256" \
    MEASUREMENT_IMAGE_IDENTITY="$image_identity" \
    "$node24" scripts/measure-game-artifact.mjs identify \
      <<< "$record" >> "$output/results.jsonl"
}

if [[ "$mode" == cpu ]]; then
  export GRENADIERS_MEASUREMENT_SOURCE="$source_commit:$source_state"
  GRENADIERS_MEASUREMENT_TOOLCHAIN="$(rustc -Vv)"
  export GRENADIERS_MEASUREMENT_TOOLCHAIN
  run_case cpu cpu_scripted_evaluation_measurement
else
  run_case runtime runtime_reference_measurement
  run_case owner-1 owner_one_room_30s_measurement
  run_case owner-8 owner_eight_rooms_30s_measurement
  run_case owner-queue-pressure owner_queue_pressure_measurement
fi

echo "Measurements passed; results: $output/results.jsonl"
