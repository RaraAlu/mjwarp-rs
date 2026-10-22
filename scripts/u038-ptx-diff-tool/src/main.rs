//! U038 三维运动学逐表达式 PTX 对照工具（只读、只编译不跑内核）。
//!
//! 用法见 `scripts/u038-ptx-diff.ps1`；本工具不修改产品源码、不启动 GPU 内核。

mod diff;
mod extract;
mod nvrtc;
mod probe;
mod ptx;
mod report;
mod support;

use diff::kind_sequence;
use extract::{raw_const, read_text, sha256_hex};
use ptx::Program;
use serde_json::json;
use std::path::{Path, PathBuf};
use support::*;

const PRODUCT_KIN_ENTRY: &str = "rigid_kinematics";
const PRODUCT_COM_ENTRY: &str = "rigid_com_position";
const UPSTREAM_KIN_ENTRY: &str = "_kinematics_branch_95f8028e_cuda_kernel_forward";
const UPSTREAM_COM_KERNELS: [&str; 4] = [
    "_subtree_com_init_002b02fd_cuda_kernel_forward",
    "_subtree_com_acc_350f0560_cuda_kernel_forward",
    "_subtree_div_0ef57c26_cuda_kernel_forward",
    "_cdof_3e73eb78_cuda_kernel_forward",
];
const EXPRESSIONS: [(&str, usize); 3] = [("rotate", 5), ("multiply", 3), ("normalize", 3)];

