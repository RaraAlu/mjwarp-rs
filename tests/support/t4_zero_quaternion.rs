//! Rust直接执行冻结Warp产物；不启动Python。
mod face_polar {
    use super::*;
    use serde_json::{Value, json};

    const ENTRY: &str = "_flex_face_kinematics_9a1620ae_cuda_kernel_forward";
    const FACE: &str = include_str!("../../fixtures/flex-face/face-state.json");
    const BASE: &str = include_str!("../../fixtures/flex-position/flex-tree.json");
    const NF: usize = 16;
    const NN: usize = 20;
    const WIDTH: usize = 31 * NF;
    const GUARD: f32 = -131072.0;

    fn config(items: usize) -> LaunchConfig {
        // 冻结Warp共享槽仅容纳256线程；禁用cudarc的1024默认值。
        LaunchConfig {
            grid_dim: ((items as u32).div_ceil(256), 1, 1),
            block_dim: (256, 1, 1),
            shared_mem_bytes: 0,
        }
    }

    fn ints(v: &Value) -> Vec<i32> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|v| i32::try_from(v.as_i64().unwrap()).unwrap())
            .collect()
    }

    // 输出按产品布局分世界；描述符允许字段间保留步长。
    fn output_array(buffer: &CudaSlice<f32>, stream: &CudaStream, nw: usize, quat: bool) -> Array {
        let mut result = array(
            buffer,
            stream,
            4 + if quat { 27 * NF } else { 0 },
            &[nw, NF],
            if quat { 4 } else { 27 },
        );
        result.strides[0] = (WIDTH * 4) as i32;
        if !quat {
            result.ndim = 3;
            result.shape[2] = 9;
            result.strides[2] = 12;
        }
        result
    }

    #[test]
    #[ignore = "requires frozen Warp CU/PTX, NVIDIA sm89+ and NVRTC; no Python"]
    fn compares_face_polar_with_actual_upstream_artifact() {
        let started = Instant::now();
        assert_eq!(size_of::<Array>(), 56);
        assert_eq!(size_of::<Bounds>(), 24);
        assert_eq!(
            hash(FACE.replace("\r\n", "\n").as_bytes()),
            "b029ed9989e2371b76d1921cebe70c1897dfb05c446d10a66debb3604cb11b49"
        );
        assert_eq!(
            hash(BASE.replace("\r\n", "\n").as_bytes()),
            "7fd347cb9dc2ee46a0f5097ae0f6af75f96b6cc408353e7db026c47ba9e91365"
        );
        let face: Value = serde_json::from_str(FACE).unwrap();
        let base: Value = serde_json::from_str(BASE).unwrap();
        let cases = face["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 12);
        assert_eq!(face["nflexface"], NF);
        let interp = ints(&face["model"]["flex_interp"]);
        let cells = ints(&base["model"]["flex_cellnum"]);
        let mapping = ints(&face["model"]["flex_face_map"]);
        let faces = ints(&face["model"]["flex_face"]);
        assert_eq!(
            (interp.len(), cells.len(), mapping.len(), faces.len()),
            (4, 12, 32, 144)
        );
        let mut metadata = vec![NF as i32, (3 * NN) as i32];
        let mut axes = [0; 3];
        for f in 0..NF {
            let id = mapping[2 * f] as usize;
            assert_eq!(interp[id], -1);
            let [x, y, z] = cells[3 * id..3 * id + 3].try_into().unwrap();
            let local = mapping[2 * f + 1];
            assert!((0..2 * (y * z + x * z + x * y)).contains(&local));
            let axis = if local < 2 * y * z {
                0
            } else if local < 2 * (y * z + x * z) {
                1
            } else {
                2
            };
            axes[axis] += 1;
            metadata.push(axis as i32);
            let row = &faces[9 * f..9 * f + 9];
            assert!(row[..4].iter().all(|&n| (0..NN as i32).contains(&n)));
            assert_eq!(&row[4..], &[-1; 5]);
            metadata.extend_from_slice(row);
        }
        assert_eq!(axes, [4, 6, 6]);
        let cu = artifact("cu", CU_HASH);
        assert!(cu.contains(ENTRY));
        let raw_ptx = artifact("sm89.ptx", PTX_HASH);
        let upstream_ptx = raw_ptx.strip_suffix('\0').unwrap();
        assert!(!upstream_ptx.contains('\0'));
        assert!(
            upstream_ptx
                .contains(".b8 _ZZN2wp21tile_shared_storage_t13get_smem_baseEvE9smem_base[1024]")
        );
        eprintln!("face polar: create context");
        let context = CudaContext::new(0).unwrap();
        let stream = context.default_stream();
        let upstream = context
            .load_module(Ptx::from_src(upstream_ptx))
            .unwrap()
            .load_function(ENTRY)
            .unwrap();
        eprintln!("face polar: upstream loaded; compile product");
        let text = include_str!("../../src/physics/flex_face.rs");
        let source =
            super::super::between(text, "const FLEX_FACE_CUDA: &str = r#\"", "\"#;").unwrap();
        let ptx = compile_ptx_with_opts(
            source,
            CompileOptions {
                fmad: Some(true),
                use_fast_math: Some(false),
                options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
                name: Some("flex_faces.cu".into()),
                ..Default::default()
            },
        )
        .unwrap();
        let product_ptx_hash = hash(ptx.to_src().as_bytes());
        let product = context
            .load_module(ptx)
            .unwrap()
            .load_function("flex_faces")
            .unwrap();
        eprintln!("face polar: product loaded; upload metadata");
        let di = stream.clone_htod(&interp).unwrap();
        let dc = stream.clone_htod(&cells).unwrap();
        let dm = stream.clone_htod(&mapping).unwrap();
        let df = stream.clone_htod(&faces).unwrap();
        let meta = stream.clone_htod(&metadata).unwrap();
        let params = stream.clone_htod(&[0.0f32]).unwrap();
        let mut batches = Vec::new();
        let mut total = 0;
        let mut violations = 0;
        let mut samples = Vec::new();
        for nw in [1usize, 2, 5, 513] {
            for round in 0..4 {
                eprintln!("face polar: worlds={nw} round={round} upload");
                let mut nodes = vec![GUARD; 4];
                for w in 0..nw {
                    let values = cases[(w + round * 3) % 12]["flexnode_xpos"]
                        .as_array()
                        .unwrap();
                    assert_eq!(values.len(), 3 * NN);
                    nodes.extend(values.iter().map(|v| v.as_f64().unwrap() as f32));
                }
                nodes.extend([GUARD; 4]);
                let dn = stream.clone_htod(&nodes).unwrap();
                let initial = vec![GUARD; 8 + nw * WIDTH];
                let du = stream.clone_htod(&initial).unwrap();
                let mut dp = stream.clone_htod(&initial).unwrap();
                let args = [
                    array(&di, &stream, 0, &[4], 1),
                    array(&dc, &stream, 0, &[4], 3),
                    array(&dm, &stream, 0, &[NF], 2),
                    array(&df, &stream, 0, &[NF, 9], 1),
                    array(&dn, &stream, 4, &[nw, NN], 3),
                    output_array(&du, &stream, nw, false),
                    output_array(&du, &stream, nw, true),
                ];
                let dim = bounds(nw, NF);
                stream.synchronize().unwrap();
                eprintln!("face polar: upstream launch");
                let run = Instant::now();
                let mut launch = stream.launch_builder(&upstream);
                launch.arg(&dim);
                for arg in &args {
                    launch.arg(arg);
                }
                // SAFETY: 冻结ABI与哈希固定。输入节点索引均在20节点内。
                // 各线程独占一面。输出步长隔离字段与世界，分配保留至同步。
                unsafe {
                    launch.launch(config(nw * NF)).unwrap();
                }
                stream.synchronize().unwrap();
                let upstream_us = run.elapsed().as_micros();
                eprintln!("face polar: product launch");
                let run = Instant::now();
                // SAFETY: 产品模板使用162项元数据及60项节点步长。
                // 每世界独占496项输出，前后各有四项哨兵。
                unsafe {
                    stream
                        .launch_builder(&product)
                        .arg(&meta)
                        .arg(&params)
                        .arg(&dn)
                        .arg(&mut dp)
                        .arg(&0u32)
                        .arg(&0u32)
                        .arg(&0u32)
                        .arg(&(nw as u32))
                        .launch(config(nw))
                        .unwrap();
                }
                stream.synchronize().unwrap();
                let product_us = run.elapsed().as_micros();
                let expected = stream.clone_dtoh(&du).unwrap();
                let actual = stream.clone_dtoh(&dp).unwrap();
                for output in [&actual, &expected] {
                    assert_eq!(&output[..4], &[GUARD; 4]);
                    assert_eq!(&output[4 + nw * WIDTH..], &[GUARD; 4]);
                    assert!(
                        output[4..4 + nw * WIDTH]
                            .iter()
                            .all(|v| v.is_finite() && *v != GUARD)
                    );
                }
                assert_eq!(stream.clone_dtoh(&dn).unwrap(), nodes);
                let mut changed = [0usize; 2];
                let mut max_error = [0.0f64; 2];
                let mut over = [0usize; 2];
                for w in 0..nw {
                    for k in 0..WIDTH {
                        let a = actual[4 + w * WIDTH + k];
                        let e = expected[4 + w * WIDTH + k];
                        let field = usize::from(k >= 27 * NF);
                        let err = (f64::from(a) - f64::from(e)).abs();
                        max_error[field] = max_error[field].max(err);
                        changed[field] += usize::from(a.to_bits() != e.to_bits());
                        // 沿用原面测试容差；诊断保留原始四元数符号。
                        over[field] += usize::from(err > 2e-5 + 2e-5 * f64::from(e).abs());
                        if a.to_bits() != e.to_bits() && samples.len() < 8 {
                            samples.push(json!({"worlds":nw,"round":round,"world":w,"element":k,"product":a,"upstream":e}));
                        }
                    }
                }
                violations += over.iter().sum::<usize>();
                total += nw * WIDTH;
                batches.push(json!({"worlds":nw,"round":round,"comparisons":nw*WIDTH,"changed_bits":changed,
                    "max_abs_error":max_error,"outside_original_tolerance":over,"upstream_wall_us":upstream_us,
                    "product_wall_us":product_us,"owned_device_bytes":4*(interp.len()+cells.len()+mapping.len()+faces.len()+metadata.len()+1+nodes.len()+2*initial.len())}));
            }
        }
        assert_eq!(total, 2084 * WIDTH);
        let report = json!({"schema":1,"scope":"face-polar-generated-artifact","upstream_revision":super::super::REVISION,
            "upstream_cuda_sha256":CU_HASH,"upstream_ptx_sha256":PTX_HASH,"product_cuda_sha256":hash(source.as_bytes()),
            "product_ptx_sha256":product_ptx_hash,"world_counts":[1,2,5,513],"rounds":4,"cases":12,"faces":NF,
            "block_threads":256,"upstream_static_thread_slots":256,"product_fmad":true,"product_fast_math":false,
            "face_fixture_sha256":hash(FACE.replace("\r\n", "\n").as_bytes()),"node_fixture_sha256":hash(BASE.replace("\r\n", "\n").as_bytes()),
            "comparisons":total,"fields":["face_xpos","face_quat"],"absolute_tolerance":2e-5,"relative_tolerance":2e-5,
            "batches":batches,"first_bit_differences":samples,"outside_original_tolerance":violations,
            "elapsed_ms":started.elapsed().as_millis(),"python_executions":0,"np01_complete":false,"np02_complete":false,
            "np03":"historical-unchanged","np04":"unequal-compilers-and-targets-not-performance-acceptance"});
        println!("{report}");
        if let Some(path) = std::env::var_os("MJWARP_FACE_POLAR_REPORT") {
            std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
        }
        assert_eq!(
            violations, 0,
            "face product differs outside original tolerance"
        );
    }
}

