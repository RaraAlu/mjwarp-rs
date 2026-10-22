#![cfg(feature = "cuda-probe")]
//! W15：Warp 头文件原样复制、include 路径与 NVRTC 编译门禁。
//! 产品测试只读仓内静态头文件，不启动上游作者流程。

use mjwarp_rs::runtime::launcher::{
    ArtifactCache, CompileRecipe, HeaderFile, NvrtcSource, RecipePurpose,
};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

const MANIFEST: &str =
    include_str!("../docs/development-evidence/w15-verbatim-headers/headers.json");
const LICENSE_SHA256: &str = "6d1d968fb225eca367cb7f0b8831ab012a35d92b547e945e17ef8e7b05c3e5cc";
const UPSTREAM_COMMIT: &str = "ffa038b1adf3927a8e893d4536f1a7562f17d749";
const PROBE: &str = r#"
#define WP_NO_BFLOAT16
#define WP_TILE_BLOCK_DIM 256
#define WP_NO_CRT
#include "builtin.h"
#include "deterministic.h"

extern "C" __global__ void w15_probe(float* out, int n) {
    int i = blockIdx.x * blockDim.x + threadIdx.x;
    if (i >= n) {
        return;
    }
    wp::vec3 a = wp::vec3(1.0f, 2.0f, 3.0f);
    wp::vec3 b = wp::vec3(0.5f, -1.0f, 0.25f);
    wp::vec3 c = a + b;
    out[i] = c[0] + c[1] + c[2];
}
"#;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn header_files(directory: &Path, current: &Path, found: &mut Vec<String>) {
    let mut entries: Vec<_> = std::fs::read_dir(current)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            header_files(directory, &entry, found);
        } else if entry.extension().and_then(|value| value.to_str()) == Some("h") {
            found.push(
                entry
                    .strip_prefix(directory)
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace('\\', "/"),
            );
        }
    }
}

#[test]
fn copied_headers_match_the_frozen_manifest() {
    let manifest: Value = serde_json::from_str(MANIFEST).unwrap();
    assert_eq!(manifest["upstream"]["commit"], UPSTREAM_COMMIT);
    assert_eq!(manifest["header_count"], 36);
    let directory = root().join("include/warp-headers");
    let mut listed = Vec::new();
    for header in manifest["headers"].as_array().unwrap() {
        let relative = header["path"].as_str().unwrap();
        let bytes = std::fs::read(directory.join(relative)).unwrap();
        assert_eq!(bytes.len() as u64, header["bytes"].as_u64().unwrap());
        assert_eq!(digest(&bytes), header["sha256"].as_str().unwrap());
        listed.push(relative.to_owned());
    }
    let mut on_disk = Vec::new();
    header_files(&directory, &directory, &mut on_disk);
    listed.sort();
    on_disk.sort();
    assert_eq!(on_disk, listed, "copied header set must equal the manifest");
    let license = std::fs::read(root().join("LICENSES/warp-1.15.0/LICENSE.md")).unwrap();
    assert_eq!(digest(&license), LICENSE_SHA256);
    assert!(String::from_utf8_lossy(&license).contains("Apache License"));
    let notices = std::fs::read_to_string(root().join("THIRD_PARTY_NOTICES.md")).unwrap();
    assert!(notices.contains(UPSTREAM_COMMIT));
    assert!(notices.contains("include/warp-headers"));
}

#[test]
fn headers_load_from_the_copied_directory_with_relative_paths() {
    let directory = root().join("include/warp-headers");
    let source = NvrtcSource::new("w15_probe", PROBE)
        .unwrap()
        .with_headers_from_dir(&directory)
        .unwrap();
    assert_eq!(source.headers().len(), 36);
    let names: Vec<_> = source.headers().iter().map(|h| h.name()).collect();
    assert!(names.contains(&"builtin.h"));
    assert!(names.contains(&"deterministic.h"));
    assert!(names.contains(&"nanovdb/PNanoVDB.h"));
    let mut sorted = names.clone();
    sorted.sort_unstable();
    assert_eq!(names, sorted, "header order is deterministic");
    assert!(source.source().contains("#include \"builtin.h\""));
}

#[cfg(windows)]
#[test]
#[ignore = "requires NVRTC and the CUDA SDK include directory (MJWARP_CUDA_INCLUDE)"]
fn nvrtc_compiles_probe_with_verbatim_warp_headers() {
    let cuda_include = std::env::var("MJWARP_CUDA_INCLUDE")
        .expect("set MJWARP_CUDA_INCLUDE to the CUDA 12.8 include directory");
    let headers = root().join("include/warp-headers");
    let fp16 = Path::new(&cuda_include).join("cuda_fp16.h");
    let build = || {
        NvrtcSource::new("w15_probe", PROBE)
            .unwrap()
            .with_headers_from_dir(&headers)
            .unwrap()
            .with_include_path(cuda_include.clone())
            .unwrap()
            .with_external_header(&fp16)
            .unwrap()
    };
    let source = build();
    let cache = ArtifactCache::in_memory();
    let recipe = CompileRecipe::WARP_1_15_DIAGNOSTIC;
    let artifact = cache
        .compile(&source, &recipe, RecipePurpose::Diagnostic)
        .unwrap();
    assert!(artifact.ptx().contains(".version"));
    assert!(artifact.ptx().contains("w15_probe"));
    assert_eq!(cache.stats().compiles, 1);
    let again = cache
        .compile(&source, &recipe, RecipePurpose::Diagnostic)
        .unwrap();
    assert_eq!(again.key_sha256(), artifact.key_sha256());
    assert_eq!(
        cache.stats().compiles,
        1,
        "second compile must hit the cache"
    );
    let changed = source
        .clone()
        .with_header(HeaderFile::new("w15_extra.h", "// extra").unwrap())
        .unwrap();
    let second = cache
        .compile(&changed, &recipe, RecipePurpose::Diagnostic)
        .unwrap();
    assert_ne!(second.key_sha256(), artifact.key_sha256());
    assert_eq!(cache.stats().compiles, 2, "header change must recompile");
}