fn main() {
    if let Err(error) = run() {
        eprintln!("错误：{error}");
        std::process::exit(1);
    }
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|value| value == key)
        .and_then(|index| args.get(index + 1).cloned())
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let default_repo = manifest
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "无法推导仓库根目录".to_string())?
        .to_path_buf();
    let repo =
        PathBuf::from(arg(&args, "--repo").unwrap_or_else(|| default_repo.display().to_string()));
    let cache = PathBuf::from(arg(&args, "--cache").unwrap_or_else(|| {
        repo.join("target/mjwarp-parallel/cache-quarantine/warp-1.15.0-double-version-2026-10-18/wp_mujoco_warp._src.smooth_ca013d6")
            .display()
            .to_string()
    }));
    let warp_include = PathBuf::from(arg(&args, "--warp-include").unwrap_or_else(|| {
        repo.join("target/mjwarp-parallel/venv/Lib/site-packages/warp/native")
            .display()
            .to_string()
    }));
    let out = PathBuf::from(arg(&args, "--out").unwrap_or_else(|| {
        repo.join("target/u038-ptx-diff-tool/out")
            .display()
            .to_string()
    }));
    let cuda_bin = PathBuf::from(arg(&args, "--cuda-bin").unwrap_or_else(|| {
        repo.join("target/toolchains/cuda-12.8.1/bin")
            .display()
            .to_string()
    }));
    std::fs::create_dir_all(&out).map_err(|e| format!("创建输出目录: {e}"))?;

    // ---------- 输入 ----------
    let mod_text = read_text(&repo.join("src/physics/mod.rs"))?;
    let kin_cuda = raw_const(&mod_text, "KINEMATICS_CUDA")?;
    let com_cuda = raw_const(&mod_text, "COM_POSITION_CUDA")?;
    let pack_text = read_text(&repo.join("src/physics/resident/pack_batches.rs"))?;
    let field_batch = raw_const(&pack_text, "FIELD_BATCH_CUDA")?;
    let smooth_cu = cache.join("wp_mujoco_warp._src.smooth_ca013d6.cu");
    let frozen_ptx = cache.join("wp_mujoco_warp._src.smooth_ca013d6.sm89.ptx");
    let smooth_text = read_text(&smooth_cu)?;
    let frozen_text = read_text(&frozen_ptx)?;

    let product_kin_source = format!("{field_batch}\n#define MJWARP_MOCAP\n{kin_cuda}");
    let product_com_source = format!("{field_batch}\n{com_cuda}");
    let product_probes = probe::product_probes(&kin_cuda)?;
    let upstream_probes = probe::upstream_probes(&smooth_text)?;

    // ---------- 编译 ----------
    let product_kin = compile(
        &product_kin_source,
        &product_options(false),
        "rigid_kinematics.cu",
    )?;
    write_file(&out.join("product-kinematics.ptx"), &product_kin.ptx)?;
    let product_kin_li = compile(
        &product_kin_source,
        &product_options(true),
        "rigid_kinematics.cu",
    )?;
    write_file(
        &out.join("product-kinematics.lineinfo.ptx"),
        &product_kin_li.ptx,
    )?;
    let product_com = compile(
        &product_com_source,
        &product_options(false),
        "rigid_com_position.cu",
    )?;
    write_file(&out.join("product-com.ptx"), &product_com.ptx)?;

    let product_ptx = Program::parse(&product_kin.ptx);
    let product_li_ptx = Program::parse(&product_kin_li.ptx);
    let product_com_ptx = Program::parse(&product_com.ptx);
    let product_kin_entry = require_entry(&product_ptx, PRODUCT_KIN_ENTRY)?;
    let product_com_entry = require_entry(&product_com_ptx, PRODUCT_COM_ENTRY)?;
    let lineinfo_invariant = instruction_diff(
        product_kin_entry,
        require_entry(&product_li_ptx, PRODUCT_KIN_ENTRY)?,
    )
    .0;

    let upstream = compile(
        &smooth_text,
        &warp_options(&warp_include, false),
        "smooth.cu",
    )?;
    write_file(&out.join("upstream-recompiled.ptx"), &upstream.ptx)?;
    let upstream_li = compile(
        &smooth_text,
        &warp_options(&warp_include, true),
        "smooth.cu",
    )?;
    write_file(
        &out.join("upstream-recompiled.lineinfo.ptx"),
        &upstream_li.ptx,
    )?;
    let upstream_ptx = Program::parse(&upstream.ptx);
    let frozen_program = Program::parse(&frozen_text);
    let upstream_kin_entry = require_entry(&upstream_ptx, UPSTREAM_KIN_ENTRY)?;

    let mut frozen_equal = 0usize;
    let mut frozen_compared = 0usize;
    let mut frozen_mismatched = Vec::new();
    for entry in &frozen_program.entries {
        let Some(other) = upstream_ptx.entry(&entry.name) else {
            continue;
        };
        frozen_compared += 1;
        let (same, _) = instruction_diff(entry, other);
        if same {
            frozen_equal += 1;
        } else {
            frozen_mismatched.push(entry.name.clone());
        }
    }
    // 带 lineinfo 的重编译也必须与冻结 PTX 指令流一致，否则行号映射不可信。
    let upstream_li_ptx = Program::parse(&upstream_li.ptx);
    let (li_frozen_equal, li_frozen_compared) = frozen_program
        .entries
        .iter()
        .filter_map(|entry| {
            upstream_li_ptx
                .entry(&entry.name)
                .map(|other| (entry, other))
        })
        .fold((0usize, 0usize), |(equal, compared), (entry, other)| {
            let (same, _) = instruction_diff(entry, other);
            (equal + usize::from(same), compared + 1)
        });

    // ---------- 探针 ----------
    let mut expression_rows = Vec::new();
    let mut in_situ_rows = Vec::new();
    let mut rotate_probe_ptx: Option<(String, String)> = None;
    for (id, expected) in EXPRESSIONS {
        let product_source = product_probes
            .iter()
            .find(|(name, _)| name == id)
            .map(|(_, source)| source.clone())
            .ok_or_else(|| format!("缺少产品探针 {id}"))?;
        let upstream_source = upstream_probes
            .iter()
            .find(|(name, _)| name == id)
            .map(|(_, source)| source.clone())
            .ok_or_else(|| format!("缺少上游探针 {id}"))?;
        let entry_name = probe::product_entry(id);

        let product = compile(&product_source, &product_options(false), "probe.cu")?;
        write_file(&out.join(format!("probe-product-{id}.ptx")), &product.ptx)?;
        write_file(&out.join(format!("probe-product-{id}.cu")), &product_source)?;
        let product_program = Program::parse(&product.ptx);
        let product_entry = require_entry(&product_program, &entry_name)?;

        let upstream_product_opts = compile(
            &upstream_source,
            &with_include(&product_options(false), &warp_include),
            "probe.cu",
        )?;
        write_file(
            &out.join(format!("probe-upstream-{id}.product-opts.ptx")),
            &upstream_product_opts.ptx,
        )?;
        write_file(
            &out.join(format!("probe-upstream-{id}.cu")),
            &upstream_source,
        )?;
        let upstream_product_program = Program::parse(&upstream_product_opts.ptx);
        let upstream_product_entry = require_entry(&upstream_product_program, &entry_name)?;

        let upstream_warp_opts = compile(
            &upstream_source,
            &warp_options(&warp_include, false),
            "probe.cu",
        )?;
        write_file(
            &out.join(format!("probe-upstream-{id}.warp-opts.ptx")),
            &upstream_warp_opts.ptx,
        )?;
        let upstream_warp_program = Program::parse(&upstream_warp_opts.ptx);
        let upstream_warp_entry = require_entry(&upstream_warp_program, &entry_name)?;

        let source_form = diff::compare(upstream_product_entry, product_entry);
        let product_occurrences =
            diff::occurrences(&product_kin_entry.kinds(), &product_entry.kinds());
        let frozen_occurrences = diff::occurrences(
            &frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?.kinds(),
            &upstream_warp_entry.kinds(),
        );
        let recompiled_occurrences =
            diff::occurrences(&upstream_kin_entry.kinds(), &upstream_warp_entry.kinds());
        let frozen_from_product_opts = diff::occurrences(
            &frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?.kinds(),
            &upstream_product_entry.kinds(),
        );
        if id == "rotate" {
            rotate_probe_ptx = Some((upstream_product_opts.ptx.clone(), product.ptx.clone()));
        }
        expression_rows.push(json!({
            "id": id,
            "product": entry_json(product_entry, &product),
            "upstream": entry_json(upstream_product_entry, &upstream_product_opts),
            "upstream_warp_options": entry_json(upstream_warp_entry, &upstream_warp_opts),
            "source_form_diff": comparison_json(&source_form),
            "warp_options_diff": comparison_json(&diff::compare(upstream_warp_entry, product_entry)),
            "expected_instances": expected,
        }));
        in_situ_rows.push(json!({
            "expression": id,
            "expected_instances": expected,
            "product_kernel": PRODUCT_KIN_ENTRY,
            "product_probe_occurrences": product_occurrences,
            "upstream_frozen_kernel": UPSTREAM_KIN_ENTRY,
            "upstream_probe_warp_options_occurrences": frozen_occurrences,
            "upstream_probe_product_options_occurrences": frozen_from_product_opts,
            "upstream_recompiled_occurrences": recompiled_occurrences,
        }));
    }

    // ---------- rotate 候选转录验证（只读诊断，不进入产品源码） ----------
    let candidate_source = probe::product_rotate_candidate(&kin_cuda)?;
    let candidate = compile(&candidate_source, &product_options(false), "probe.cu")?;
    write_file(
        &out.join("probe-product-rotate-candidate.ptx"),
        &candidate.ptx,
    )?;
    write_file(
        &out.join("probe-product-rotate-candidate.cu"),
        &candidate_source,
    )?;
    let candidate_program = Program::parse(&candidate.ptx);
    let candidate_entry = require_entry(&candidate_program, "probe_rotate_candidate")?;
    let candidate_row = match rotate_probe_ptx {
        Some((upstream_text, product_text)) => {
            let upstream_program = Program::parse(&upstream_text);
            let product_program = Program::parse(&product_text);
            let upstream_entry = require_entry(&upstream_program, "probe_rotate")?;
            let product_entry = require_entry(&product_program, "probe_rotate")?;
            json!({
                "expression": "rotate",
                "product_source_form": comparison_json(&diff::compare(upstream_entry, product_entry)),
                "candidate_source_form": comparison_json(&diff::compare(upstream_entry, candidate_entry)),
                "candidate_entry": entry_json(candidate_entry, &candidate),
                "candidate_matches_upstream_source_form": diff::compare(upstream_entry, candidate_entry).diff_type == "一致",
                "candidate_occurrences_in_product_kernel": diff::occurrences(&product_kin_entry.kinds(), &candidate_entry.kinds()),
                "candidate_occurrences_in_frozen_upstream_kernel": diff::occurrences(&frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?.kinds(), &candidate_entry.kinds()),
            })
        }
        None => json!({"expression": "rotate", "missing": true}),
    };

    // ---------- 上下文敏感性：同一表达式换包装加载顺序 ----------
    let product_v_first_source = probe::product_rotate_v_first(&kin_cuda)?;
    let upstream_v_first_source = probe::upstream_rotate_v_first(&smooth_text)?;
    let candidate_v_first_source = probe::product_rotate_candidate_v_first(&kin_cuda)?;
    let product_rotate_base = probe_entry(
        &product_probes[0].1,
        &product_options(false),
        "probe_rotate",
    )?;
    let upstream_rotate_base = probe_entry(
        &upstream_probes[0].1,
        &with_include(&product_options(false), &warp_include),
        "probe_rotate",
    )?;
    let product_v_first_entry = probe_entry(
        &product_v_first_source,
        &product_options(false),
        "probe_rotate_v_first",
    )?;
    let upstream_v_first_entry = probe_entry(
        &upstream_v_first_source,
        &with_include(&product_options(false), &warp_include),
        "probe_rotate_v_first",
    )?;
    let candidate_base_entry = probe_entry(
        &candidate_source,
        &product_options(false),
        "probe_rotate_candidate",
    )?;
    let candidate_v_first_entry = probe_entry(
        &candidate_v_first_source,
        &product_options(false),
        "probe_rotate_candidate",
    )?;
    let context_row = json!({
        "purpose": "同一表达式源码仅改变包装内核里的加载顺序，观察 NVRTC 收缩选择是否变化；用于解释为何单纯改写表达式源形态不能复现内核收缩。",
        "rotate": {
            "product_q_first": entry_sequence_json(&product_rotate_base),
            "product_v_first": entry_sequence_json(&product_v_first_entry),
            "upstream_q_first": entry_sequence_json(&upstream_rotate_base),
            "upstream_v_first": entry_sequence_json(&upstream_v_first_entry),
            "candidate_q_first": entry_sequence_json(&candidate_base_entry),
            "candidate_v_first": entry_sequence_json(&candidate_v_first_entry),
            "product_wrapper_effect": comparison_json(&diff::compare(&product_rotate_base, &product_v_first_entry)),
            "upstream_wrapper_effect": comparison_json(&diff::compare(&upstream_rotate_base, &upstream_v_first_entry)),
            "candidate_wrapper_effect": comparison_json(&diff::compare(&candidate_base_entry, &candidate_v_first_entry)),
            "product_v_first_occurrences_in_product_kernel": diff::occurrences(&product_kin_entry.kinds(), &product_v_first_entry.kinds()),
            "upstream_v_first_occurrences_in_frozen_kernel": diff::occurrences(&frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?.kinds(), &upstream_v_first_entry.kinds()),
            "upstream_v_first_occurrences_in_recompiled_kernel": diff::occurrences(&upstream_kin_entry.kinds(), &upstream_v_first_entry.kinds()),
        }
    });

    // ---------- com/cdof ----------
    let mut com_cdof_rows = Vec::new();
    com_cdof_rows.push(json!({
        "side": "product",
        "kernel": PRODUCT_COM_ENTRY,
        "note": "单内核顺序循环（subtree_com 初始化/反向累加/除法 + 空间惯量 + cdof）",
        "counts": counts_json(&counts(product_com_entry)),
        "atomic_add_instructions": atomic_adds(product_com_entry),
        "kind_sequence": kind_sequence(&product_com_entry.kinds()),
    }));
    for name in UPSTREAM_COM_KERNELS {
        let Some(entry) = frozen_program.entry(name) else {
            com_cdof_rows.push(json!({"side": "upstream", "kernel": name, "missing": true}));
            continue;
        };
        let note = match name {
            "_subtree_com_init_002b02fd_cuda_kernel_forward" => "上游独立内核：subtree_com 初始化",
            "_subtree_com_acc_350f0560_cuda_kernel_forward" => "上游独立内核：按层累加 subtree_com",
            "_subtree_div_0ef57c26_cuda_kernel_forward" => "上游独立内核：按质量归一",
            "_cdof_3e73eb78_cuda_kernel_forward" => "上游独立内核：cdof（xmat 列 + cross）",
            _ => "",
        };
        com_cdof_rows.push(json!({
            "side": "upstream",
            "kernel": name,
            "note": note,
            "counts": counts_json(&counts(entry)),
            "atomic_add_instructions": atomic_adds(entry),
            "kind_sequence": kind_sequence(&entry.kinds()),
        }));
    }

    // ---------- 汇总输出 ----------
    let inputs = json!({
        "repo": display_relative(&repo, &repo),
        "product_kinematics_source": {"path": "src/physics/mod.rs", "sha256": sha256_hex(kin_cuda.as_bytes())},
        "product_com_source": {"path": "src/physics/mod.rs", "sha256": sha256_hex(com_cuda.as_bytes())},
        "field_batch_source": {"path": "src/physics/resident/pack_batches.rs", "sha256": sha256_hex(field_batch.as_bytes())},
        "upstream_generated_cuda": {"path": display_relative(&smooth_cu, &repo), "sha256": sha256_hex(smooth_text.as_bytes())},
        "upstream_frozen_ptx": {"path": display_relative(&frozen_ptx, &repo), "sha256": sha256_hex(frozen_text.as_bytes()), "compiler_line": compiler_line(&frozen_text)},
        "warp_include_dir": display_relative(&warp_include, &repo),
        "nvrtc_dll": dll_identity(&cuda_bin),
    });
    let compiles = json!({
        "product_kinematics": compile_json(&product_kin, &product_options(false)),
        "product_kinematics_lineinfo": compile_json(&product_kin_li, &product_options(true)),
        "product_com": compile_json(&product_com, &product_options(false)),
        "upstream_recompiled": compile_json(&upstream, &warp_options(&warp_include, false)),
        "upstream_recompiled_lineinfo": compile_json(&upstream_li, &warp_options(&warp_include, true)),
    });
    let validation = json!({
        "lineinfo_instruction_invariant": lineinfo_invariant,
        "upstream_recompile_vs_frozen": {
            "entries_compared": frozen_compared,
            "entries_with_identical_instruction_stream": frozen_equal,
            "mismatched_entries": frozen_mismatched,
        },
        "upstream_lineinfo_recompile_vs_frozen": {
            "entries_compared": li_frozen_compared,
            "entries_with_identical_instruction_stream": li_frozen_equal,
        },
    });
    let summary_row = json!({
        "expressions": EXPRESSIONS.len(),
        "diff_types": expression_rows
            .iter()
            .map(|row| row["source_form_diff"]["diff_type"].clone())
            .collect::<Vec<_>>(),
        "source_form_changed": expression_rows
            .iter()
            .filter(|row| row["source_form_diff"]["diff_type"] != json!("一致"))
            .count(),
        "in_situ_matched_expressions": in_situ_rows
            .iter()
            .filter(|row| row["product_probe_occurrences"].as_array().map(|v| !v.is_empty()).unwrap_or(false))
            .count(),
        "rotate_candidate_matches_upstream": candidate_row["candidate_matches_upstream_source_form"].clone(),
    });
    let kinematics_kernels = json!({
        "purpose": "两端运动学内核的 f32 算术总量（产品单内核含 xmat/xipos；上游拆成 branch/matrices/inertial 多内核），仅作规模参照，不做逐项对齐。",
        "product_rigid_kinematics": {
            "counts": counts_json(&counts(product_kin_entry)),
            "arith_instructions": product_kin_entry.arith().len(),
        },
        "upstream_frozen_kinematics_branch": {
            "counts": counts_json(&counts(frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?)),
            "arith_instructions": frozen_entry(&frozen_program, UPSTREAM_KIN_ENTRY)?.arith().len(),
        },
        "upstream_recompiled_kinematics_branch": {
            "counts": counts_json(&counts(upstream_kin_entry)),
            "arith_instructions": upstream_kin_entry.arith().len(),
        },
    });
    let report = json!({
        "schema": 1,
        "task_id": "2026-10-23-u038-3d-ptx-diff-tool",
        "tool": "u038-ptx-diff-tool 0.1.0",
        "scope": "read-only PTX comparison; no product source changes; compile only (no kernel launch)",
        "inputs": inputs,
        "compiles": compiles,
        "validation": validation,
        "kinematics_kernels": kinematics_kernels,
        "expressions": expression_rows,
        "in_situ": in_situ_rows,
        "candidate_transcriptions": candidate_row,
        "context_sensitivity": context_row,
        "com_cdof": com_cdof_rows,
        "summary": summary_row,
    });
    report::write_json(&out.join("ptx-diff.json"), &report)?;
    report::print_summary(&report);
    println!("输出目录：{}", out.display());
    Ok(())
}