use cudarc::{
    driver::{
        CudaContext, CudaSlice, CudaStream, DevicePtr, DeviceRepr, LaunchConfig, PushKernelArg,
    },
    nvrtc::{CompileOptions, Ptx, compile_ptx_with_opts},
};
use sha2::{Digest, Sha256};
use std::{path::PathBuf, time::Instant};

const CACHE: &str = "target/mjwarp-parallel/warp-cache/1.15.0/wp_mujoco_warp._src.smooth_ca013d6/wp_mujoco_warp._src.smooth_ca013d6";
const CU_HASH: &str = "3a6b3856727f63797a28646a5864aa02cd76a20ed3658199c69b3ad75c802bfa";
const PTX_HASH: &str = "1d9e4e3a983ca35090ade3a48fd83aa0dc86a23228bb0865010f681698ab75dd";
const ENTRIES: [&str; 3] = [
    "_kinematics_branch_95f8028e_cuda_kernel_forward",
    "_compute_body_matrices_27760750_cuda_kernel_forward",
    "_compute_body_inertial_frames_d3bdd81a_cuda_kernel_forward",
];
const NB: usize = 5;
const NJ: usize = 2;
const NQ: usize = 11;
const NW: usize = 4;
const STRIDE: usize = 28 * NB + 6 * NJ;

