//! L v2：host 逐 world 灯位姿容器与逐 world 透传选择（spec v3 §10.3；`.cu`/81 槽零改动）。
//!
//! 机制：内核灯数组仍是 **1-D vec3**（`render_megakernel.cu` 37/38 槽
//! `light_xpos_in`/`light_xdir_in`），per-world 发射时内核 `worldid` 恒 0 →
//! 每次发射前把目标 world 的灯位姿重传到这两个数组即可；不新增 2-D 灯数组、
//! 不改单发射结构、不改 81 槽签名（内核身份不变）。
//!
//! 数据口径（与 spec v3 §10.3 / §5 一致）：逐 world 的 [`LightDataPose`] 的
//! `xpos`/`xdir` 为 **f32 位型**，直接取采集侧 `state.npz` 的 `light_xpos`/
//! `light_xdir` 原值；**不做 FK 推导、不做 f64 重算**。`xdir` 已是上游
//! `normalize` 结果，本模块**逐位透传、不重新归一化**（重归一化可能改变末位，
//! 破坏与参考的位级对齐）。
//!
//! 默认单 world 路径的 [`super::megakernel_arrays::light_data_poses`] 语义不变：
//! 仍要求 `light_mode == 0` / `light_bodyid == 0`，归一化沿用
//! `crate::math::fma::fma_f32` 序列；该拒绝面只在默认路径生效，不因本入口全局放开。
//!
//! 所有校验在发射前返回显式错误，不 panic。

use crate::runtime::launcher::LauncherError;

use super::megakernel_arrays::LightDataPose;

/// 逐 world 灯位姿的显式错误（不发 GPU、不 panic）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorldLightError {
    /// `worlds == 0`。
    WorldCountZero,
    /// `lights == 0`。
    LightCountZero,
    /// 位姿条目数不等于 `worlds × lights`。
    CountMismatch { expected: usize, actual: usize },
    /// world 下标越界。
    WorldOutOfRange { world: usize, worlds: usize },
    /// 灯下标越界。
    LightOutOfRange { light: usize, lights: usize },
    /// 计数乘加溢出。
    CountOverflow { field: &'static str },
}

impl std::error::Error for WorldLightError {}

impl std::fmt::Display for WorldLightError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldCountZero => write!(formatter, "nworld 必须 >= 1"),
            Self::LightCountZero => write!(formatter, "nlight 必须 >= 1"),
            Self::CountMismatch { expected, actual } => write!(
                formatter,
                "per-world 灯位姿条目数 {actual} != nworld×nlight {expected}"
            ),
            Self::WorldOutOfRange { world, worlds } => {
                write!(formatter, "world {world} 越界（worlds={worlds}）")
            }
            Self::LightOutOfRange { light, lights } => {
                write!(formatter, "灯下标 {light} 越界（lights={lights}）")
            }
            Self::CountOverflow { field } => write!(formatter, "{field} 计数溢出"),
        }
    }
}

/// `nworld × nlight` 灯位姿；world-major（`world * nlight + light`）。
///
/// 每个条目是采集侧 `state.npz` 的 `light_xpos`/`light_xdir` f32 原值
/// （见模块文档的数据口径）。
#[derive(Clone, Debug, PartialEq)]
pub struct WorldLightPoses {
    worlds: usize,
    lights: usize,
    poses: Vec<LightDataPose>,
}

impl WorldLightPoses {
    /// 校验 `worlds >= 1`、`lights >= 1` 且 `poses.len() == worlds × lights`。
    pub fn new(
        worlds: usize,
        lights: usize,
        poses: Vec<LightDataPose>,
    ) -> Result<Self, WorldLightError> {
        if worlds == 0 {
            return Err(WorldLightError::WorldCountZero);
        }
        if lights == 0 {
            return Err(WorldLightError::LightCountZero);
        }
        let expected = worlds
            .checked_mul(lights)
            .ok_or(WorldLightError::CountOverflow {
                field: "world_light_poses",
            })?;
        if poses.len() != expected {
            return Err(WorldLightError::CountMismatch {
                expected,
                actual: poses.len(),
            });
        }
        Ok(Self {
            worlds,
            lights,
            poses,
        })
    }

    /// world 数。
    pub fn worlds(&self) -> usize {
        self.worlds
    }

    /// 每 world 灯数。
    pub fn lights(&self) -> usize {
        self.lights
    }

    /// `(world, light)` 的位姿（拷贝）。
    pub fn get(&self, world: usize, light: usize) -> Result<LightDataPose, WorldLightError> {
        if world >= self.worlds {
            return Err(WorldLightError::WorldOutOfRange {
                world,
                worlds: self.worlds,
            });
        }
        if light >= self.lights {
            return Err(WorldLightError::LightOutOfRange {
                light,
                lights: self.lights,
            });
        }
        Ok(self.poses[self.index(world, light)?])
    }

