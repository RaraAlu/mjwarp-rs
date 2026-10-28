//! R05-S1 地形 hfield 静态参数（纯宿主；渲染/BVH R05）。
//!
//! 上游锚点（冻结 MJWarp 3.15 `render.py`，`71da24d956378a87a703b6e1442b13aec0c4ac29`）：
//! - L332 `_texture_hfield`：hfield 高程纹理资源；
//! - L462-464 / L682-685 HFIELD 分支：`mjGEOM_HFIELD=1` 的射线求交路径；
//! - L623 / L858 `hfield_bvh_id`：hfield 专用 BVH id（设备槽位）。
//!
//! 输入来源（裁决 A，2026-10-28）：本模块使用独立只读输入 [`HfieldInputs`]，不依赖
//! [`crate::model::render::RenderFields`]——该快照的 116 项字段规范里没有 hfield 通道
//! （`field("hfield_*")` 恒为 `None`），且 `src/model/render.rs` 是 500 行红线文件。
//! 渲染侧 hfield 数据通道由后续卡 `2026-10-28-render-bvh-r05-hfield-channel`
//! （先按 S7b3 先例拆分 `model/render.rs` 再扩 `RENDER_FIELDS`/`io/render.rs`/native）
//! 承担；S2 的数据源适配（`HfieldInputs` 从既有 collision/volume 输入构建）另开卡。
//! 与卡面原签名 `resolve_hfield(&RenderFields)` 的语义等价清单见证据 README。
//!
//! 单份口径：hfield 静态数组按**单份**处理（不读 `nworld`、不按 world 复制），
//! 沿用 R10 单份口径；逐 world 几何/灯数据缺口见 backlog
//! `2026-10-28-render-per-world-geom-light-gap`。
//!
//! 本模块只做校验与宿主只读视图：不分配设备资源、不发射内核、不改 81 槽 ABI
//! （`hfield_bvh_id` 仍为既有占位 `[0u32, 0]`，S2 再接入实际 BVH 并在证据中登记 ABI 决策）。

/// hfield 五个静态数组的只读输入（`hfield_size/nrow/ncol/adr/data`）。
///
/// 语义：任一字段为 `Some` 时五字段必须**同时**提供；全部 `None` 表示模型无 hfield
/// （合法，不新增上游未拒绝的输入）。
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HfieldInputs<'a> {
    /// `hfield_size`：`(nhfield, 4)` 的 f32（宽度 4，长度 `4 × nhfield`）。
    pub size: Option<&'a [f32]>,
    /// `hfield_nrow`：`(nhfield,)` 的 i32。
    pub nrow: Option<&'a [i32]>,
    /// `hfield_ncol`：`(nhfield,)` 的 i32。
    pub ncol: Option<&'a [i32]>,
    /// `hfield_adr`：`(nhfield,)` 的 i32，各单位高程数据的起始下标。
    pub adr: Option<&'a [i32]>,
    /// `hfield_data`：高程数据 f32（行主序，按 `nrow × ncol` 分段）。
    pub data: Option<&'a [f32]>,
}

impl HfieldInputs<'_> {
    /// 是否提供了任一字段。
    pub fn any_present(&self) -> bool {
        self.size.is_some()
            || self.nrow.is_some()
            || self.ncol.is_some()
            || self.adr.is_some()
            || self.data.is_some()
    }

    /// 是否五个字段全部提供。
    pub fn all_present(&self) -> bool {
        self.size.is_some()
            && self.nrow.is_some()
            && self.ncol.is_some()
            && self.adr.is_some()
            && self.data.is_some()
    }

    /// 首个缺失字段名（五字段全给时返回 `None`）。
    pub fn first_missing(&self) -> Option<&'static str> {
        if self.size.is_none() {
            return Some("hfield_size");
        }
        if self.nrow.is_none() {
            return Some("hfield_nrow");
        }
        if self.ncol.is_none() {
            return Some("hfield_ncol");
        }
        if self.adr.is_none() {
            return Some("hfield_adr");
        }
        if self.data.is_none() {
            return Some("hfield_data");
        }
        None
    }
}

/// 单个 hfield 的只读视图（`size[4]` + 行/列 + 数据区段）。
///
/// 字段私有：视图只能由 [`resolve_hfield`] 产出，外部无法构造违反不变量
/// （`nrow/ncol > 0`、`adr + span <= data.len()`）的实例。
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HfieldView {
    size: [f32; 4],
    nrow: i32,
    ncol: i32,
    adr: usize,
    span: usize,
}

impl HfieldView {
    /// `hfield_size` 四项（`(x, y, z, ...)` 的 f32 原值）。
    pub fn size(&self) -> [f32; 4] {
        self.size
    }

    /// 高程行数（恒 `> 0`）。
    pub fn nrow(&self) -> i32 {
        self.nrow
    }

