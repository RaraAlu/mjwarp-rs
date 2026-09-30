#!/usr/bin/env bash
# Shared by the explicit Linux acceptance scripts, never by product builds.
linux_probe_setup() {
    [[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Requires Linux x86_64' >&2; return 1; }
    root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
    cd -- "$root"
    for command in cargo rustc jq g++ sha256sum; do
        command -v "$command" >/dev/null || { echo "Missing command: $command" >&2; return 1; }
    done
    export MJWARP_MUJOCO_ROOT="$root/target/toolchains/mujoco-3.12.0/linux-package"
    # Keep the existing test environment name on both platforms.
    export MJWARP_MUJOCO_DLL="$MJWARP_MUJOCO_ROOT/lib/libmujoco.so.3.12.0"
    expected=$(jq -er '.library_sha256' "$root/fixtures/native-probe/linux-runtime.json")
    printf '%s  %s\n' "$expected" "$MJWARP_MUJOCO_DLL" | sha256sum --check --status
    export CUDA_PATH="$root/target/toolchains/cuda-12.8.1"
    [[ -f $CUDA_PATH/lib64/libnvrtc.so.12 ]] || { echo 'Run prepare-linux-cuda.sh first' >&2; return 1; }
    export LD_LIBRARY_PATH="$CUDA_PATH/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export PATH="$CUDA_PATH/bin:$PATH"
    feature=(--features native-model-probe,cuda-probe)
    if (( all_features )); then feature=(--all-features); fi
    profile=()
    if (( release )); then profile=(--release); fi
    directory=$(mktemp -d "$root/target/linux-g01-XXXXXXXX")
    printf '[]\n' > "$directory/checks.json"
}

verify_fixtures() {
    local manifest base path hash actual rows manifests
    manifests=$(find "$root/fixtures" -name manifest.json -type f | sort)
    [[ -n $manifests ]] || return 1
    while IFS= read -r manifest; do
        base=${manifest%/*}
        rows=$(jq -er '.files | if type == "array" and length > 0 and
            all(.[]; (.path | type == "string" and length > 0) and
                (.sha256 | type == "string" and test("^[a-f0-9]{64}$")))
            then .[] else error("invalid fixture manifest") end |
            [.path,.sha256] | @tsv' "$manifest")
        [[ -n $rows ]] || return 1
        while IFS=$'\t' read -r path hash; do
            if [[ $path == fixtures/* ]]; then actual="$root/$path"; else actual="$base/$path"; fi
            printf '%s  %s\n' "$hash" "$actual" | sha256sum --check --status
        done <<< "$rows"
    done <<< "$manifests"
}

run_checked() {
    local name=$1 expected=$2 minimum=$3
    shift 3
    local log="$directory/$name.log" code=0 counts passed failed ignored
    printf 'Checking %s\n' "$name"
    "$@" > "$log" 2>&1 || code=$?
    counts=$(sed -nE 's/^test result: ok\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored;.*/\1 \2 \3/p' "$log" |
        jq -Rsc '[split("\n")[] | select(length>0) | split(" ") | map(tonumber) |
            {passed:.[0],failed:.[1],ignored:.[2]}]')
    passed=$(jq '[.[].passed] | add // 0' <<< "$counts")
    failed=$(jq '[.[].failed] | add // 0' <<< "$counts")
    ignored=$(jq '[.[].ignored] | add // 0' <<< "$counts")
    jq --arg name "$name" --arg log "$log" --argjson exitCode "$code" --argjson counts "$counts" \
        --argjson passed "$passed" '. + [{name:$name,log:$log,exitCode:$exitCode,passed:$passed,counts:$counts}]' \
        "$directory/checks.json" > "$directory/checks.tmp"
    mv -- "$directory/checks.tmp" "$directory/checks.json"
    if (( code != 0 || failed != 0 || passed < minimum || (expected >= 0 && (passed != expected || ignored != 0)) )); then
        tail -n 70 "$log" >&2
        printf 'Failed %s: exit=%s passed=%s expected=%s minimum=%s\n' "$name" "$code" "$passed" "$expected" "$minimum" >&2
        return 1
    fi
    printf '%s passed=%s\n' "$name" "$passed"
}

build_native_mocks() {
    export MJWARP_NATIVE_MOCKS="$directory/mocks"
    export MJWARP_NATIVE_LIFETIME_LOG="$directory/lifetime.log"
    mkdir -p -- "$MJWARP_NATIVE_MOCKS"
    local name definitions
    while IFS=: read -r name definitions; do
        local flags=()
        read -r -a flags <<< "$definitions"
        g++ -shared -fPIC -std=c++17 -I "$MJWARP_MUJOCO_ROOT/include" "${flags[@]}" \
            "$root/tests/native/mock_mujoco.cpp" -o "$MJWARP_NATIVE_MOCKS/$name.so"
    done <<'CASES'
wrong-version:-DMOCK_VERSION=123
missing-load:-DMOCK_MISSING_LOAD
missing-version:-DMOCK_MISSING_VERSION
missing-delete:-DMOCK_MISSING_DELETE
tracking:-DMOCK_TRACKING
invalid-counts:-DMOCK_TRACKING -DMOCK_INVALID_COUNTS
missing-kinematic-pointer:-DMOCK_TRACKING -DMOCK_MISSING_KINEMATIC_POINTER
missing-inertial-pointer:-DMOCK_TRACKING -DMOCK_MISSING_INERTIAL_POINTER
missing-flex-pointer:-DMOCK_TRACKING -DMOCK_MISSING_FLEX_POINTER
invalid-flex-counts:-DMOCK_TRACKING -DMOCK_INVALID_FLEX_COUNTS
oversized-flex-counts:-DMOCK_TRACKING -DMOCK_OVERSIZED_FLEX_COUNTS
g01-missing-size:-DMOCK_TRACKING -DMOCK_G01 -DMOCK_G01_MISSING_SIZE
g01-negative-site:-DMOCK_TRACKING -DMOCK_G01 -DMOCK_G01_NEGATIVE_SITE
g01-oversized-camera:-DMOCK_TRACKING -DMOCK_G01 -DMOCK_G01_OVERSIZED_CAMERA
direct-flex:-DMOCK_TRACKING -DMOCK_FLEX_DIRECT
CASES
}
