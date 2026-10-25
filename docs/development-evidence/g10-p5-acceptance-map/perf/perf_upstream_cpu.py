"""G10@P5 G6-2：上游 f32 CPU 同机同版本接触计算性能参考（authoring-only）。

性质：authoring_only；不进产品路径、产品测试不跑 Python；只用 warp CPU 后端，不占 GPU。
输入：既有 G2/G3/G4/G5 捕获脚本的同一 MJCF 模型与参考 JSON 的 qpos（f32 位型原样）。
输出：docs/development-evidence/g10-p5-acceptance-map/perf/upstream-cpu.json（+ 同名 .log）。

计时口径：
- 只计 `mujoco_warp.collision`（broadphase + narrowphase + contact 写入）的单次调用；
- 每次调用后 `wp.synchronize_device()`；warp CPU launch 本身是串行 for 循环，脚本另把
  OMP/MKL/OPENBLAS/NUMEXPR 线程数固定为 1；
- 模型加载（mujoco.MjModel）、`mjw.put_model`、`make_data`、`kinematics` 均不计入 samples
  （前两者在 JSON 单列；后两者只做一次准备），首次 collision 的 JIT 编译落在 warmup 内；
- 同进程串行跑全部 case；每个 rep 读回 `d.nacon` 校验接触数，计时前/后各做一次 17 字段
  逐位快照，防止空转或结果漂移。

用法（仓库根目录）：
    target/mjwarp-parallel/venv/Scripts/python.exe scripts/authoring/g10-p5/perf_upstream_cpu.py
    ... --warmup 20 --reps 200 --cases cylinder-box,sdf-box --out <path> --log <path>
"""

import argparse
import datetime
import gc
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
import tempfile
import time
import traceback
from pathlib import Path

# 单线程口径：必须在 numpy / warp 导入前固定底层线程数。
for _name in ("OMP_NUM_THREADS", "MKL_NUM_THREADS", "OPENBLAS_NUM_THREADS", "NUMEXPR_NUM_THREADS"):
    os.environ[_name] = "1"

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

ROOT = Path(__file__).resolve().parents[3]
OUT_DIR = ROOT / "docs/development-evidence/g10-p5-acceptance-map/perf"
DEFAULT_OUT = OUT_DIR / "upstream-cpu.json"
DEFAULT_LOG = OUT_DIR / "upstream-cpu.log"

UPSTREAM = "71da24d956378a87a703b6e1442b13aec0c4ac29"
UPSTREAM_SOURCE = ROOT / "target/mjwarp-parallel/upstream"
WARMUP = 20
REPS = 200
NCONMAX = 4096
NJMAX = 20000

# 计时用的既有 CPU 内核缓存可复用于本机同一 warp 版本；只在 %TEMP% 合并，不写仓库。
CACHE_SOURCES = (
    "target/g10-p5-g2/warp-cache-cpu",
    "target/g10-p5-g3/warp-cache-cpu",
    "target/g10-p5-g4/warp-cache-cpu",
    "target/g10-p5-g5/warp-cache-cpu",
)

# (case, reference json, collision_cases 内的 case 名；standalone 参考为 None)
CASES = (
    ("cylinder-box", "fixtures/g10-p5/references/cylinder-box.json", None),
    ("box-box-margin", "fixtures/g10-p5/references/box-box-margin.json", None),
    ("mesh-mesh", "fixtures/g10-p5/references/mesh-mesh.json", None),
    ("convex-mesh", "fixtures/mjwarp-complex-physics/reference.json", "convex-mesh"),
    ("degenerate-mesh", "fixtures/mjwarp-complex-physics/reference.json", "degenerate-mesh"),
    ("sdf-box", "fixtures/g10-p5/references/sdf-box.json", None),
    ("hfield-box", "fixtures/mjwarp-complex-physics/reference.json", "hfield-box"),
    ("hfield-sphere", "fixtures/g10-p5/references/hfield-sphere.json", None),
    ("hfield-capsule", "fixtures/g10-p5/references/hfield-capsule.json", None),
)