// Warp 1.15: array.h与builtin.h定义按值传参布局。
// 显式尾部填充避免传递未初始化字节。
#[repr(C)]
#[derive(Clone, Copy)]
struct Array {
    data: u64,
    grad: u64,
    shape: [i32; 4],
    strides: [i32; 4],
    ndim: u16,
    flags: u16,
    padding: u32,
}
#[repr(C)]
struct Bounds {
    shape: [i32; 2],
    size: u64,
    coord_mult: u64,
}
// SAFETY: repr(C)、固定宽度、无引用，布局断言绑定冻结PTX参数。
unsafe impl DeviceRepr for Array {}
// SAFETY: repr(C)与launch_bounds_t<2>逐字段一致。
unsafe impl DeviceRepr for Bounds {}

fn array<T>(
    buffer: &CudaSlice<T>,
    stream: &CudaStream,
    offset: usize,
    shape: &[usize],
    width: usize,
) -> Array {
    assert!((1..=2).contains(&shape.len()));
    let bytes = width * 4;
    let (ptr, _record) = buffer.device_ptr(stream);
    Array {
        data: ptr + (offset * 4) as u64,
        grad: 0,
        shape: [shape[0] as i32, *shape.get(1).unwrap_or(&0) as i32, 0, 0],
        strides: [
            if shape.len() == 2 {
                shape[1] * bytes
            } else {
                bytes
            } as i32,
            if shape.len() == 2 { bytes } else { 0 } as i32,
            0,
            0,
        ],
        ndim: shape.len() as u16,
        flags: 0,
        padding: 0,
    }
}
fn hash(s: &[u8]) -> String {
    format!("{:x}", Sha256::digest(s))
}
fn artifact(extension: &str, expected: &str) -> String {
    let bytes = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("{CACHE}.{extension}")),
    )
    .unwrap();
    assert_eq!(hash(&bytes), expected, "frozen artifact drift");
    String::from_utf8(bytes).unwrap()
}
fn bounds(worlds: usize, items: usize) -> Bounds {
    Bounds {
        shape: [worlds as i32, items as i32],
        size: (worlds * items) as u64,
        coord_mult: 1,
    }
}