    /// 高程列数（恒 `> 0`）。
    pub fn ncol(&self) -> i32 {
        self.ncol
    }

    /// 高程数据起始下标（恒在 `data` 界内）。
    pub fn adr(&self) -> usize {
        self.adr
    }

    /// 高程跨度 `nrow × ncol`（恒不溢出，且 `adr + span <= data.len()`）。
    pub fn span(&self) -> usize {
        self.span
    }
}

/// 校验后的 hfield 静态参数（宿主只读视图；单份，不随 world 复制）。
///
/// 字段私有：只能由 [`resolve_hfield`] 产出；空输入（`nhfield=0`）返回 `Ok(None)`，
/// 因此本结构一旦存在即至少含一个 hfield。
#[derive(Clone, Debug, PartialEq)]
pub struct HfieldParams<'a> {
    data: &'a [f32],
    hfields: Vec<HfieldView>,
}

impl<'a> HfieldParams<'a> {
    /// hfield 数（`hfield_nrow` 长度，恒 `>= 1`）。
    pub fn len(&self) -> usize {
        self.hfields.len()
    }

    /// 是否无 hfield：恒为 `false`——[`resolve_hfield`] 对空输入（`nhfield=0`）返回
    /// `Ok(None)`，不构造本结构；保留方法仅为与 [`Self::len`] 配对。
    pub fn is_empty(&self) -> bool {
        self.hfields.is_empty()
    }

    /// 整段高程数据（S2 按 [`HfieldView::adr`]/[`HfieldView::span`] 切片）。
    pub fn data(&self) -> &'a [f32] {
        self.data
    }

    /// 逐 hfield 只读视图（顺序与 `hfield_nrow` 一致）。
    pub fn hfields(&self) -> &[HfieldView] {
        &self.hfields
    }

    /// 第 `index` 个 hfield 的高程切片（`nrow × ncol`，行主序）。
    ///
    /// 下标越界或区段计算溢出时返回 `None`（可判定失败，不 panic）。
    pub fn data_of(&self, index: usize) -> Option<&[f32]> {
        let view = self.hfields.get(index)?;
        let end = view.adr.checked_add(view.span)?;
        self.data.get(view.adr..end)
    }
}

/// hfield 参数校验错误；全部在分配/读取前给出，**任何路径不 panic**。
///
/// 与 `ContextError` 同构的变体（`MissingField`/`LengthMismatch`/`CountOverflow`）保持
/// 卡面语义；`NonPositiveRowsCols`/`DataRange` 为 hfield 专用可读错误（含第 index 个
/// 单位的定位与期望/实际值）。
///
/// 说明：原 `FieldKind` 变体已删除（真实构造点 0 处，属死面）；真实渲染通道接入
/// （`2026-10-28-render-bvh-r05-hfield-channel`）若出现类型错配路径，按实际构造点重新引入。
#[derive(Clone, Debug, PartialEq)]
pub enum HfieldError {
    /// 五字段未同时提供（`field` 为首个缺失字段名）。
    MissingField { field: &'static str },
    /// 长度不一致。
    LengthMismatch {
        field: &'static str,
        expected: usize,
        actual: usize,
    },
    /// 计数溢出。
    CountOverflow { field: &'static str },
    /// 第 `index` 个 hfield 的 `nrow`/`ncol` 非正。
    NonPositiveRowsCols { index: usize, nrow: i32, ncol: i32 },
    /// 第 `index` 个 hfield 的 data 区段非法（`adr < 0` 或 `adr + span > data.len()`）。
    DataRange {
        index: usize,
        field: &'static str,
        start: i64,
        end: usize,
        actual: usize,
    },
}

impl std::error::Error for HfieldError {}

impl std::fmt::Display for HfieldError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField { field } => write!(formatter, "缺少 hfield 字段 {field}"),
            Self::LengthMismatch {
                field,
                expected,
                actual,
            } => write!(formatter, "hfield 字段 {field} 长度 {actual} != {expected}"),
            Self::CountOverflow { field } => write!(formatter, "hfield 字段 {field} 计数溢出"),
            Self::NonPositiveRowsCols { index, nrow, ncol } => write!(
                formatter,
                "hfield[{index}] 的 nrow/ncol 必须为正（nrow={nrow}, ncol={ncol}）"
            ),
            Self::DataRange {
                index,
                field,
                start,
                end,
                actual,
            } => write!(
                formatter,
                "hfield[{index}] 的 {field} 区段 [{start}, {end}) 超出长度 {actual}"
            ),
        }
    }
}

