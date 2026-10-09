#!/usr/bin/env bash
set -euo pipefail

release=0
require_t4=0
for argument in "$@"; do
    case "$argument" in
        --release) release=1 ;;
        --require-t4) require_t4=1 ;;
        --help) echo 'Usage: bash scripts/test-linux-com-velocity.sh [--release] [--require-t4]'; exit 0 ;;
        *) echo "Unknown argument: $argument" >&2; exit 2 ;;
    esac
done

[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Requires Linux x86_64' >&2; exit 1; }
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
cd -- "$root"
source "$root/scripts/linux-probe-common.sh"
for command in cargo rustc jq sha256sum nvidia-smi; do
    command -v "$command" >/dev/null || { echo "Missing command: $command" >&2; exit 1; }
done
export CUDA_PATH="$root/target/toolchains/cuda-12.8.1"
[[ -f $CUDA_PATH/lib64/libnvrtc.so.12 ]] || { echo 'Run prepare-linux-cuda.sh first' >&2; exit 1; }
export LD_LIBRARY_PATH="$CUDA_PATH/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
gpu=$(nvidia-smi --query-gpu=name,driver_version,memory.total,compute_cap --format=csv,noheader | head -n 1)
if (( require_t4 )) && [[ ! ${gpu%%,*} =~ (^|[[:space:]])T4($|[[:space:]]) ]]; then
    echo "Requires actual T4 device zero; found: $gpu" >&2
    exit 1
fi

manifest="$root/fixtures/com-velocity/manifest.json"
rows=$(jq -er '.files | if type == "array" and length > 0 and
    all(.[]; (.path | type == "string" and startswith("fixtures/")) and
        (.sha256 | type == "string" and test("^[a-f0-9]{64}$")))
    then .[] else error("invalid fixture manifest") end |
    [.path,.sha256] | @tsv' "$manifest")
while IFS=$'\t' read -r path hash; do
    printf '%s  %s\n' "$hash" "$root/$path" | sha256sum --check --status
done <<< "$rows"
directory=$(mktemp -d "$root/target/linux-com-velocity-XXXXXXXX")
printf '[]\n' > "$directory/checks.json"
profile=()
if (( release )); then profile=(--release); fi
base=(cargo test --locked --features cuda-probe "${profile[@]}")
run_checked fmt -1 0 cargo fmt --check
run_checked reference 1 1 "${base[@]}" --test resident_com_velocity reference_hashes
run_checked host 2 2 "${base[@]}" --lib physics::com_velocity::tests \
    -- --skip velocity_readback_rejects_guards_and_nonfinite_buffers
run_checked gpu 6 6 "${base[@]}" --test resident_com_velocity -- --ignored --test-threads=1 --nocapture
run_checked guards 1 1 "${base[@]}" --lib velocity_readback_rejects_guards_and_nonfinite_buffers \
    -- --ignored --test-threads=1
run_checked default-host 2 2 cargo test --locked "${profile[@]}" --lib physics::com_velocity
run_checked clippy-default -1 0 cargo clippy --locked --all-targets -- -D warnings
run_checked clippy-cuda -1 0 cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
run_checked rustdoc -1 0 env RUSTDOCFLAGS='-D warnings' cargo doc --locked --features cuda-probe --no-deps
run_checked diff -1 0 git diff --check

maximum=$(sed -nE 's/.*max_abs_error=([0-9.e+-]+).*/\1/p' "$directory/gpu.log" |
    jq -Rse 'split("\n") | map(select(length > 0) | tonumber) | max // error("missing numeric evidence")')
manifest_hash=$(sha256sum "$manifest" | cut -d ' ' -f 1)
source_hash=$(sha256sum src/physics/com_velocity.rs | cut -d ' ' -f 1)
dirty=false
if [[ -n $(git status --porcelain) ]]; then dirty=true; fi
jq -n --arg date "$(date -u +%Y-%m-%dT%H:%M:%SZ)" --arg gpu "$gpu" \
    --arg revision "$(git rev-parse HEAD)" --arg rustc "$(rustc --version)" \
    --arg manifest "$manifest_hash" --arg source "$source_hash" \
    --argjson release "$release" --argjson dirty "$dirty" --argjson maximum "$maximum" \
    --slurpfile checks "$directory/checks.json" \
    '{recordedAtUtc:$date,platform:"Linux x86_64 GNU",gpu:$gpu,rustc:$rustc,
      gitRevision:$revision,workingTreeDirty:$dirty,release:($release == 1),
      upstreamRevision:"71da24d956378a87a703b6e1442b13aec0c4ac29",
      referenceManifestSha256:$manifest,sourceSha256:$source,
      nativeReferenceStates:40,crossBlockWorlds:513,maxAbsoluteError:$maximum,
      comVelocitySubsetVerified:true,g02Verified:false,t4Verified:false,
      productTestsInvokePython:false,productTestsInvokeNativePhysics:false,
      checks:$checks[0]}' > "$directory/report.json"
printf 'Report: %s\n' "$directory/report.json"
