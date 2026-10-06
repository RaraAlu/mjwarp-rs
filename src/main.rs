use mjwarp_rs::runtime::{ProbeBackend, ProbeConfig, ProbeKernel, run_probe};
use std::process::ExitCode;

const USAGE: &str = "mjwarp-rs probe [--backend native-ptx|cubecl-cpp|cubecl-llvm] [--kernel affine|atomic-sum|block-reduce|block-scan|control-flow|small-solve] [--device N] [--elements N] [--replays N]\n原生：cargo run --features cuda-probe -- probe\nC++：cargo run --features cubecl-cpp-probe -- probe --backend cubecl-cpp\nLLVM：cargo run --features cubecl-llvm-probe -- probe --backend cubecl-llvm\n探针不提供物理引擎。";

fn parse_args(args: &[String]) -> Result<Option<ProbeConfig>, String> {
    if args.len() == 1 && matches!(args[0].as_str(), "--help" | "-h") {
        return Ok(None);
    }
    if args.first().map(String::as_str) != Some("probe") {
        return Err("请使用probe命令".into());
    }
    let mut config = ProbeConfig::default();
    let mut seen = [false; 5];
    let mut flags = args[1..].iter();
    while let Some(flag) = flags.next() {
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
    Ok(Some(config))
}

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let config = match parse_args(&args) {
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
            parse_args(&args(&["probe"])).unwrap().unwrap().elements,
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
        .unwrap();
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
            .unwrap();
            assert_eq!(config.kernel, kernel);
        }
    }
}