/// 校验 hfield 五个静态数组并解析成只读视图。
///
/// 校验矩阵（全部在读取/分配前可诊断，任何路径不 panic）：
/// - 五字段须同时提供：任一 `Some` 而其它 `None` → [`HfieldError::MissingField`]（点名缺的字段）；
/// - 全部 `None` 或全部空切片（`nhfield=0`）→ `Ok(None)`（合法，不新增上游未拒绝的输入）；
/// - 长度：`nrow`/`ncol`/`adr` 长度须相等（`nhfield` 以 `nrow.len()` 为基准），
///   `size.len() == 4 × nhfield`；`data.len() ≥ Σ(nrow[i] × ncol[i])`；
/// - 边界：`nrow[i] <= 0 || ncol[i] <= 0` 拒绝；`nrow × ncol` 用 `checked_mul`；
///   `adr[i] < 0` 或 `adr[i] + span > data.len()` 报 [`HfieldError::DataRange`]（含 index/期望/实际）。
///
/// 单份：本函数不接收 `nworld`，不按 world 复制。
pub fn resolve_hfield<'a>(
    inputs: &HfieldInputs<'a>,
) -> Result<Option<HfieldParams<'a>>, HfieldError> {
    if !inputs.any_present() {
        return Ok(None);
    }
    if !inputs.all_present() {
        let field = inputs.first_missing().unwrap_or("hfield_size");
        return Err(HfieldError::MissingField { field });
    }
    // `all_present` 已保证五项均非 None；用模式匹配避免不必要解包 panic 风险。
    let (Some(size), Some(nrow), Some(ncol), Some(adr), Some(data)) = (
        inputs.size,
        inputs.nrow,
        inputs.ncol,
        inputs.adr,
        inputs.data,
    ) else {
        return Err(HfieldError::MissingField {
            field: "hfield_size",
        });
    };
    let nhfield = nrow.len();
    if ncol.len() != nhfield {
        return Err(HfieldError::LengthMismatch {
            field: "hfield_ncol",
            expected: nhfield,
            actual: ncol.len(),
        });
    }
    if adr.len() != nhfield {
        return Err(HfieldError::LengthMismatch {
            field: "hfield_adr",
            expected: nhfield,
            actual: adr.len(),
        });
    }
    let size_len = nhfield.checked_mul(4).ok_or(HfieldError::CountOverflow {
        field: "hfield_size",
    })?;
    if size.len() != size_len {
        return Err(HfieldError::LengthMismatch {
            field: "hfield_size",
            expected: size_len,
            actual: size.len(),
        });
    }
    if nhfield == 0 {
        // 全部空切片：合法（等价于无 hfield），不构造空视图。
        return Ok(None);
    }
    let mut expected_data = 0usize;
    let mut hfields = Vec::new();
    hfields
        .try_reserve_exact(nhfield)
        .map_err(|_| HfieldError::CountOverflow {
            field: "hfield_count",
        })?;
    for index in 0..nhfield {
        let rows = nrow[index];
        let cols = ncol[index];
        if rows <= 0 || cols <= 0 {
            return Err(HfieldError::NonPositiveRowsCols {
                index,
                nrow: rows,
                ncol: cols,
            });
        }
        let span = usize::try_from(rows)
            .ok()
            .and_then(|rows| {
                usize::try_from(cols)
                    .ok()
                    .and_then(|cols| rows.checked_mul(cols))
            })
            .ok_or(HfieldError::CountOverflow {
                field: "hfield_span",
            })?;
        let offset = i64::from(adr[index]);
        let start = if offset < 0 {
            return Err(HfieldError::DataRange {
                index,
                field: "hfield_adr",
                start: offset,
                end: 0,
                actual: data.len(),
            });
        } else {
            usize::try_from(offset).map_err(|_| HfieldError::CountOverflow {
                field: "hfield_adr",
            })?
        };
        let end = start.checked_add(span).ok_or(HfieldError::DataRange {
            index,
            field: "hfield_data",
            start: offset,
            end: usize::MAX,
            actual: data.len(),
        })?;
        if end > data.len() {
            return Err(HfieldError::DataRange {
                index,
                field: "hfield_data",
                start: offset,
                end,
                actual: data.len(),
            });
        }
        expected_data = expected_data
            .checked_add(span)
            .ok_or(HfieldError::CountOverflow {
                field: "hfield_data",
            })?;
        hfields.push(HfieldView {
            size: [
                size[index * 4],
                size[index * 4 + 1],
                size[index * 4 + 2],
                size[index * 4 + 3],
            ],
            nrow: rows,
            ncol: cols,
            adr: start,
            span,
        });
    }
    // `data.len() >= Σ span`：逐项 `end <= data.len()` 只保证单段在界内；若多个 hfield 的
    // `adr` 重叠，总和仍可能超出，故再显式校验一次（卡面口径）。
    if expected_data > data.len() {
        return Err(HfieldError::LengthMismatch {
            field: "hfield_data",
            expected: expected_data,
            actual: data.len(),
        });
    }
    Ok(Some(HfieldParams { data, hfields }))
}
