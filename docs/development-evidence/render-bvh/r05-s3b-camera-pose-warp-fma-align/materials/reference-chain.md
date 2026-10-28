# 参考链口径与 `mul_quat(identity, cam_quat)` 位等证明

## 参考链是什么（不是单四元数测试）

上游产品路径（pinned `mujoco_warp` 相机 kernel）逐位等价于：

```text
cam_xmat = quat_to_mat(mul_quat(world_quat, cam_quat))
```

本片参考 `materials/gpu-reference-digest.json` 由 `materials/scripts/gpu_quat_reference.py`
在 pinned 环境（warp 1.15.0 / CUDA Toolkit 12.9 / driver 13.2 / RTX 4070 Ti SUPER）上发射
**同一个 chain kernel** 得到，kernel 本体：

```python
q = mjmath.mul_quat(xq[i], cq[i])
outq[i] = q                      # mul_quat 结果（用于下面的位等证明）
out[i] = mjmath.quat_to_mat(q)   # 9 元素 mat33 位型 = 参考
```

因此 JSON 里既有 `mul_quat_bits`（链中间量）也有 `mat_bits`（链末端 9 元素）。

覆盖 1003 条四元数：

| idx | 来源 |
| --- | --- |
| 0 | hfield cam0 `cam_quat` |
| 1 | hfield cam1 `cam_quat`（xyaxes 派生，非平凡） |
| 2 | scene-a cam0 `cam_quat`（显式 quat） |
| 3..1002 | `numpy.random.default_rng(20261029).standard_normal((1000,4))`，f32，内存序 (w,x,y,z) |

## world 体单位四元数时 `mul_quat` 与 `cam_quat` 逐位相等的证明方式

固定相机挂在 world body 上，上游喂给 `mul_quat` 的 `xq` 是世界体四元数，位型 =
`0x3f800000, 0x00000000, 0x00000000, 0x00000000`（mujoco 序 (w,x,y,z) = 单位四元数）。

不是断言，而是机器计数：**对 1003 条样本逐分量比较 `mul_quat_bits` 与输入 `quat_bits`**：

```text
quats: 1003   mul_quat(identity, cam_quat) component-bitwise equal rows: 1003
```

（原始输出见 `materials/reference-chain-identity.txt`；3 条已知相机行的位型逐分量相同）

| 样本 | 输入 `cam_quat` bits (w,x,y,z) | `mul_quat(identity, cam_quat)` bits | 相等 |
| --- | --- | --- | --- |
| hfield cam0 | `3f800000 00000000 00000000 00000000` | `3f800000 00000000 00000000 00000000` | 4/4 |
| hfield cam1 | `3f3d3c57 3f01dbed 3e805783 3ebb0653` | 同左 | 4/4 |
| scene-a cam0 | `3f6c62aa 3dd21ed0 be521ed0 3e9d971c` | 同左 | 4/4 |

1003/1003 行全部相等 ⇒ 参考矩阵就是 `quat_to_mat(cam_quat)` 在**产品链上下文**中的结果；
「唯一性反推」与「1003×9 = 9027 项 0 差异」因此落在整链语义上，而不是只拟合单个四元数。

精确性说明：`mul_quat((1,0,0,0), q)` 的每个分量是 `1*q_i ± 0*q_j …`，正常输入下每步都是精确运算；
-0 极端情形不在本片夹具/随机集内出现（1003/1003 计数）；本片 host 侧不实现该乘法，
`CameraPose::from_fields` 直接对 `cam_quat` 应用收缩序矩阵（见 README §4/§11 限制）。

## 为什么必须用链上下文而不是单独 `quat_to_mat`

RCA `a62cc6aa` 已给机器证据：同一 `cam_quat` 下，**单独** `quat_to_mat` 与产品链（capture）只在
element 6 不同（单独 `0xb2aee30a` vs 链 `0xb31f2429`），其余 8 元素位等 —— 即内联上下文不同会改变
驱动 JIT 的收缩选择。因此本片参考必须走 chain kernel（fma 序也按 chain 上下文反推）。
