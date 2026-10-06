use mjwarp_rs::runtime::{
    ArtifactReport, ProbeBackend, ProbeConfig, ProbeKernel, ProbeReport, ResourceProbeConfig,
    ResourceProbeReport, build_probe_artifact, run_cached_probe, run_external_resource_probe,
    run_probe, run_resource_probe,
};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "mjwarp-rs probe [--backend native-ptx|cubecl-cpp|cubecl-llvm] [--kernel affine|atomic-sum|float-atomic-sum|block-reduce|block-scan|global-scan|control-flow|small-solve] [--device N] [--elements N] [--replays N]\nmjwarp-rs resources [--backend ROUTE] [--device N] [--elements N]\nmjwarp-rs external-resources [--backend ROUTE] [--device N] [--elements N]\nmjwarp-rs cache-build --cache DIR [--backend ROUTE] [--kernel KERNEL] [--device N] [--refresh]\nmjwarp-rs cache-run --cache DIR --trust-cache [--backend ROUTE] [--kernel KERNEL] [--device N] [--elements N] [--replays N] [--require-no-nvrtc]\n原生：cargo run --features cuda-probe -- probe\nC++：cargo run --features cubecl-cpp-probe -- probe --backend cubecl-cpp\nLLVM：cargo run --features cubecl-llvm-probe -- probe --backend cubecl-llvm\n缓存摘要不认证代码来源。\n探针不提供物理引擎。";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Probe,
    Resources,
    ExternalResources,
    CacheBuild,
    CacheRun,
}

struct Command {
    config: ProbeConfig,
    kind: Kind,
    cache: Option<PathBuf>,
    refresh: bool,
    require_no_nvrtc: bool,
}

fn parse_args(args: &[String]) -> Result<Option<Command>, String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(None);
    }
    let kind = match args.first().map(String::as_str) {
        Some("probe") => Kind::Probe,
        Some("resources") => Kind::Resources,
        Some("external-resources") => Kind::ExternalResources,
        Some("cache-build") => Kind::CacheBuild,
        Some("cache-run") => Kind::CacheRun,
        _ => return Err("请使用探针或缓存命令".into()),
    };
    let mut config = ProbeConfig::default();
    let mut cache = None;
    let mut refresh = false;
    let mut require_no_nvrtc = false;
    let mut trust_cache = false;
    let mut seen = [false; 9];
    let mut flags = args[1..].iter();
    while let Some(flag) = flags.next() {
        if matches!(kind, Kind::Resources | Kind::ExternalResources)
            && matches!(flag.as_str(), "--kernel" | "--replays")
        {
            return Err(format!("资源探针不支持：{flag}"));
        }
        if kind == Kind::CacheBuild && matches!(flag.as_str(), "--elements" | "--replays") {
            return Err(format!("缓存构建不支持：{flag}"));
        }
        let extra = match flag.as_str() {
            "--cache" => Some((5, matches!(kind, Kind::CacheBuild | Kind::CacheRun))),
            "--refresh" => Some((6, kind == Kind::CacheBuild)),
            "--trust-cache" => Some((7, kind == Kind::CacheRun)),
            "--require-no-nvrtc" => Some((8, kind == Kind::CacheRun)),
            _ => None,
        };
        if let Some((slot, allowed)) = extra {
            if !allowed {
                return Err(format!("命令不支持：{flag}"));
            }
            if seen[slot] {
                return Err(format!("重复参数：{flag}"));
            }
            seen[slot] = true;
            match slot {
                5 => {
                    let value = flags.next().ok_or_else(|| format!("缺少参数值：{flag}"))?;
                    if value.is_empty() || value.starts_with("--") {
                        return Err("缓存路径无效".into());
                    }
                    cache = Some(PathBuf::from(value));
                }
                6 => refresh = true,
                7 => trust_cache = true,
                _ => require_no_nvrtc = true,
            }
            continue;
        }
        if flag == "--backend" {
            if seen[3] {
                return Err(format!("重复参数：{flag}"));
            }
            seen[3] = true;
            let value = flags.next().ok_or_else(|| format!("缺少参数值：{flag}"))?;
            config.backend = ProbeBackend::parse(value).map_err(|error| error.to_string())?;
            continue;
        }
        if flag == "--kernel" {
            if seen[4] {
                return Err(format!("重复参数：{flag}"));
            }
            seen[4] = true;
            let value = flags.next().ok_or_else(|| format!("缺少参数值：{flag}"))?;
            config.kernel = ProbeKernel::parse(value).map_err(|error| error.to_string())?;
            continue;
        }
        let (slot, field) = match flag.as_str() {
            "--device" => (0, &mut config.device),
            "--elements" => (1, &mut config.elements),
            "--replays" => (2, &mut config.replays),
            _ => return Err(format!("未知参数：{flag}")),
        };
        if seen[slot] {
            return Err(format!("重复参数：{flag}"));
        }
        seen[slot] = true;
        let value = flags.next().ok_or_else(|| format!("缺少参数值：{flag}"))?;
        *field = value.parse().map_err(|_| format!("需要非负整数：{flag}"))?;
    }
    config.validate().map_err(|error| error.to_string())?;
    if matches!(kind, Kind::CacheBuild | Kind::CacheRun) && cache.is_none() {
        return Err("缓存命令需要--cache".into());
    }
    if kind == Kind::CacheRun && !trust_cache {
        return Err("可信产物需要--trust-cache".into());
    }
    Ok(Some(Command {
        config,
        kind,
        cache,
        refresh,
        require_no_nvrtc,
    }))
}