    /// 某 world 的灯位姿切片（长度 = `lights`）。
    pub fn world_slice(&self, world: usize) -> Result<&[LightDataPose], WorldLightError> {
        if world >= self.worlds {
            return Err(WorldLightError::WorldOutOfRange {
                world,
                worlds: self.worlds,
            });
        }
        let start = self.index(world, 0)?;
        let end = start
            .checked_add(self.lights)
            .ok_or(WorldLightError::CountOverflow {
                field: "world_light_poses",
            })?;
        Ok(&self.poses[start..end])
    }

    fn index(&self, world: usize, light: usize) -> Result<usize, WorldLightError> {
        world
            .checked_mul(self.lights)
            .and_then(|base| base.checked_add(light))
            .ok_or(WorldLightError::CountOverflow {
                field: "world_light_poses",
            })
    }
}

/// 逐 world 展平为内核 1-D 数组（`light_xpos_in`/`light_xdir_in`），顺序 = world 内灯序。
pub(crate) fn flatten_light_poses(poses: &[LightDataPose]) -> (Vec<f32>, Vec<f32>) {
    let mut xpos = Vec::with_capacity(3 * poses.len());
    let mut xdir = Vec::with_capacity(3 * poses.len());
    for pose in poses {
        xpos.extend_from_slice(&pose.xpos);
        xdir.extend_from_slice(&pose.xdir);
    }
    (xpos, xdir)
}

/// per-world 灯入口选灯：校验灯数与 `fields.light_type` 长度一致、world 在范围内，
/// 并展平该 world 的 1-D 数组；错误映射为可诊断 [`LauncherError`]（不 panic）。
pub(crate) fn select_world_lights(
    lights: &WorldLightPoses,
    world: usize,
    nlight: usize,
) -> Result<(Vec<f32>, Vec<f32>), LauncherError> {
    if lights.lights() != nlight {
        return Err(LauncherError::InvalidArray("per_world_light_count"));
    }
    let slice = lights
        .world_slice(world)
        .map_err(|error| LauncherError::Driver(format!("per-world lights: {error}")))?;
    Ok(flatten_light_poses(slice))
}

#[cfg(test)]
mod tests {
    use crate::model::render::{FieldData, RENDER_FIELDS, RenderFieldKind, RenderFields};

    use super::super::megakernel_arrays::light_data_poses;
    use super::*;

    const MODEL: &str = include_str!("../../../fixtures/render-bvh/scene-a/model-312.json");

    /// 既有 `fixtures/render-bvh/scene-a/s7b/rgb.json` 的 `light_data_check` 位型基准：
    /// pinned 3.15 forward 的 `d.light_xpos`/`d.light_xdir`（xpos_bits, xdir_bits）。
    const LIGHT_DATA_CHECK: [([u32; 3], [u32; 3]); 3] = [
        (
            [1065353216, 1065353216, 1077936128],
            [3201800005, 3198974212, 3210188613],
        ),
        ([1053609165, 1056964608, 1067030938], [0, 0, 3212836864]),
        ([0, 0, 1073741824], [0, 0, 3212836864]),
    ];

    fn light_pose(seed: f32) -> LightDataPose {
        LightDataPose {
            xpos: [seed, seed + 1.0, seed + 2.0],
            xdir: [0.0, 0.0, -1.0],
        }
    }

    fn field_data() -> Vec<FieldData> {
        let model: serde_json::Value = serde_json::from_str(MODEL).expect("model-312.json");
        let arrays = &model["arrays"];
        RENDER_FIELDS
            .iter()
            .map(|spec| {
                let values = arrays[spec.name]["values"].as_array().expect("渲染字段");
                match spec.kind {
                    RenderFieldKind::Index => FieldData::Index(
                        values.iter().map(|v| v.as_i64().unwrap() as i32).collect(),
                    ),
                    RenderFieldKind::Num => {
                        FieldData::Num(values.iter().map(|v| v.as_f64().unwrap()).collect())
                    }
                    RenderFieldKind::Flag => {
                        FieldData::Flag(values.iter().map(|v| v.as_u64().unwrap() as u8).collect())
                    }
                    RenderFieldKind::Size => {
                        FieldData::Size(values.iter().map(|v| v.as_i64().unwrap()).collect())
                    }
                    RenderFieldKind::Float => FieldData::Float(
                        values.iter().map(|v| v.as_f64().unwrap() as f32).collect(),
                    ),
                }
            })
            .collect()
    }

