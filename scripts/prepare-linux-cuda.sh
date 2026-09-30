#!/usr/bin/env bash
set -euo pipefail
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Requires Linux x86_64' >&2; exit 1; }
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
destination="$root/target/toolchains/cuda-12.8.1"
base=https://developer.download.nvidia.com/compute/cuda/redist
mkdir -p -- "$destination/downloads" "$destination/lib64" "$destination/include" "$destination/bin"
curl -fsSL --retry 3 "$base/redistrib_12.8.1.json" -o "$destination/downloads/redistrib.json"
jq -e '.release_label == "12.8.1"' "$destination/downloads/redistrib.json" >/dev/null
records=()
for component in cuda_nvrtc cuda_cudart cuda_cccl cuda_nvcc; do
    record=$(jq -ce --arg component "$component" '.[$component] |
        {component:$component,version,license,archive:."linux-x86_64"} |
        select(.archive.sha256 | test("^[a-f0-9]{64}$"))' "$destination/downloads/redistrib.json")
    path=$(jq -r '.archive.relative_path' <<< "$record")
    [[ $path =~ ^$component/linux-x86_64/[a-zA-Z0-9_.-]+\.tar\.xz$ ]]
    hash=$(jq -r '.archive.sha256' <<< "$record")
    archive="$destination/downloads/${path##*/}"
    if [[ ! -f $archive ]]; then
        curl -fL --retry 3 --connect-timeout 15 "$base/$path" -o "$archive.part"
        mv -- "$archive.part" "$archive"
    fi
    printf '%s  %s\n' "$hash" "$archive" | sha256sum --check --status
    unpacked="$destination/$component"
    mkdir -p -- "$unpacked"
    tar -xJf "$archive" -C "$unpacked" --strip-components=1 --no-same-owner
    for folder in bin include nvvm; do
        if [[ -d $unpacked/$folder ]]; then
            mkdir -p -- "$destination/$folder"
            cp -a -- "$unpacked/$folder/." "$destination/$folder/"
        fi
    done
    if [[ -d $unpacked/lib ]]; then
        cp -a -- "$unpacked/lib/." "$destination/lib64/"
    fi
    records+=("$record")
done
printf '%s\n' "${records[@]}" | jq -s . > "$destination/components.json"
[[ -f $destination/lib64/libnvrtc.so.12 ]]
printf 'CUDA_PATH=%s\n' "$destination"
