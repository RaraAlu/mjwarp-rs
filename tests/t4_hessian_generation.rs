//! 只核对生成前置与累加风险。
//! 宿主反例不提供物理准入。
#![cfg(feature = "cuda-probe")]

use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::PathBuf};

const MAP: &str = include_str!("../docs/evidence/t4-g03-retest/hessian-generation/map.json");

#[derive(Deserialize)]
struct Stamp {
    path: String,
    sha256: String,
}

#[derive(Deserialize)]
struct Source {
    id: String,
    #[serde(flatten)]
    stamp: Stamp,
}

#[derive(Deserialize)]
struct Probe {
    source: String,
    symbol: Option<String>,
    anchors: Vec<String>,
    #[serde(default)]
    ordered: Vec<String>,
}

#[derive(Deserialize)]
struct Map {
    sources: Vec<Source>,
    prerequisites: Vec<Probe>,
    accumulation: Vec<Probe>,
    cache_root: String,
    cache_files: Vec<Stamp>,
    actual_hessian_generated_artifact: bool,
    metadata_proves_historical_compile_environment: bool,
    full_acceptance: bool,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn map() -> Map {
    let map: Map = serde_json::from_str(MAP).unwrap();
    assert!(!map.actual_hessian_generated_artifact);
    assert!(!map.metadata_proves_historical_compile_environment);
    assert!(!map.full_acceptance);
    map
}

fn sources(map: &Map) -> BTreeMap<String, String> {
    let mut texts = BTreeMap::new();
    for source in &map.sources {
        let text = fs::read_to_string(root().join(&source.stamp.path))
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(digest(text.as_bytes()), source.stamp.sha256);
        assert!(texts.insert(source.id.clone(), text).is_none());
    }
    assert_eq!(texts.len(), 11);
    texts
}

// 冻结源码采用顶层函数；不解析或执行Python。
fn section<'a>(text: &'a str, symbol: Option<&str>) -> Result<&'a str, String> {
    let Some(symbol) = symbol else {
        return Ok(text);
    };
    let marker = format!("\ndef {symbol}(");
    let (_, tail) = text.split_once(&marker).ok_or("missing function")?;
    let end = ["\n\ndef ", "\n\n@"]
        .iter()
        .filter_map(|marker| tail.find(marker))
        .min()
        .unwrap_or(tail.len());
    Ok(&tail[..end])
}

fn probes(probes: &[Probe], texts: &BTreeMap<String, String>) -> Result<(), String> {
    for probe in probes {
        let text = texts.get(&probe.source).ok_or("unknown source")?;
        let text = section(text, probe.symbol.as_deref())?;
        for anchor in &probe.anchors {
            if !text.contains(anchor) {
                return Err(format!("missing anchor: {}: {anchor}", probe.source));
            }
        }
        let mut tail = text;
        for anchor in &probe.ordered {
            tail = tail.split_once(anchor).ok_or("phase order")?.1;
        }
    }
    Ok(())
}

#[test]
fn checks_hessian_generation_prerequisites() {
    let map = map();
    let texts = sources(&map);
    assert_eq!(map.prerequisites.len(), 9);
    probes(&map.prerequisites, &texts).unwrap();
    let mut changed = texts;
    let revised = changed["passive"].replace("if m.nflexelem > 0:", "if False:");
    changed.insert("passive".into(), revised);
    assert!(probes(&map.prerequisites, &changed).is_err());
}

fn cache(map: &Map, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    if files.len() != map.cache_files.len() || files.len() != 3 {
        return Err("cache denominator".into());
    }
    for stamp in &map.cache_files {
        let bytes = files.get(&stamp.path).ok_or("cache missing")?;
        if digest(bytes) != stamp.sha256 {
            return Err("cache hash".into());
        }
        let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
        if text.contains("_flex_hessian_") {
            return Err("Hessian cache boundary changed".into());
        }
    }
    Ok(())
}

fn walk(dir: &std::path::Path, base: &std::path::Path, files: &mut BTreeMap<String, Vec<u8>>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            walk(&path, base, files);
        } else {
            let name = path
                .strip_prefix(base)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            assert!(files.insert(name, fs::read(path).unwrap()).is_none());
        }
    }
}

#[test]
fn checks_cached_artifact_boundary_and_rejects_forgery() {
    let map = map();
    let base = root().join(&map.cache_root);
    let mut files = BTreeMap::new();
    walk(&base, &base, &mut files);
    cache(&map, &files).unwrap();
    let key = map.cache_files[0].path.clone();
    let original = files.remove(&key).unwrap();
    assert!(cache(&map, &files).is_err());
    files.insert(key.clone(), b"_flex_hessian_elem".to_vec());
    assert!(cache(&map, &files).is_err());
    files.insert(key, original);
    files.insert("extra.ptx".into(), vec![]);
    assert!(cache(&map, &files).is_err());
}

#[test]
fn demonstrates_atomic_order_and_zero_sign_risks() {
    let map = map();
    let texts = sources(&map);
    assert_eq!(map.accumulation.len(), 7);
    probes(&map.accumulation, &texts).unwrap();
    // 宿主f32反例只证明重排风险；不模拟设备原子调度。
    let a = std::hint::black_box(16_777_216.0_f32);
    let b = std::hint::black_box(1.0_f32);
    let c = std::hint::black_box(-16_777_216.0_f32);
    assert_eq!(((a + b) + c).to_bits(), 0.0_f32.to_bits());
    assert_eq!(((a + c) + b).to_bits(), 1.0_f32.to_bits());
    let zero = std::hint::black_box(0.0_f32);
    let negative_zero = -zero;
    assert_eq!(negative_zero.to_bits(), 0x8000_0000);
    assert_eq!((zero - zero).to_bits(), 0);
    assert_eq!((zero + negative_zero).to_bits(), 0);
    assert_eq!((negative_zero * b).to_bits(), 0x8000_0000);
    // 零符号中间量不同，不意味着最终和必定不同。
}
