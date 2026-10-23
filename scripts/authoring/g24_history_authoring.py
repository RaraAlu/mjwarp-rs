"""G24@P7 离线 authoring 辅助：冻结上游 wp.func 回放、重置内核与场景构造。

离线专用：只被 scripts/authoring/author-g24-history.py 调用；
产品构建 / 测试 / 门禁 / 验收脚本都不启动 Python。
"""

from __future__ import annotations

import hashlib
from pathlib import Path

import numpy as np
import warp as wp

from mujoco_warp._src import history as wh
from mujoco_warp._src.types import CtrlChart, GainType

ROOT = Path(__file__).resolve().parents[2]
MODEL_XML = ROOT / "fixtures" / "history" / "model.xml"
FIXTURE_JSON = ROOT / "fixtures" / "history" / "cases.json"
EVIDENCE_DIR = (
    ROOT
    / "docs"
    / "development-evidence"
    / "backlog-batch3"
    / "2026-10-11-roadmap-g24-p7"
)
EVIDENCE_JSON = EVIDENCE_DIR / "authoring.json"
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
PINNED_REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
TASK_ID = "2026-10-11-roadmap-g24-p7"
NATIVE_TOLERANCE = 1e-6

def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def f32(value) -> float:
    return float(np.float32(value))


def f32_list(values) -> list[float]:
    return [f32(value) for value in np.asarray(values).reshape(-1)]


def f32_matrix(values) -> list[list[float]]:
    array = np.asarray(values, dtype=np.float32)
    return [f32_list(row) for row in array.reshape(array.shape[0], -1)]


def region_elements(samples: int, channels: int) -> int:
    return 2 + samples + samples * channels


@wp.kernel
def op_insert_scalar(
    buf: wp.array2d[float],
    worldid: int,
    samples: int,
    t: float,
    value: float,
):
    wh._history_insert_scalar(worldid, 0, samples, t, value, buf)


@wp.kernel
def op_insert_vector(
    src: wp.array2d[float],
    buf: wp.array2d[float],
    worldid: int,
    samples: int,
    dim: int,
    t: float,
):
    wh._history_insert_vector(worldid, 0, samples, dim, t, src, 0, buf)


@wp.kernel
def op_read_scalar(
    buf: wp.array2d[float],
    worldid: int,
    samples: int,
    t: float,
    interp: int,
    out: wp.array2d[float],
    out_adr: int,
):
    out[worldid, out_adr] = wh._history_read_scalar(buf, worldid, 0, samples, t, interp)


@wp.kernel
def op_read_vector(
    buf: wp.array2d[float],
    worldid: int,
    samples: int,
    dim: int,
    t: float,
    interp: int,
    out: wp.array2d[float],
    out_adr: int,
):
    wh._history_read_vector(out_adr, buf, worldid, 0, samples, dim, t, interp, out)


class Scenario:
    """在一个冻结缓冲上回放上游 wp.func 插入/读取脚本。"""

    def __init__(self, name: str, samples: int, channels: int, initial_rows: np.ndarray):
        self.name = name
        self.samples = samples
        self.channels = channels
        self.initial = np.asarray(initial_rows, dtype=np.float32).reshape(
            np.asarray(initial_rows).shape[0], region_elements(samples, channels)
        )
        self.worlds = self.initial.shape[0]
        self.buffer = wp.array(self.initial.copy(), dtype=float)
        self.ops: list[dict] = []
        self.reads: list[dict] = []

    def insert_scalar(self, world: int, t: float, value: float) -> None:
        wp.launch(
            op_insert_scalar,
            dim=1,
            inputs=[self.buffer, world, self.samples, f32(t), f32(value)],
        )
        self.ops.append(
            {"kind": "insert_scalar", "world": world, "t": f32(t), "value": f32(value)}
        )

    def insert_vector(self, world: int, t: float, values) -> None:
        source = np.zeros((self.worlds, self.channels), dtype=np.float32)
        source[world] = np.asarray(values, dtype=np.float32)
        src = wp.array(source, dtype=float)
        wp.launch(
            op_insert_vector,
            dim=1,
            inputs=[src, self.buffer, world, self.samples, self.channels, f32(t)],
        )
        self.ops.append(
            {
                "kind": "insert_vector",
                "world": world,
                "t": f32(t),
                "values": f32_list(values),
            }
        )

    def read(
        self,
        world: int,
        t: float,
        interp: int,
        delay: float | None = None,
        scalar: bool = False,
    ) -> None:
        query = f32(t if delay is None else f32(t) - f32(delay))
        out = wp.zeros((self.worlds, self.channels), dtype=float)
        if scalar:
            wp.launch(
                op_read_scalar,
                dim=1,
                inputs=[self.buffer, world, self.samples, query, interp, out, 0],
            )
            expected = f32_list(np.asarray(out.numpy())[world, :1])
        else:
            wp.launch(
                op_read_vector,
                dim=1,
                inputs=[
                    self.buffer,
                    world,
                    self.samples,
                    self.channels,
                    query,
                    interp,
                    out,
                    0,
                ],
            )
            expected = f32_list(np.asarray(out.numpy())[world, : self.channels])
        record = {
            "world": world,
            "t": f32(t),
            "interp": interp,
            "expected": expected,
        }
        if delay is not None:
            record["delay"] = f32(delay)
        if scalar:
            record["scalar"] = True
        self.reads.append(record)

    def export(self) -> dict:
        return {
            "name": self.name,
            "samples": self.samples,
            "channels": self.channels,
            "initial_words": f32_matrix(self.initial),
            "ops": self.ops,
            "expected_words": f32_matrix(np.asarray(self.buffer.numpy())),
            "reads": self.reads,
        }


