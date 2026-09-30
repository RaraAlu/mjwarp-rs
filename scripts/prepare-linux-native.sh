#!/usr/bin/env bash
set -euo pipefail
[[ $(uname -s) == Linux && $(uname -m) == x86_64 ]] || { echo 'Requires Linux x86_64' >&2; exit 1; }
root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
destination="$root/target/toolchains/mujoco-3.12.0"
name=mujoco-3.12.0-linux-x86_64.tar.gz
hash=a9367911e6d5eaeade17c2197304687421c1fc932cdf7bcd4cb8cfaf0374dcb2
archive="$destination/downloads/$name"
mkdir -p -- "$destination/downloads"
if [[ ! -f $archive ]]; then
    curl -fL --retry 3 --connect-timeout 15 \
        "https://github.com/google-deepmind/mujoco/releases/download/3.12.0/$name" -o "$archive.part"
    mv -- "$archive.part" "$archive"
fi
printf '%s  %s\n' "$hash" "$archive" | sha256sum --check --status
# Extract only a checksum-verified official archive. Do not compile any model.
mkdir -p -- "$destination/linux-package"
tar -xzf "$archive" -C "$destination/linux-package" --strip-components=1 --no-same-owner
package="$destination/linux-package"
[[ -f $package/include/mujoco/mujoco.h && -f $package/lib/libmujoco.so.3.12.0 ]]
printf 'MJWARP_MUJOCO_ROOT=%s\nMJWARP_MUJOCO_DLL=%s\n' \
    "$package" "$package/lib/libmujoco.so.3.12.0"
