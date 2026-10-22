"""Offline authoring common constants and helpers for the U038 3D reference.

OFFLINE ONLY. Not imported by product tests, builds or scripts.
"""

from __future__ import annotations

import hashlib
import importlib.metadata
import json
import math
import pathlib
import re
import subprocess
import sys

import numpy as np

ROOT = pathlib.Path(__file__).resolve().parents[2]
UPSTREAM = ROOT / "target" / "mjwarp-parallel" / "upstream"
CACHE_ROOT = ROOT / "target" / "mjwarp-parallel" / "cache-quarantine" / "warp-1.15.0-double-version-2026-10-18"
CACHE = CACHE_ROOT / "wp_mujoco_warp._src.smooth_ca013d6"
HERE = ROOT / "fixtures" / "gravity-upstream-abi" / "three-d-kinematics"
REVISION = "71da24d956378a87a703b6e1442b13aec0c4ac29"
SMOOTH_SHA256 = "d0952a647a55b4c5f3e89f7224c4842d15c504fe40bafb2fd8bd9b9bd8ef8f4b"
TYPES_SHA256 = "7970e04d40dd15b33171dd1558c369cf45478b6ffdb31d8a805aec93702ec717"
CU_SHA256 = "844dca14a5932827729313a2500f343bdff950b8cdb6170aed1c5973e0ff290e"
META_SHA256 = "2bb1bcdb12cbcce6143af26da5798c047ed628d4817c4b76358fc646db0e3cc8"
PTX_SHA256 = "4179acd01fe8e7064fa97566cfe4158dd9ad4b89761dcab0da88001d9b521f57"
EXPECTED_VERSIONS = {
    "python": "3.12.13",
    "mujoco": "3.15.0",
    "warp": "1.15.0",
    "numpy": "2.5.4",
    "mujoco-warp": "3.15.0",
}
WORLDS = 7
WORLD = 5  # nbody
NJNT = 4
NQ = 13
NV = 11
NBRANCH = 2
NBODY = 5
KERNELS = [
    "_kinematics_branch_95f8028e_cuda_kernel_forward",
    "_compute_body_matrices_27760750_cuda_kernel_forward",
    "_compute_body_inertial_frames_d3bdd81a_cuda_kernel_forward",
    "_subtree_com_init_002b02fd_cuda_kernel_forward",
    "_subtree_com_acc_350f0560_cuda_kernel_forward",
    "_subtree_div_0ef57c26_cuda_kernel_forward",
    "_cdof_3e73eb78_cuda_kernel_forward",
]
GRAVITY_PERIODS = 2
MASS_PERIODS = 3
GRAVCOMP_PERIODS = 5


def digest(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def digest_text(text: str) -> str:
    return hashlib.sha256(text.encode()).hexdigest()


def write_json(path: pathlib.Path, value) -> None:
    path.write_bytes((json.dumps(value, indent=2) + "\n").encode())

PER_JOINT_DOFS = {0: 6, 1: 3, 2: 1, 3: 1}