def synthetic_actuator_reset(
    worlds: int,
    samples: int,
    dim: int,
    timestep: float,
    gaintype: int = 0,
    ctrlspec: int = 0,
) -> np.ndarray:
    """用上游重置内核冻结合成执行器缓冲的初始字。"""
    region = region_elements(samples, dim)
    history = wp.zeros((worlds, region), dtype=float)
    hist = wp.array(np.array([[[samples, 0]]], dtype=np.int32), dtype=wp.vec2i)
    histadr = wp.array(np.array([[0]], dtype=np.int32), dtype=int)
    ctrlnum = wp.array(np.array([dim], dtype=np.int32), dtype=int)
    gain = wp.array(np.array([gaintype], dtype=np.int32), dtype=int)
    spec = wp.array(np.array([ctrlspec], dtype=np.int32), dtype=int)
    dt = wp.array(np.full(worlds, f32(timestep), dtype=np.float32), dtype=float)
    reset_in = wp.empty(0, dtype=bool)
    wp.launch(
        wh._reset_actuator_history_kernel,
        dim=(worlds, 1),
        inputs=[dt, gain, ctrlnum, spec, hist, histadr, reset_in],
        outputs=[history],
    )
    return np.asarray(history.numpy()).copy()


def model_buffers(mjm, model) -> tuple[list[dict], list[dict]]:
    act_hist = np.asarray(model.actuator_history.numpy())[0]
    act_adr = np.asarray(model.actuator_historyadr.numpy())[0]
    ctrlnum = np.asarray(model.actuator_ctrlnum.numpy())
    delay = np.asarray(model.actuator_delay.numpy())[0]
    gaintype = np.asarray(model.actuator_gaintype.numpy())
    ctrlspec = np.asarray(model.actuator_ctrlspec.numpy())
    actuators = []
    for uid in range(int(mjm.nactuator)):
        samples = int(act_hist[uid][0])
        if samples <= 0:
            continue
        actuators.append(
            {
                "index": uid,
                "buf_offset": int(act_adr[uid]),
                "samples": samples,
                "channels": int(ctrlnum[uid]),
                "interp": int(act_hist[uid][1]),
                "delay": f32(delay[uid]),
                "so3_quat": bool(
                    int(gaintype[uid]) == int(GainType.SO3)
                    and int(ctrlspec[uid]) == int(CtrlChart.QUAT)
                ),
            }
        )
    sens_hist = np.asarray(model.sensor_history.numpy())[0]
    sens_adr = np.asarray(model.sensor_historyadr.numpy())[0]
    sens_dim = np.asarray(model.sensor_dim.numpy())
    sens_delay = np.asarray(model.sensor_delay.numpy())[0]
    sens_interval = np.asarray(model.sensor_interval.numpy())[0]
    sensors = []
    for sid in range(int(mjm.nsensor)):
        samples = int(sens_hist[sid][0])
        if samples <= 0:
            continue
        sensors.append(
            {
                "index": sid,
                "buf_offset": int(sens_adr[sid]),
                "samples": samples,
                "channels": int(sens_dim[sid]),
                "interp": int(sens_hist[sid][1]),
                "delay": f32(sens_delay[sid]),
                "interval": [f32(sens_interval[sid][0]), f32(sens_interval[sid][1])],
            }
        )
    return actuators, sensors


def real_reset(model, nworld: int, words: np.ndarray, mask) -> np.ndarray:
    """用上游重置内核冻结真实模型的全量/掩码局部重置结果。"""
    history = wp.array(np.asarray(words, dtype=np.float32), dtype=float)
    reset_in = (
        wp.empty(0, dtype=bool)
        if mask is None
        else wp.array(np.asarray(mask, dtype=bool), dtype=bool)
    )
    wp.launch(
        wh._reset_actuator_history_kernel,
        dim=(nworld, int(model.nactuator)),
        inputs=[
            model.opt.timestep,
            model.actuator_gaintype,
            model.actuator_ctrlnum,
            model.actuator_ctrlspec,
            model.actuator_history,
            model.actuator_historyadr,
            reset_in,
        ],
        outputs=[history],
    )
    wp.launch(
        wh._reset_sensor_history_kernel,
        dim=(nworld, int(model.nsensor)),
        inputs=[
            model.opt.timestep,
            model.sensor_dim,
            model.sensor_history,
            model.sensor_historyadr,
            model.sensor_interval,
            reset_in,
        ],
        outputs=[history],
    )
    return np.asarray(history.numpy()).copy()


