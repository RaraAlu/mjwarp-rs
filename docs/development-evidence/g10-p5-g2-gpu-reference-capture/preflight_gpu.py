"""窗口 preflight：打印版本 + CUDA 设备信息（只读；warp 初始化 CUDA context，但不跑采集 kernel）。

用法（窗口脚本调用）：python -B preflight_gpu.py
"""

import sys

if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")

import mujoco
import mujoco_warp as mjw
import numpy as np
import warp as wp

print("python", sys.version.split()[0])
print("warp", wp.__version__)
print("mujoco", mujoco.__version__)
print("mujoco_warp", getattr(mjw, "__version__", "?"))
print("numpy", np.__version__)
devices = wp.get_cuda_devices()
print("cuda_devices", [str(device) for device in devices])
print("global_cuda_device", str(wp.get_device()))