#[test]
fn frozen_warp_argument_layout_matches_generated_ptx() {
    assert_eq!(size_of::<Array>(), 56);
    assert_eq!(align_of::<Array>(), 8);
    assert_eq!(std::mem::offset_of!(Array, shape), 16);
    assert_eq!(std::mem::offset_of!(Array, strides), 32);
    assert_eq!(std::mem::offset_of!(Array, ndim), 48);
    assert_eq!(size_of::<Bounds>(), 24);
    assert_eq!(std::mem::offset_of!(Bounds, size), 8);
    assert_eq!(std::mem::offset_of!(Bounds, coord_mult), 16);
}

#[test]
#[ignore = "requires frozen Warp CU/PTX, NVIDIA sm89+ and NVRTC; no Python"]
fn compares_zero_quaternion_with_actual_upstream_artifact() {
    let started = Instant::now();
    let cu = artifact("cu", CU_HASH);
    let upstream_ptx = artifact("sm89.ptx", PTX_HASH);
    for entry in ENTRIES {
        assert!(cu.contains(entry));
        assert!(upstream_ptx.contains(entry));
    }
    let branch = super::between(&cu, &format!("void {}(", ENTRIES[0]), "extern \"C\"").unwrap();
    for call in [
        "var_65 = wp::normalize(var_60);",
        "var_165 = wp::normalize(var_160);",
        "var_215 = wp::normalize(var_124);",
    ] {
        assert!(branch.contains(call));
    }
    assert!(branch.contains("wp::quat_t<wp::float32> var_60;"));
    let context = CudaContext::new(0).unwrap();
    let stream = context.default_stream();
    // Warp缓存附带一个C字符串终止符；cudarc自行添加该符号。
    // 哈希覆盖原始字节，只移除尾部一个NUL，不改内核指令。
    let upstream_ptx = upstream_ptx.strip_suffix('\0').unwrap();
    assert!(!upstream_ptx.contains('\0'));
    let upstream = context.load_module(Ptx::from_src(upstream_ptx)).unwrap();

    // 三条独立分支：自由关节及子体、球关节、mocap。
    let parent = [0, 0, 0, 0, 1];
    let adr = [-1, 0, 1, -1, -1];
    let num = [0, 1, 1, 0, 0];
    let types = [0, 1];
    let qa = [0, 7];
    let mocap = [-1, -1, -1, 0, -1];
    let mut ints = Vec::new();
    let mut io = Vec::new();
    for field in [
        &parent[..],
        &mocap,
        &num,
        &adr,
        &types,
        &qa,
        &[1, 4, 2, 3],
        &[0, 2, 3, 4],
    ] {
        io.push(ints.len());
        ints.extend_from_slice(field);
    }
    let identity = [1.0f32, 0.0, 0.0, 0.0];
    let mut bp = vec![0.0; 3 * NB];
    bp[12..15].copy_from_slice(&[1.0, 2.0, 3.0]);
    let bq = identity.repeat(NB);
    let ip = vec![0.25; 3 * NB];
    let iq = identity.repeat(NB);
    let jp = vec![0.25; 3 * NJ];
    let axis = [0.0, 0.0, 1.0].repeat(NJ);
    let mut model = Vec::new();
    let mut offsets = Vec::new();
    for field in [&vec![0.0; NQ], &bp, &bq, &ip, &iq, &jp, &axis] {
        offsets.push(model.len());
        model.extend_from_slice(field);
    }
    let quats = [[0.0; 4], [-0.0; 4], [1.0e-30; 4], identity];
    let mut state = Vec::new();
    for q in quats {
        state.extend_from_slice(&[1.0, 2.0, 3.0]);
        state.extend_from_slice(&q);
        state.extend_from_slice(&q);
    }
    state.extend([2.0, 3.0, 4.0].repeat(NW));
    for q in quats {
        state.extend(q);
    }
    let mut initial = vec![0.0; STRIDE * NW + 8];
    initial[..4].fill(-131072.0);
    initial[4 + STRIDE * NW..].fill(-131072.0);
    for w in 0..NW {
        initial[4 + w * STRIDE + 3 * NB..4 + w * STRIDE + 3 * NB + 4].copy_from_slice(&identity);
    }
    // 上游独立连续字段；产品采用逐世界打包输出。
    let widths = [3 * NB, 4 * NB, 9 * NB, 3 * NB, 9 * NB, 3 * NJ, 3 * NJ];
    let mut out_offsets = Vec::new();
    let mut cursor = 4;
    for width in widths {
        out_offsets.push(cursor);
        cursor += NW * width;
    }
    let mut upstream_initial = vec![0.0f32; cursor + 4];
    upstream_initial[..4].fill(-131072.0);
    upstream_initial[cursor..].fill(-131072.0);
    for w in 0..NW {
        upstream_initial[out_offsets[1] + w * 4 * NB..out_offsets[1] + w * 4 * NB + 4]
            .copy_from_slice(&identity);
    }
    let di = stream.clone_htod(&ints).unwrap();
    let dm = stream.clone_htod(&model).unwrap();
    let ds = stream.clone_htod(&state).unwrap();
    let dout = stream.clone_htod(&upstream_initial).unwrap();
    let m = |field, count, width| array(&dm, &stream, offsets[field], &[1, count], width);
    let i = |field, count| array(&di, &stream, io[field], &[count], 1);
    let s = |offset, count, width| array(&ds, &stream, offset, &[NW, count], width);
    let o =
        |field: usize, count, width| array(&dout, &stream, out_offsets[field], &[NW, count], width);
    let arrays = [
        m(0, NQ, 1),
        i(0, NB),
        i(1, NB),
        i(2, NB),
        i(3, NB),
        m(1, NB, 3),
        m(2, NB, 4),
        i(4, NJ),
        i(5, NJ),
        m(5, NJ, 3),
        m(6, NJ, 3),
        i(6, 4),
        i(7, 4),
        s(0, NQ, 1),
        s(NW * NQ, 1, 3),
        s(NW * (NQ + 3), 1, 4),
        o(0, NB, 3),
        o(1, NB, 4),
        o(5, NJ, 3),
        o(6, NJ, 3),
    ];
    let config = LaunchConfig {
        grid_dim: (1, 1, 1),
        block_dim: (256, 1, 1),
        shared_mem_bytes: 0,
    };
    stream.synchronize().unwrap();
    for (entry, dim, args) in [
        (ENTRIES[0], bounds(NW, 3), arrays.to_vec()),
        (ENTRIES[1], bounds(NW, NB), vec![o(1, NB, 4), o(2, NB, 9)]),
        (
            ENTRIES[2],
            bounds(NW, NB),
            vec![
                m(3, NB, 3),
                m(4, NB, 4),
                o(0, NB, 3),
                o(1, NB, 4),
                o(3, NB, 3),
                o(4, NB, 9),
            ],
        ),
    ] {
        let function = upstream.load_function(entry).unwrap();
        let mut launch = stream.launch_builder(&function);
        launch.arg(&dim);
        for arg in &args {
            launch.arg(arg);
        }
        // SAFETY: Hash-pinned Warp ABI; shapes/strides match owned allocations.
        // Branches own disjoint bodies/joints; all pointers stay alive until sync.
        // Raw output pointers require explicit synchronization before readback.
        unsafe {
            launch.launch(config).unwrap();
        }
        stream.synchronize().unwrap();
    }
    let expected = stream.clone_dtoh(&dout).unwrap();
    assert_eq!(&expected[..4], &[-131072.0; 4]);
    assert_eq!(&expected[cursor..], &[-131072.0; 4]);
    // 数值比较允许正负零；回退槽位必须精确，不使用新容差。
    for w in 0..NW {
        for b in 1..NB {
            let q = out_offsets[1] + w * NB * 4 + b * 4;
            assert_eq!(
                &expected[q..q + 4],
                if w == 3 {
                    &identity
                } else {
                    &[0.0, 0.0, 0.0, 1.0]
                }
            );
        }
    }

    let physics = include_str!("../../src/physics/mod.rs");
    let resident = include_str!("../../src/physics/resident.rs");
    let kernel = super::between(physics, "const KINEMATICS_CUDA: &str = r#\"", "\"#;").unwrap();
    let prefix = super::between(resident, "const FIELD_BATCH_CUDA: &str = r#\"", "\"#;").unwrap();
    let source = format!("{prefix}\n#define MJWARP_MOCAP\n#define MJWARP_G01_EQUIVALENT\n{kernel}");
    let ptx = compile_ptx_with_opts(
        &source,
        CompileOptions {
            fmad: Some(true),
            use_fast_math: Some(false),
            options: vec!["--std=c++17".into(), "--gpu-architecture=compute_70".into()],
            name: Some("rigid_kinematics.cu".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let product_ptx_hash = hash(ptx.to_src().as_bytes());
    let product = context
        .load_module(ptx)
        .unwrap()
        .load_function("rigid_kinematics")
        .unwrap();
    let mut meta = Vec::new();
    for field in [&parent[..], &adr, &num, &types, &qa, &[1], &mocap] {
        meta.extend_from_slice(field);
    }
    for offset in offsets {
        meta.extend([offset as i32, 1]);
    }
    let metadata = stream.clone_htod(&meta).unwrap();
    let mut result = stream.clone_htod(&initial).unwrap();
    // SAFETY: Actual resident template and fixed ABI; metadata covers 5 bodies,
    // 2 joints, 11 qpos and 1 mocap. Each world owns STRIDE floats plus guards.
    unsafe {
        stream
            .launch_builder(&product)
            .arg(&metadata)
            .arg(&dm)
            .arg(&ds)
            .arg(&mut result)
            .arg(&(NQ as u32))
            .arg(&(NB as u32))
            .arg(&(NJ as u32))
            .arg(&(NW as u32))
            .launch(config)
            .unwrap();
    }
    let actual = stream.clone_dtoh(&result).unwrap();
    assert_eq!(&actual[..4], &[-131072.0; 4]);
    assert_eq!(&actual[4 + NW * STRIDE..], &[-131072.0; 4]);
    let mut differences = Vec::new();
    for w in 0..NW {
        let mut offset = 0;
        for (field, width) in widths.into_iter().enumerate() {
            for element in 0..width {
                let a = actual[4 + w * STRIDE + offset + element];
                let e = expected[out_offsets[field] + w * width + element];
                if a != e {
                    differences.push(serde_json::json!({"world":w,"field":field,"element":element,"product":a,"upstream":e}));
                }
            }
            offset += width;
        }
    }
    let report = serde_json::json!({"schema":1,"scope":"zero-quaternion-generated-artifact","upstream_revision":super::REVISION,"upstream_cuda_sha256":CU_HASH,"upstream_ptx_sha256":PTX_HASH,"product_cuda_sha256":hash(source.as_bytes()),"product_ptx_sha256":product_ptx_hash,"worlds":NW,"bodies":NB,"joints":NJ,"fields":7,"comparisons":NW*STRIDE,"zero_fallback":[0,0,0,1],"differences":differences,"elapsed_ms":started.elapsed().as_millis(),"owned_device_bytes":4*(ints.len()+model.len()+state.len()+upstream_initial.len()+meta.len()+initial.len()),"python_executions":0,"np01_complete":false,"np02_complete":false,"np03":"historical-unchanged","np04":"not-performance-acceptance"});
    println!("{report}");
    if let Some(path) = std::env::var_os("MJWARP_ZERO_QUATERNION_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
    assert!(
        differences.is_empty(),
        "product differs from the actual frozen artifact"
    );
}