def build_scenarios(model, initial: np.ndarray, actuators: list[dict], sensors: list[dict]):
    nworld = initial.shape[0]
    scenarios = []

    actuator0 = actuators[0]
    offset = actuator0["buf_offset"]
    region = region_elements(actuator0["samples"], actuator0["channels"])
    rows = [initial[world, offset : offset + region] for world in range(nworld)]
    scenario = Scenario("real_actuator0", actuator0["samples"], actuator0["channels"], rows)
    scenario.insert_vector(0, 0.0, [10.0])
    scenario.insert_vector(0, 0.01, [20.0])
    scenario.insert_vector(0, 0.02, [30.0])
    scenario.insert_vector(1, 0.005, [100.0])
    scenario.read(0, 0.0, 1)
    scenario.read(0, 0.005, 1)
    scenario.read(0, 0.015, 0)
    scenario.read(0, 0.015, 1)
    scenario.read(0, -0.5, 1)
    scenario.read(0, 5.0, 1)
    scenario.read(1, 0.0, 1)
    scenario.read(1, 0.005, 1)
    scenario.read(0, 0.015, 1, delay=0.005)
    scenarios.append(scenario)

    sensor0 = sensors[0]
    offset = sensor0["buf_offset"]
    region = region_elements(sensor0["samples"], sensor0["channels"])
    rows = [initial[world, offset : offset + region] for world in range(nworld)]
    scenario = Scenario("real_sensor0", sensor0["samples"], sensor0["channels"], rows)
    scenario.insert_vector(0, 0.0, [1.0])
    scenario.insert_vector(0, 0.01, [2.0])
    scenario.insert_vector(1, 0.02, [3.0])
    scenario.read(0, 0.005, 1)
    scenario.read(0, 0.015, 0)
    scenario.read(1, 0.01, 1)
    scenario.read(1, 0.02, 1)
    scenario.read(0, -1.0, 1)
    scenario.read(0, 0.015, 1, delay=0.01)
    scenarios.append(scenario)

    scalar_initial = synthetic_actuator_reset(2, 4, 1, 0.01)
    scenario = Scenario("scalar_n4", 4, 1, scalar_initial)
    scenario.insert_scalar(0, 0.0, 1.0)
    scenario.insert_scalar(0, 0.01, 2.0)
    scenario.insert_scalar(0, 0.02, 3.0)
    scenario.insert_scalar(0, 0.015, 4.0)
    scenario.insert_scalar(0, -1.0, 6.0)
    scenario.insert_scalar(1, 0.5, 9.0)
    scenario.read(0, -1.0, 1, scalar=True)
    scenario.read(0, -0.005, 0, scalar=True)
    scenario.read(0, -0.005, 1, scalar=True)
    scenario.read(0, 0.0075, 1, scalar=True)
    scenario.read(0, 0.0125, 1, scalar=True)
    scenario.read(0, 0.03, 1, scalar=True)
    scenario.read(0, -2.0, 1, scalar=True)
    scenario.read(1, 0.5, 1, scalar=True)
    scenario.read(1, 0.0, 1, scalar=True)
    scenarios.append(scenario)

    vector_initial = synthetic_actuator_reset(2, 4, 3, 0.01)
    scenario = Scenario("vector_n4_dim3", 4, 3, vector_initial)
    scenario.insert_vector(0, 0.0, [1.0, 2.0, 3.0])
    scenario.insert_vector(0, 0.01, [4.0, 5.0, 6.0])
    scenario.insert_vector(0, 0.02, [7.0, 8.0, 9.0])
    scenario.insert_vector(0, 0.015, [10.0, 11.0, 12.0])
    scenario.insert_vector(0, 0.01, [13.0, 14.0, 15.0])
    scenario.insert_vector(1, 0.005, [100.0, 200.0, 300.0])
    scenario.read(0, 0.0125, 0)
    scenario.read(0, 0.0125, 1)
    scenario.read(0, 0.0175, 1)
    scenario.read(0, 0.015, 1)
    scenario.read(0, -1.0, 1)
    scenario.read(0, 1.0, 1)
    scenario.read(0, 0.0175, 1, delay=0.0025)
    scenario.read(1, 0.004, 1)
    scenario.read(1, 0.005, 1)
    scenarios.append(scenario)
    return scenarios
