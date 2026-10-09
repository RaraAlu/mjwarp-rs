#!/usr/bin/env bash
set -euo pipefail
release=0
all_features=0
full_regression=0
require_t4=0
for option in "$@"; do
    case "$option" in
        --release) release=1 ;;
        --all-features) all_features=1 ;;
        --full-regression) full_regression=1 ;;
        --require-t4) require_t4=1 ;;
        --help)
            echo 'Usage: bash scripts/test-linux-g01.sh [--release] [--all-features] [--full-regression] [--require-t4]'
            exit 0 ;;
        *) echo "Unknown option: $option" >&2; exit 2 ;;
    esac
done
source "$(dirname -- "${BASH_SOURCE[0]}")/linux-probe-common.sh"
source "$(dirname -- "${BASH_SOURCE[0]}")/linux-resident-checks.sh"
linux_probe_setup
trap 'echo "LINUX_G01_EVIDENCE=$directory" >&2' EXIT
gpu=$(nvidia-smi --id=0 --query-gpu=name,driver_version,memory.total,compute_cap --format=csv,noheader)
if (( require_t4 )) && [[ $gpu != Tesla\ T4,* && $gpu != NVIDIA\ T4,* ]]; then
    echo "T4 required; device 0 is $gpu" >&2
    exit 1
fi
verify_fixtures
build_native_mocks
base=(cargo test --locked "${feature[@]}" "${profile[@]}")
run_checked fmt -1 0 cargo fmt --check
run_checked native 27 27 "${base[@]}" --test native_model_probe -- --ignored --test-threads=1
# Select the exact Cargo artifact, not a stale glob from another feature build.
cargo test --locked "${feature[@]}" "${profile[@]}" --test native_model_probe \
    --no-run --message-format=json > "$directory/native-artifact.jsonl"
artifact=$(jq -sr '[.[] | select(.reason == "compiler-artifact" and
    .target.name == "native_model_probe" and .executable != null) | .executable] |
    if length == 1 then .[0] else error("missing or ambiguous native artifact") end' "$directory/native-artifact.jsonl")
run_checked native-restricted 27 27 env -u LD_LIBRARY_PATH -u LD_PRELOAD PATH=/usr/bin:/bin \
    "$artifact" --ignored --test-threads=1
run_checked host -1 1 "${base[@]}"
run_checked g01 7 7 "${base[@]}" --test g01 -- --include-ignored --test-threads=1 --nocapture
run_checked native-g01 2 2 "${base[@]}" --test native_g01 -- --include-ignored --test-threads=1
run_checked native-only-g01 1 1 cargo test --locked --features native-model-probe "${profile[@]}" \
    --test native_g01 -- --include-ignored --test-threads=1
run_checked native-only 22 22 cargo test --locked --features native-model-probe "${profile[@]}" \
    --test native_model_probe -- --ignored --test-threads=1
run_resident_checks
if (( full_regression )); then
    expected=168
    if (( all_features )); then expected=176; fi
    run_checked all-probes "$expected" "$expected" "${base[@]}" -- --ignored --test-threads=1 --nocapture
    run_checked host-default -1 203 cargo test --locked
    run_checked clippy-default -1 0 cargo clippy --locked --all-targets -- -D warnings
    run_checked clippy-cuda -1 0 cargo clippy --locked --all-targets --features cuda-probe -- -D warnings
    run_checked clippy-native -1 0 cargo clippy --locked --all-targets --features native-model-probe -- -D warnings
fi
run_checked clippy -1 0 cargo clippy --locked --all-targets "${feature[@]}" "${profile[@]}" -- -D warnings
run_checked rustdoc -1 0 env RUSTDOCFLAGS='-D warnings' cargo doc --locked "${feature[@]}" --no-deps
run_checked diff -1 0 git diff --check
stats=$(jq -Rsc 'capture("G01-complete states=(?<states>[0-9]+) scalars=(?<scalars>[0-9]+) max_abs_error=(?<error>[0-9.eE+-]+)") |
    map_values(tonumber) | select(.states == 2084 and .scalars == 7704548)' "$directory/g01.log")
raw=$(jq -Rsc 'capture("G01 raw scalars=(?<scalars>[0-9]+) max_abs_error=(?<error>[0-9.eE+-]+)") |
    map_values(tonumber) | select(.scalars == 50512)' "$directory/g01.log")
[[ -n $stats && -n $raw ]] || { echo 'Missing G01 scalar counts' >&2; exit 1; }
revision=$(git rev-parse HEAD)
dirty=false
if [[ -n $(git status --porcelain) ]]; then dirty=true; fi
jq -n --arg platform 'Linux x86_64 GNU' --arg gpu "$gpu" --arg rustc "$(rustc --version)" \
    --arg kernel "$(uname -r)" --arg revision "$revision" --arg recordedAtUtc "$(date -u +%FT%TZ)" \
    --argjson dirty "$dirty" --argjson release "$release" --argjson fullRegression "$full_regression" \
    --argjson stats "$stats" --argjson raw "$raw" --arg features "${feature[*]}" \
    --arg librarySha256 "$(sha256sum "$MJWARP_MUJOCO_DLL" | cut -d ' ' -f 1)" \
    --arg manifestSha256 "$(sha256sum fixtures/g01/manifest.json | cut -d ' ' -f 1)" \
    --slurpfile checks "$directory/checks.json" --slurpfile native fixtures/native-probe/linux-runtime.json \
    --slurpfile fixture fixtures/g01/manifest.json --slurpfile cuda "$CUDA_PATH/components.json" \
    '{platform:$platform,gpu:$gpu,kernel:$kernel,rustc:$rustc,gitRevision:$revision,workingTreeDirty:$dirty,
      recordedAtUtc:$recordedAtUtc,release:($release==1),fullRegression:($fullRegression==1),features:$features,
      upstreamRevision:$fixture[0].upstream_revision,completeStates:$stats.states,completeScalars:$stats.scalars,
      maxAbsoluteError:$stats.error,rawScalars:$raw.scalars,rawMaxAbsoluteError:$raw.error,
      librarySha256:$librarySha256,fixtureManifestSha256:$manifestSha256,nativeRuntime:$native[0],
      cudaComponents:$cuda[0],checks:$checks[0],linuxVerified:true,
      t4Verified:($gpu | test("^(Tesla T4|NVIDIA T4),")),
      scriptInvokesPython:false,scriptInvokesModelCompiler:false,scriptInvokesNativePhysics:false}' \
    > "$directory/report.json"