fn resources(config: ProbeConfig, external: bool) -> ExitCode {
    let config = ResourceProbeConfig {
        backend: config.backend,
        device: config.device,
        elements: config.elements,
    };
    let result = if external {
        run_external_resource_probe(config).map(|report| {
            println!(
                "imports={}; owner_releases={}; producer_dependencies={}",
                report.imported_allocations, report.released_owners, report.producer_dependencies
            );
            report.resources
        })
    } else {
        run_resource_probe(config)
    };
    match result {
        Ok(report) => {
            print_resources(&report);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let command = match parse_args(&args) {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            eprintln!("{error}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if matches!(command.kind, Kind::Resources | Kind::ExternalResources) {
        return resources(command.config, command.kind == Kind::ExternalResources);
    }
    let config = command.config;
    if let Some(directory) = &command.cache {
        if command.kind == Kind::CacheBuild {
            return match build_probe_artifact(config, directory, command.refresh) {
                Ok(report) => {
                    print_artifact(&report);
                    ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::FAILURE
                }
            };
        }
        // SAFETY: 用户显式声明信任目录。
        // --trust-cache要求可信探针产物。
        return match unsafe { run_cached_probe(config, directory, command.require_no_nvrtc) } {
            Ok(report) => {
                print_artifact(&report.artifact);
                if command.require_no_nvrtc {
                    println!("nvrtc_available=false");
                }
                print_probe(&report.probe);
                ExitCode::SUCCESS
            }
            Err(error) => {
                eprintln!("{error}");
                ExitCode::FAILURE
            }
        };
    }
    match run_probe(config) {
        Ok(report) => {
            print_probe(&report);
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn print_artifact(report: &ArtifactReport) {
    println!("artifact={}", report.path.display());
    println!("cache_status={:?}; stages={}", report.status, report.stages);
    println!("cache_key_sha256={}", report.key_sha256);
}

fn print_resources(report: &ResourceProbeReport) {
    println!("platform={}/{}", report.os, report.arch);
    println!("device={}: {}", report.device, report.gpu_name);
    println!(
        "sm={}.{}; driver_api={}",
        report.compute_capability.0, report.compute_capability.1, report.driver_api_version
    );
    println!("route={}", report.backend.name());
    if let Some(revision) = report.compiler_revision {
        println!("cubecl_revision={revision}");
    }
    println!(
        "elements={}; view_bytes={}; output_owner_bytes={}",
        report.elements, report.view_bytes, report.output_owner_bytes
    );
    println!(
        "submissions={}; host_failures={}; rejections={}; graph_replays={}",
        report.completed_submissions,
        report.failed_submissions,
        report.rejected_requests,
        report.graph_replays
    );
    println!("owner/offset-view/cross-stream/token-drop/token-forget/queue-drop/guard=pass");
    println!("探针不冻结外部设备ABI。");
}

fn print_probe(report: &ProbeReport) {
    println!("platform={}/{}", report.os, report.arch);
    println!("device={}: {}", report.device, report.gpu_name);
    println!(
        "sm={}.{}",
        report.compute_capability.0, report.compute_capability.1
    );
    println!("driver_api={}", report.driver_api_version);
    println!("route={}", report.backend.name());
    println!("kernel={}", report.kernel.name());
    if let Some(revision) = report.compiler_revision {
        println!("cubecl_revision={revision}");
    } else {
        println!("ptx=6.0; target=sm_70");
    }
    println!(
        "elements={}; output_bytes={}",
        report.elements, report.buffer_bytes
    );
    println!("upload/kernel/download/cross-stream/guard=pass");
    println!("graph_replays={}; changed_input=pass", report.graph_replays);
    println!(
        "graph_kernel_nodes={}; node_updates={}; inactive_output=pass",
        report.graph_kernel_nodes, report.graph_node_updates
    );
    println!("此结果只验证当前路线。");
    println!("引擎与正式准入仍待完成。");
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).into()).collect()
    }

    #[test]
    fn accepts_help_defaults_and_explicit_values() {
        assert!(parse_args(&args(&["--help"])).unwrap().is_none());
        assert_eq!(
            parse_args(&args(&["probe"]))
                .unwrap()
                .unwrap()
                .config
                .elements,
            257
        );
        let config = parse_args(&args(&[
            "probe",
            "--backend",
            "cubecl-cpp",
            "--device",
            "2",
            "--elements",
            "129",
            "--replays",
            "4",
        ]))
        .unwrap()
        .unwrap()
        .config;
        assert_eq!(
            (config.device, config.elements, config.replays),
            (2, 129, 4)
        );
        assert_eq!(config.backend, ProbeBackend::CubeClCpp);
    }

    #[test]
    fn rejects_bad_commands_flags_and_values() {
        for values in [
            vec![],
            vec!["run"],
            vec!["probe", "--unknown"],
            vec!["probe", "--device"],
            vec!["probe", "--device", "-1"],
            vec!["probe", "--elements", "0"],
            vec!["probe", "--replays", "no"],
            vec!["probe", "--device", "0", "--device", "1"],
            vec!["probe", "--backend"],
            vec!["probe", "--backend", "cpu"],
            vec!["probe", "--backend", "auto"],
            vec!["probe", "--kernel"],
            vec!["probe", "--kernel", "unknown"],
            vec!["probe", "--kernel", "atomic-sum"],
            vec!["probe", "--kernel", "affine", "--kernel", "affine"],
            vec![
                "probe",
                "--backend",
                "cubecl-cpp",
                "--backend",
                "cubecl-llvm",
            ],
        ] {
            assert!(parse_args(&args(&values)).is_err(), "{values:?}");
        }
    }

    #[test]
    fn accepts_explicit_cubecl_kernels() {
        for kernel in ProbeKernel::ALL {
            let config = parse_args(&args(&[
                "probe",
                "--backend",
                "cubecl-llvm",
                "--kernel",
                kernel.name(),
            ]))
            .unwrap()
            .unwrap()
            .config;
            assert_eq!(config.kernel, kernel);
        }
    }

    #[test]
    fn accepts_resource_command_and_rejects_kernel_options() {
        let command = parse_args(&args(&[
            "resources",
            "--backend",
            "cubecl-cpp",
            "--elements",
            "129",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(command.kind, Kind::Resources);
        assert_eq!(command.config.elements, 129);
        assert_eq!(command.config.backend, ProbeBackend::CubeClCpp);
        for values in [
            vec!["resources", "--kernel", "affine"],
            vec!["resources", "--replays", "1"],
            vec!["resources", "--elements", "0"],
            vec!["resources", "--backend", "cpu"],
            vec!["resources", "--elements", "1", "--elements", "2"],
        ] {
            assert!(parse_args(&args(&values)).is_err());
        }
    }

    #[test]
    fn accepts_external_resources_and_rejects_unrelated_options() {
        let command = parse_args(&args(&[
            "external-resources",
            "--backend",
            "cubecl-llvm",
            "--elements",
            "129",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(command.kind, Kind::ExternalResources);
        assert_eq!(command.config.elements, 129);
        assert_eq!(command.config.backend, ProbeBackend::CubeClLlvm);
        for flag in [
            "--kernel",
            "--replays",
            "--cache",
            "--refresh",
            "--trust-cache",
            "--require-no-nvrtc",
        ] {
            assert!(parse_args(&args(&["external-resources", flag, "1"])).is_err());
        }
        for values in [
            vec!["external-resources", "--elements", "0"],
            vec!["external-resources", "--backend", "cpu"],
            vec!["external-resources", "--elements", "1", "--elements", "2"],
        ] {
            assert!(parse_args(&args(&values)).is_err());
        }
    }

    #[test]
    fn requires_explicit_cache_path_and_trust() {
        for values in [
            vec!["cache-build"],
            vec!["cache-run", "--cache", "trusted"],
            vec!["cache-run", "--trust-cache"],
            vec!["cache-build", "--cache", ""],
            vec!["cache-build", "--cache", "--refresh"],
        ] {
            assert!(parse_args(&args(&values)).is_err(), "{values:?}");
        }
        let command = parse_args(&args(&[
            "cache-run",
            "--cache",
            "trusted",
            "--trust-cache",
            "--require-no-nvrtc",
            "--backend",
            "cubecl-llvm",
            "--kernel",
            "global-scan",
            "--elements",
            "16385",
        ]))
        .unwrap()
        .unwrap();
        assert_eq!(command.kind, Kind::CacheRun);
        assert_eq!(command.cache, Some(PathBuf::from("trusted")));
        assert!(command.require_no_nvrtc);
        assert_eq!(command.config.elements, 16385);
        let command = parse_args(&args(&["cache-build", "--cache", "trusted", "--refresh"]))
            .unwrap()
            .unwrap();
        assert!(command.refresh);
    }

    #[test]
    fn rejects_irrelevant_and_duplicate_cache_flags() {
        for values in [
            vec!["probe", "--cache", "a"],
            vec!["resources", "--refresh"],
            vec!["cache-build", "--cache", "a", "--elements", "1"],
            vec!["cache-build", "--cache", "a", "--replays", "1"],
            vec!["cache-build", "--cache", "a", "--trust-cache"],
            vec!["cache-build", "--cache", "a", "--require-no-nvrtc"],
            vec!["cache-run", "--cache", "a", "--trust-cache", "--refresh"],
            vec!["cache-build", "--cache", "a", "--cache", "b"],
            vec!["cache-build", "--cache", "a", "--refresh", "--refresh"],
            vec![
                "cache-run",
                "--cache",
                "a",
                "--trust-cache",
                "--trust-cache",
            ],
            vec![
                "cache-run",
                "--cache",
                "a",
                "--trust-cache",
                "--require-no-nvrtc",
                "--require-no-nvrtc",
            ],
        ] {
            assert!(parse_args(&args(&values)).is_err(), "{values:?}");
        }
    }
}
