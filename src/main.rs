use mjwarp_rs::runtime::{
    ProbeBackend, ProbeConfig, ProbeKernel, ResourceProbeConfig, run_probe, run_resource_probe,
};
use std::process::ExitCode;

const USAGE: &str = "mjwarp-rs probe [--backend native-ptx|cubecl-cpp|cubecl-llvm] [--kernel affine|atomic-sum|float-atomic-sum|block-reduce|block-scan|global-scan|control-flow|small-solve] [--device N] [--elements N] [--replays N]\nmjwarp-rs resources [--backend native-ptx|cubecl-cpp|cubecl-llvm] [--device N] [--elements N]\n原生：cargo run --features cuda-probe -- probe\nC++：cargo run --features cubecl-cpp-probe -- probe --backend cubecl-cpp\nLLVM：cargo run --features cubecl-llvm-probe -- probe --backend cubecl-llvm\n探针不提供物理引擎。";

struct Command {
    config: ProbeConfig,
    resources: bool,
}

fn parse_args(args: &[String]) -> Result<Option<Command>, String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(None);
    }
    let resources = match args.first().map(String::as_str) {
        Some("probe") => false,
        Some("resources") => true,
        _ => return Err("请使用probe或resources命令".into()),
    };
    let mut config = ProbeConfig::default();
    let mut seen = [false; 5];
    let mut flags = args[1..].iter();
    while let Some(flag) = flags.next() {
        if resources && matches!(flag.as_str(), "--kernel" | "--replays") {
            return Err(format!("资源探针不支持：{flag}"));
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
    Ok(Some(Command { config, resources }))
}

fn resources(config: ProbeConfig) -> ExitCode {
    match run_resource_probe(ResourceProbeConfig {
        backend: config.backend,
        device: config.device,
        elements: config.elements,
    }) {
        Ok(report) => {
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
            println!(
                "owner/offset-view/cross-stream/token-drop/token-forget/queue-drop/guard=pass"
            );
            println!("探针不冻结外部设备ABI。");
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
    if command.resources {
        return resources(command.config);
    }
    let config = command.config;
    match run_probe(config) {
        Ok(report) => {
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
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
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
        assert!(command.resources);
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
}