FLOAT_FIELDS = (
    "dist", "pos", "frame", "includemargin", "friction", "solref",
    "solreffriction", "solimp", "adhesion",
)
INT_FIELDS = ("dim", "geom", "flex", "elem", "vert", "worldid", "type", "geomcollisionid")


class CaseError(RuntimeError):
    """单个 case 的配方/结果校验失败。"""


class _Tee:
    """把 stdout/stderr 同时写入控制台与 .log，保留 warp 模块加载/警告原文。"""

    def __init__(self, stream, handle):
        self.stream = stream
        self.handle = handle

    def write(self, text):
        self.stream.write(text)
        self.handle.write(text)
        return len(text)

    def flush(self):
        self.stream.flush()
        self.handle.flush()

    def isatty(self):
        return False


def sha256_file(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def cpu_name():
    if sys.platform == "win32":
        try:
            import winreg

            key = winreg.OpenKey(
                winreg.HKEY_LOCAL_MACHINE,
                r"HARDWARE\DESCRIPTION\System\CentralProcessor\0",
            )
            value = winreg.QueryValueEx(key, "ProcessorNameString")[0].strip()
            if value:
                return value
        except OSError:
            pass
    return platform.processor() or "unknown"


def git_head(path):
    try:
        result = subprocess.run(
            ["git", "-C", str(path), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except (OSError, subprocess.SubprocessError):
        pass
    return None


def prepare_kernel_cache():
    """把已有 CPU 内核缓存合并到 %TEMP%，避免 JIT 编译时间污染作者机等待（不进计时）。"""
    base = Path(tempfile.gettempdir()) / "g10p5-upstream-cpu-warp-cache"
    version_dir = base / wp.__version__
    version_dir.mkdir(parents=True, exist_ok=True)
    seeded = 0
    for source_rel in CACHE_SOURCES:
        source = ROOT / source_rel / wp.__version__
        if not source.is_dir():
            continue
        for module in sorted(source.iterdir()):
            target = version_dir / module.name
            if module.is_dir() and not target.exists():
                try:
                    shutil.copytree(module, target)
                    seeded += 1
                except OSError as error:
                    print(f"warning: cache seed skipped {module.name}: {error}", flush=True)
    wp.config.kernel_cache_dir = str(base)
    return base, seeded


def load_case(spec):
    name, reference_rel, reference_case = spec
    doc = json.loads((ROOT / reference_rel).read_text(encoding="utf-8"))
    if reference_case is not None:
        doc = next(case for case in doc["collision_cases"] if case["name"] == reference_case)
    data = doc["input"]
    return {
        "case": name,
        "xml": data["path"],
        "reference": reference_rel,
        "reference_case": reference_case,
        "worlds": int(data["worlds"]),
        "qpos": np.asarray(data["qpos"]["values"], dtype=np.float32),
        "reference_count": int(doc["output"]["count"]),
    }


def snapshot(data, count):
    """接触数组前 count 行逐位快照（float 按 uint32 位型，整数原样）。"""
    result = {}
    for name in FLOAT_FIELDS + INT_FIELDS:
        array = np.ascontiguousarray(getattr(data.contact, name).numpy()[:count])
        result[name] = array.view(np.uint32).copy() if name in FLOAT_FIELDS else array.copy()
    return result


def snapshots_equal(left, right):
    if left.keys() != right.keys():
        return False
    for name in left:
        if left[name].shape != right[name].shape or not np.array_equal(left[name], right[name]):
            return False
    return True


def measure_case(spec, warmup, reps):
    case = load_case(spec)
    name = case["case"]
    xml_path = ROOT / case["xml"]
    worlds = case["worlds"]
    qpos = case["qpos"]

    begin = time.perf_counter_ns()
    host = mujoco.MjModel.from_xml_path(str(xml_path))
    load_ns = time.perf_counter_ns() - begin

    begin = time.perf_counter_ns()
    model = mjw.put_model(host)
    compile_ns = time.perf_counter_ns() - begin

    if qpos.size != worlds * host.nq:
        raise CaseError(f"qpos {qpos.size} != worlds({worlds}) * nq({host.nq})")

    begin = time.perf_counter_ns()
    data = mjw.make_data(host, nworld=worlds, nconmax=NCONMAX, njmax=NJMAX)
    make_data_ns = time.perf_counter_ns() - begin

    data.qpos.assign(qpos.reshape(worlds, host.nq))
    data.qvel.assign(np.zeros((worlds, host.nv), dtype=np.float32))
    mjw.kinematics(model, data)
    wp.synchronize_device()

    count0 = None
    first_call_ns = None
    for index in range(warmup):
        start = time.perf_counter_ns()
        mjw.collision(model, data)
        wp.synchronize_device()
        elapsed = time.perf_counter_ns() - start
        if index == 0:
            first_call_ns = elapsed
            count0 = int(data.nacon.numpy()[0])
    if count0 is None:
        mjw.collision(model, data)
        wp.synchronize_device()
        count0 = int(data.nacon.numpy()[0])

    if count0 != case["reference_count"]:
        raise CaseError(f"contacts {count0} != reference {case['reference_count']}")
    if count0 <= 0:
        raise CaseError("contacts must be positive to measure real work")
    last_count = int(data.nacon.numpy()[0])
    if last_count != count0:
        raise CaseError(f"warmup contacts unstable: {last_count} != {count0}")

    baseline = snapshot(data, count0)
    count_sum = 0
    mismatch = []
    samples = []
    gc.collect()
    gc.disable()  # 按 timeit 惯例排除 Python 循环 GC 暂停；不计入接触计算
    try:
        for index in range(reps):
            start = time.perf_counter_ns()
            mjw.collision(model, data)
            wp.synchronize_device()
            elapsed = time.perf_counter_ns() - start
            samples.append(int(elapsed))
            count = int(data.nacon.numpy()[0])
            count_sum += count
            if count != count0:
                mismatch.append(index)
    finally:
        gc.enable()
    gc.collect()

    # 计时后再跑一次：逐字段位型核对，证明每个 rep 都做了同样的接触计算。
    mjw.collision(model, data)
    wp.synchronize_device()
    final_count = int(data.nacon.numpy()[0])
    stable = final_count == count0 and snapshots_equal(baseline, snapshot(data, count0))

    median_ns = round(float(np.median(samples)), 3)
    p95_ns = round(float(np.percentile(samples, 95)), 3)
    contacts_per_s = count0 / (median_ns / 1e9) if median_ns > 0 else None
    calls_per_s = 1e9 / median_ns if median_ns > 0 else None

    return {
        "case": name,
        "xml": case["xml"],
        "xml_sha256": sha256_file(xml_path),
        "reference": case["reference"],
        "reference_case": case["reference_case"] or name,
        "worlds": worlds,
        "ngeom": int(host.ngeom),
        "warmup": int(warmup),
        "reps": int(reps),
        "ncontacts": int(count0),
        "reference_count": int(case["reference_count"]),
        "count_matches_reference": True,
        "count_stable_across_reps": not mismatch,
        "count_mismatch_reps": len(mismatch),
        "count_mismatch_first_reps": mismatch[:10],
        "contact_bitwise_stable_after_timed_loop": bool(stable),
        "load_ms": round(load_ns / 1e6, 3),
        "compile_ms": round(compile_ns / 1e6, 3),
        "make_data_ms": round(make_data_ns / 1e6, 3),
        "first_warmup_call_ms": round(first_call_ns / 1e6, 3) if first_call_ns else None,
        "samples_ns": samples,
        "median_ns": median_ns,
        "p95_ns": p95_ns,
        "min_ns": int(min(samples)),
        "max_ns": int(max(samples)),
        "calls_per_s": round(calls_per_s, 3) if calls_per_s else None,
        "contacts_per_s": round(contacts_per_s, 3) if contacts_per_s else None,
        "median_contacts_per_call": count0,
        "timed_reps_contact_sum": int(count_sum),
    }


def parse_args(argv):
    parser = argparse.ArgumentParser(description="G10@P5 upstream f32 CPU contact perf (authoring-only)")
    parser.add_argument("--warmup", type=int, default=WARMUP)
    parser.add_argument("--reps", type=int, default=REPS)
    parser.add_argument("--cases", default=None, help="comma separated subset; default all 9")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--log", type=Path, default=DEFAULT_LOG)
    args = parser.parse_args(argv)
    if args.warmup < 0:
        parser.error("--warmup must be >= 0")
    if args.reps < 1:
        parser.error("--reps must be >= 1")
    selected = {spec[0] for spec in CASES}
    if args.cases:
        wanted = [item.strip() for item in args.cases.split(",") if item.strip()]
        unknown = [item for item in wanted if item not in selected]
        if unknown:
            parser.error(f"unknown cases: {', '.join(unknown)}")
        args.selected = set(wanted)
    else:
        args.selected = selected
    args.out = (ROOT / args.out).resolve() if not args.out.is_absolute() else args.out
    args.log = (ROOT / args.log).resolve() if not args.log.is_absolute() else args.log
    return args


def main(argv):
    args = parse_args(argv)
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.log.parent.mkdir(parents=True, exist_ok=True)
    handle = open(args.log, "w", encoding="utf-8", newline="\n")
    original_stdout, original_stderr = sys.stdout, sys.stderr
    sys.stdout = _Tee(original_stdout, handle)
    sys.stderr = _Tee(original_stderr, handle)
    wall_start = time.perf_counter()

    try:
        head = git_head(UPSTREAM_SOURCE)
        print("G10@P5 G6-2 upstream f32 CPU contact perf (authoring-only)", flush=True)
        print(f"upstream: {UPSTREAM}", flush=True)
        print(f"upstream source HEAD: {head} ({UPSTREAM_SOURCE})", flush=True)
        print(f"python: {platform.python_version()} {sys.version.split()[0]}", flush=True)
        print(f"cpu: {cpu_name()} logical={os.cpu_count()} platform={platform.platform()}", flush=True)
        print(f"timed call: mujoco_warp.collision(model, data) + wp.synchronize_device()", flush=True)
        print(f"warmup={args.warmup} reps={args.reps} nconmax={NCONMAX} njmax={NJMAX}", flush=True)

        cache_dir, seeded = prepare_kernel_cache()
        print(f"kernel cache: {cache_dir} (seeded {seeded} module dirs from existing CPU caches)", flush=True)
        wp.set_device("cpu")
        device = wp.get_device()
        if not device.is_cpu:
            raise SystemExit(f"expected cpu device, got {device}")
        print(f"device: {device}", flush=True)
        print(
            "threads: 1 (warp CPU serial launch loop; OMP/MKL/OPENBLAS/NUMEXPR=1; no Python threads)",
            flush=True,
        )

        records = []
        failures = []
        for spec in CASES:
            if spec[0] not in args.selected:
                continue
            try:
                record = measure_case(spec, args.warmup, args.reps)
            except Exception as error:  # noqa: BLE001 - 如实记录未覆盖 case，不伪造数字
                print(f"{spec[0]}: ERROR {type(error).__name__}: {error}", flush=True)
                traceback.print_exc()
                failures.append(
                    {
                        "case": spec[0],
                        "error": f"{type(error).__name__}: {error}",
                        "traceback": traceback.format_exc(),
                    }
                )
                continue
            records.append(record)
            print(
                f"{record['case']}: ncontacts={record['ncontacts']} "
                f"(ref {record['reference_count']}) "
                f"median={record['median_ns'] / 1e6:.3f}ms "
                f"p95={record['p95_ns'] / 1e6:.3f}ms "
                f"contacts/s={record['contacts_per_s']:.1f} "
                f"load={record['load_ms']:.2f}ms compile={record['compile_ms']:.2f}ms "
                f"first_warmup={record['first_warmup_call_ms']:.2f}ms "
                f"count_stable={record['count_stable_across_reps']} "
                f"contact_bitwise_stable={record['contact_bitwise_stable_after_timed_loop']}",
                flush=True,
            )

        wall_s = round(time.perf_counter() - wall_start, 3)
        measured = [record["case"] for record in records]
        uncovered = [item["case"] for item in failures]
        report = {
            "schema": 1,
            "kind": "upstream-cpu-perf",
            "upstream": UPSTREAM,
            "upstream_source_tree": "target/mjwarp-parallel/upstream",
            "upstream_head_verified": head,
            "upstream_head_matches_pinned": head == UPSTREAM,
            "backend": "cpu",
            "device": str(device),
            "warp_version": wp.__version__,
            "mujoco_version": mujoco.__version__,
            "mujoco_warp_version": getattr(mjw, "__version__", "unknown"),
            "python": platform.python_version(),
            "python_full": sys.version,
            "numpy_version": np.__version__,
            "cpu": cpu_name(),
            "cpu_logical_processors": os.cpu_count(),
            "platform": platform.platform(),
            "environment": {
                "device": str(device),
                "cpu": cpu_name(),
                "cpu_logical_processors": os.cpu_count(),
                "python": platform.python_version(),
                "warp_version": wp.__version__,
                "mujoco_version": mujoco.__version__,
                "mujoco_warp_version": getattr(mjw, "__version__", "unknown"),
                "threads": 1,
            },
            "threads": 1,
            "thread_env": {
                name: os.environ.get(name) for name in
                ("OMP_NUM_THREADS", "MKL_NUM_THREADS", "OPENBLAS_NUM_THREADS", "NUMEXPR_NUM_THREADS")
            },
            "warmup": int(args.warmup),
            "reps": int(args.reps),
            "nconmax": NCONMAX,
            "njmax": NJMAX,
            "timed_call": "mujoco_warp.collision(model, data) (broadphase + narrowphase + contact write)",
            "excluded_from_samples": [
                "mujoco.MjModel.from_xml_path (model load/compile)",
                "mujoco_warp.put_model",
                "mujoco_warp.make_data",
                "mujoco_warp.kinematics (once before timing)",
                "first collision JIT compile (inside warmup, not timed)",
                "Python cyclic GC pause (gc.disable during timed loop, gc.collect before each case)",
                "contact count readback (after each timed sample)",
            ],
            "python_gc": "gc.disable() during timed loop (timeit convention); gc.collect() before/after",
            "synchronization": "wp.synchronize_device() after every collision call, inside each timed sample",
            "timer": "time.perf_counter_ns",
            "p95_method": "numpy.percentile(samples, 95) linear interpolation",
            "kernel_cache": {
                "dir": str(cache_dir),
                "seeded_module_dirs": seeded,
                "note": "仅复用本机既有 warp 1.15.0 CPU 内核缓存；JIT 编译不计入性能样本",
            },
            "generated_at": datetime.datetime.now().astimezone().isoformat(timespec="seconds"),
            "cases": records,
            "totals": {
                "cases_measured": len(records),
                "cases_failed": len(failures),
                "load_ms_total": round(sum(item["load_ms"] for item in records), 3),
                "compile_ms_total": round(sum(item["compile_ms"] for item in records), 3),
                "make_data_ms_total": round(sum(item["make_data_ms"] for item in records), 3),
                "timed_samples_s": round(sum(sum(item["samples_ns"]) for item in records) / 1e9, 6),
                "wall_s": wall_s,
            },
            "covered_cases": measured,
            "uncovered_cases": uncovered,
            "failures": failures,
            "note": "上游 f32 CPU 同机同版本参考；性能不进精度判据、不跨硬件外推；编译/加载时间单列不计入",
        }

        text = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
        parsed = json.loads(text)  # 写出前自检
        args.out.write_text(text, encoding="utf-8")
        raw = args.out.read_bytes()
        if raw.startswith(b"\xef\xbb\xbf"):
            raise RuntimeError("output JSON has BOM")
        reparsed = json.loads(raw.decode("utf-8"))
        if reparsed["kind"] != "upstream-cpu-perf" or len(reparsed["cases"]) != len(records):
            raise RuntimeError("output JSON self-check failed")

        print(f"wrote {args.out} bytes={len(raw)} sha256={sha256_file(args.out)}", flush=True)
        print(
            f"done: covered={len(records)}/{len(records) + len(failures)} "
            f"wall={wall_s}s cases={','.join(measured)}",
            flush=True,
        )
        print(f"json_parse_ok=True utf8_no_bom=True", flush=True)
        return 0 if not failures else 1
    finally:
        sys.stdout, sys.stderr = original_stdout, original_stderr
        handle.flush()
        handle.close()


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