    #[test]
    fn keeps_world_major_light_poses_and_round_trips_bits() {
        let poses: Vec<LightDataPose> = (0..12)
            .map(|index| light_pose(index as f32 * 0.5))
            .collect();
        let world = WorldLightPoses::new(3, 4, poses.clone()).unwrap();
        assert_eq!((world.worlds(), world.lights()), (3, 4));
        for (index, expected) in poses.iter().enumerate() {
            let actual = world.get(index / 4, index % 4).unwrap();
            assert_eq!(
                actual.xpos.map(f32::to_bits),
                expected.xpos.map(f32::to_bits)
            );
            assert_eq!(
                actual.xdir.map(f32::to_bits),
                expected.xdir.map(f32::to_bits)
            );
        }
        let slice = world.world_slice(2).unwrap();
        assert_eq!(slice.len(), 4);
        assert_eq!(
            slice[0].xpos.map(f32::to_bits),
            poses[8].xpos.map(f32::to_bits)
        );
    }

    #[test]
    fn rejects_count_mismatch_zero_sides_and_overflow_without_panic() {
        assert!(matches!(
            WorldLightPoses::new(3, 3, vec![light_pose(0.0); 8]),
            Err(WorldLightError::CountMismatch {
                expected: 9,
                actual: 8
            })
        ));
        assert_eq!(
            WorldLightPoses::new(0, 3, Vec::new()),
            Err(WorldLightError::WorldCountZero)
        );
        assert_eq!(
            WorldLightPoses::new(3, 0, Vec::new()),
            Err(WorldLightError::LightCountZero)
        );
        assert!(matches!(
            WorldLightPoses::new(usize::MAX, 2, Vec::new()),
            Err(WorldLightError::CountOverflow { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_world_and_light() {
        let world = WorldLightPoses::new(2, 3, vec![light_pose(0.0); 6]).unwrap();
        assert_eq!(
            world.get(2, 0),
            Err(WorldLightError::WorldOutOfRange {
                world: 2,
                worlds: 2
            })
        );
        assert_eq!(
            world.get(0, 3),
            Err(WorldLightError::LightOutOfRange {
                light: 3,
                lights: 3
            })
        );
        assert!(world.world_slice(2).is_err());
    }

    #[test]
    fn flattens_target_world_in_light_order_with_verbatim_bits() {
        let poses = vec![
            light_pose(1.0),
            light_pose(2.0),
            light_pose(10.0),
            light_pose(20.0),
        ];
        let world = WorldLightPoses::new(2, 2, poses).unwrap();
        let (xpos, xdir) = flatten_light_poses(world.world_slice(1).unwrap());
        assert_eq!(
            xpos.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            [10.0f32, 11.0, 12.0, 20.0, 21.0, 22.0]
                .iter()
                .map(|value| value.to_bits())
                .collect::<Vec<_>>()
        );
        assert_eq!(xdir.len(), 6);
        assert_eq!(xdir[2].to_bits(), (-1.0f32).to_bits());
    }

    #[test]
    fn select_world_lights_rejects_light_count_mismatch_and_bad_world() {
        let world = WorldLightPoses::new(2, 3, vec![light_pose(0.0); 6]).unwrap();
        assert!(matches!(
            select_world_lights(&world, 0, 2),
            Err(LauncherError::InvalidArray("per_world_light_count"))
        ));
        assert!(matches!(
            select_world_lights(&world, 2, 3),
            Err(LauncherError::Driver(_))
        ));
        assert!(select_world_lights(&world, 1, 3).is_ok());
    }

    #[test]
    fn default_path_light_data_poses_keep_golden_bits() {
        let fields = RenderFields::new(field_data());
        let poses = light_data_poses(&fields).expect("默认路径灯位姿");
        assert_eq!(poses.len(), LIGHT_DATA_CHECK.len());
        for (pose, (xpos_bits, xdir_bits)) in poses.iter().zip(LIGHT_DATA_CHECK) {
            assert_eq!(pose.xpos.map(f32::to_bits), xpos_bits, "xpos 位型回归");
            assert_eq!(pose.xdir.map(f32::to_bits), xdir_bits, "xdir 位型回归");
        }
    }

    #[test]
    fn default_path_still_rejects_tracked_lights() {
        for (field, replacement) in [
            ("light_mode", FieldData::Index(vec![1, 0, 0])),
            ("light_bodyid", FieldData::Index(vec![4, 0, 0])),
        ] {
            let mut data = field_data();
            data[RenderFields::index_of(field).expect(field)] = replacement;
            let fields = RenderFields::new(data);
            assert!(
                light_data_poses(&fields).is_err(),
                "默认路径必须拒绝 {field} != 0（拒绝面不因 per-world 灯入口全局放开）"
            );
        }
    }
}
