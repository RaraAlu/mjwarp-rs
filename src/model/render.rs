//! 渲染模型输入：相机/光源/材质/纹理/网格/vis/名称表。
//! 本模块只定义字段规范、独占快照与名称表语义；原生读取与校验后的模型输入
//! 位于 `io::render`（宿主校验、无设备缓冲、无纹理对象、无BVH）。
//! 字段顺序与 native/model_probe.cpp 的 render_stage 一致。

use crate::diagnostics::{InputError, NativeProbeError};
use CountUnit::{
    Fixed as Fx, Nbody as Nb, Ncam as Cm, Ngeom as Ge, Njnt as Jt, Nlight as Li, Nmat as Mt,
    Nmesh as Me, Nmeshface as Mf, Nmeshgraph as Mg, Nmeshnormal as Mn, Nmeshpoly as My,
    Nmeshpolymap as Mp, Nmeshpolyvert as Mz, Nmeshtexcoord as Tc, Nmeshvert as Mv, Nnames as Nn,
    Nsite as Si, Ntex as Tx, Ntexdata as Td,
};
use RenderFieldKind::{Flag as U, Float as F, Index as I, Num as D, Size as S};

/// 渲染字段的标量类型（原生ABI kind 1..5）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderFieldKind {
    /// 原生int32。
    Index,
    /// 原生double（mjtNum）。
    Num,
    /// 原生uint8（mjtByte/mjtBool）。
    Flag,
    /// 原生int64（mjtSize）。
    Size,
    /// 原生float32。
    Float,
}

impl RenderFieldKind {
    /// 原生ABI kind编号。
    pub const fn code(self) -> u32 {
        match self {
            Self::Index => 1,
            Self::Num => 2,
            Self::Flag => 3,
            Self::Size => 4,
            Self::Float => 5,
        }
    }

    /// 单个标量的字节数。
    pub const fn scalar_bytes(self) -> usize {
        match self {
            Self::Index => 4,
            Self::Num => 8,
            Self::Flag => 1,
            Self::Size => 8,
            Self::Float => 4,
        }
    }
}

/// 字段计数的基数；`Fixed` 表示常量个标量。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CountUnit {
    Fixed,
    Nbody,
    Njnt,
    Ngeom,
    Nsite,
    Ncam,
    Nlight,
    Nmat,
    Ntex,
    Ntexdata,
    Nmesh,
    Nmeshvert,
    Nmeshnormal,
    Nmeshtexcoord,
    Nmeshface,
    Nmeshgraph,
    Nmeshpoly,
    Nmeshpolyvert,
    Nmeshpolymap,
    Nnames,
}

/// 单个渲染字段的规范；期望标量数 = 基数 × 分量宽度。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldSpec {
    pub name: &'static str,
    pub kind: RenderFieldKind,
    pub unit: CountUnit,
    pub width: i64,
}

/// 渲染相关原生计数；全部非负。
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RenderModelCounts {
    pub nbody: i64,
    pub njnt: i64,
    pub ngeom: i64,
    pub nsite: i64,
    pub ncam: i64,
    pub nlight: i64,
    pub nmat: i64,
    pub ntex: i64,
    pub ntexdata: i64,
    pub nmesh: i64,
    pub nmeshvert: i64,
    pub nmeshnormal: i64,
    pub nmeshtexcoord: i64,
    pub nmeshface: i64,
    pub nmeshgraph: i64,
    pub nmeshpoly: i64,
    pub nmeshpolyvert: i64,
    pub nmeshpolymap: i64,
    pub nnames: i64,
}

impl RenderModelCounts {
    /// 校验全部计数非负。
    pub fn validate(self) -> Result<(), NativeProbeError> {
        let counts = [
            ("nbody", self.nbody),
            ("njnt", self.njnt),
            ("ngeom", self.ngeom),
            ("nsite", self.nsite),
            ("ncam", self.ncam),
            ("nlight", self.nlight),
            ("nmat", self.nmat),
            ("ntex", self.ntex),
            ("ntexdata", self.ntexdata),
            ("nmesh", self.nmesh),
            ("nmeshvert", self.nmeshvert),
            ("nmeshnormal", self.nmeshnormal),
            ("nmeshtexcoord", self.nmeshtexcoord),
            ("nmeshface", self.nmeshface),
            ("nmeshgraph", self.nmeshgraph),
            ("nmeshpoly", self.nmeshpoly),
            ("nmeshpolyvert", self.nmeshpolyvert),
            ("nmeshpolymap", self.nmeshpolymap),
            ("nnames", self.nnames),
        ];
        for (field, value) in counts {
            if value < 0 {
                return Err(InputError::InvalidDimension { field }.into());
            }
        }
        Ok(())
    }

    /// 取计数基数的值。
    pub const fn unit(self, unit: CountUnit) -> i64 {
        match unit {
            CountUnit::Fixed => 1,
            CountUnit::Nbody => self.nbody,
            CountUnit::Njnt => self.njnt,
            CountUnit::Ngeom => self.ngeom,
            CountUnit::Nsite => self.nsite,
            CountUnit::Ncam => self.ncam,
            CountUnit::Nlight => self.nlight,
            CountUnit::Nmat => self.nmat,
            CountUnit::Ntex => self.ntex,
            CountUnit::Ntexdata => self.ntexdata,
            CountUnit::Nmesh => self.nmesh,
            CountUnit::Nmeshvert => self.nmeshvert,
            CountUnit::Nmeshnormal => self.nmeshnormal,
            CountUnit::Nmeshtexcoord => self.nmeshtexcoord,
            CountUnit::Nmeshface => self.nmeshface,
            CountUnit::Nmeshgraph => self.nmeshgraph,
            CountUnit::Nmeshpoly => self.nmeshpoly,
            CountUnit::Nmeshpolyvert => self.nmeshpolyvert,
            CountUnit::Nmeshpolymap => self.nmeshpolymap,
            CountUnit::Nnames => self.nnames,
        }
    }

    /// 字段期望标量数；乘积溢出即拒绝。
    pub fn expected(self, spec: &FieldSpec) -> Result<usize, NativeProbeError> {
        self.validate()?;
        if spec.width <= 0 {
            return Err(InputError::InvalidDimension { field: spec.name }.into());
        }
        let count = self
            .unit(spec.unit)
            .checked_mul(spec.width)
            .ok_or(InputError::Overflow { field: spec.name })?;
        usize::try_from(count).map_err(|_| InputError::Overflow { field: spec.name }.into())
    }
}

/// 字段的独占宿主数据。
#[derive(Clone, Debug, PartialEq)]
pub enum FieldData {
    Index(Vec<i32>),
    Num(Vec<f64>),
    Flag(Vec<u8>),
    Size(Vec<i64>),
    Float(Vec<f32>),
}

impl FieldData {
    /// 标量数。
    pub fn len(&self) -> usize {
        match self {
            Self::Index(v) => v.len(),
            Self::Num(v) => v.len(),
            Self::Flag(v) => v.len(),
            Self::Size(v) => v.len(),
            Self::Float(v) => v.len(),
        }
    }

    /// 是否为空字段。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 标量类型。
    pub fn kind(&self) -> RenderFieldKind {
        match self {
            Self::Index(_) => RenderFieldKind::Index,
            Self::Num(_) => RenderFieldKind::Num,
            Self::Flag(_) => RenderFieldKind::Flag,
            Self::Size(_) => RenderFieldKind::Size,
            Self::Float(_) => RenderFieldKind::Float,
        }
    }
}

/// 全部渲染字段的独占快照；顺序严格对应 `RENDER_FIELDS`。
#[derive(Clone, Debug, PartialEq)]
pub struct RenderFields {
    data: Vec<FieldData>,
}

impl RenderFields {
    /// 按字段顺序包装数据。
    pub fn new(data: Vec<FieldData>) -> Self {
        Self { data }
    }

    /// 字段数。
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 按下标取字段。
    pub fn get(&self, id: usize) -> Option<&FieldData> {
        self.data.get(id)
    }

    /// 按字段名取下标。
    pub fn index_of(name: &str) -> Option<usize> {
        RENDER_FIELDS.iter().position(|spec| spec.name == name)
    }

    /// 按字段名取字段数据。
    pub fn field(&self, name: &str) -> Option<&FieldData> {
        Self::index_of(name).and_then(|id| self.data.get(id))
    }

    pub(crate) fn ints(&self, name: &str) -> Result<&[i32], InputError> {
        match self.field(name) {
            Some(FieldData::Index(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "render_field_kind",
            }),
        }
    }

    pub(crate) fn bytes(&self, name: &str) -> Result<&[u8], InputError> {
        match self.field(name) {
            Some(FieldData::Flag(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "render_field_kind",
            }),
        }
    }

    pub(crate) fn sizes(&self, name: &str) -> Result<&[i64], InputError> {
        match self.field(name) {
            Some(FieldData::Size(v)) => Ok(v),
            _ => Err(InputError::InvalidDimension {
                field: "render_field_kind",
            }),
        }
    }

    /// 名称字节；语义与 `mj_id2name` 一致。
    pub fn name_bytes(&self, object: RenderObject, id: i64) -> Option<&[u8]> {
        let table = self.ints(object.table()).ok()?;
        let names = self.bytes("names").ok()?;
        table_name(table, names, id)
    }

    /// UTF-8名称；非UTF-8名称返回 `None`，此时用 `name_bytes`。
    pub fn name(&self, object: RenderObject, id: i64) -> Option<&str> {
        std::str::from_utf8(self.name_bytes(object, id)?).ok()
    }
}

/// 名称表对象类型；值等于MuJoCo的 mjOBJ_*。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderObject {
    Body = 1,
    Joint = 3,
    Geom = 5,
    Site = 6,
    Camera = 7,
    Light = 8,
    Mesh = 10,
    Texture = 13,
    Material = 14,
}

/// 全部名称表对象；用于自洽校验与名称语义测试。
pub const RENDER_OBJECTS: [RenderObject; 9] = [
    RenderObject::Body,
    RenderObject::Joint,
    RenderObject::Geom,
    RenderObject::Site,
    RenderObject::Camera,
    RenderObject::Light,
    RenderObject::Mesh,
    RenderObject::Texture,
    RenderObject::Material,
];

impl RenderObject {
    /// 原生 mjOBJ_* 编号。
    pub const fn mj_obj(self) -> i32 {
        self as i32
    }

    /// 名称偏移表字段名。
    pub const fn table(self) -> &'static str {
        match self {
            Self::Body => "name_bodyadr",
            Self::Joint => "name_jntadr",
            Self::Geom => "name_geomadr",
            Self::Site => "name_siteadr",
            Self::Camera => "name_camadr",
            Self::Light => "name_lightadr",
            Self::Mesh => "name_meshadr",
            Self::Texture => "name_texadr",
            Self::Material => "name_matadr",
        }
    }

    /// 该类型的对象个数。
    pub const fn count(self, counts: RenderModelCounts) -> i64 {
        match self {
            Self::Body => counts.nbody,
            Self::Joint => counts.njnt,
            Self::Geom => counts.ngeom,
            Self::Site => counts.nsite,
            Self::Camera => counts.ncam,
            Self::Light => counts.nlight,
            Self::Mesh => counts.nmesh,
            Self::Texture => counts.ntex,
            Self::Material => counts.nmat,
        }
    }
}

/// 按 `mj_id2name` 语义查找名称字节：越界/负偏移/缺NUL返回 `None`，空名返回 `Some(&[])`。
pub(crate) fn table_name<'a>(table: &[i32], names: &'a [u8], id: i64) -> Option<&'a [u8]> {
    let id = usize::try_from(id).ok()?;
    let adr = usize::try_from(*table.get(id)?).ok()?;
    let tail = names.get(adr..)?;
    let end = tail.iter().position(|&byte| byte == 0)?;
    Some(&tail[..end])
}

const fn f(name: &'static str, kind: RenderFieldKind, unit: CountUnit, width: i64) -> FieldSpec {
    FieldSpec {
        name,
        kind,
        unit,
        width,
    }
}

/// 渲染字段规范；顺序与 `native/model_probe.cpp` 的 `render_stage` 一致。
pub const RENDER_FIELDS: [FieldSpec; 116] = [
    f("geom_type", I, Ge, 1),
    f("geom_bodyid", I, Ge, 1),
    f("geom_dataid", I, Ge, 1),
    f("geom_matid", I, Ge, 1),
    f("geom_group", I, Ge, 1),
    f("geom_size", D, Ge, 3),
    f("geom_pos", D, Ge, 3),
    f("geom_quat", D, Ge, 4),
    f("geom_rgba", F, Ge, 4),
    f("cam_mode", I, Cm, 1),
    f("cam_bodyid", I, Cm, 1),
    f("cam_targetbodyid", I, Cm, 1),
    f("cam_projection", I, Cm, 1),
    f("cam_resolution", I, Cm, 2),
    f("cam_output", I, Cm, 1),
    f("cam_fovy", D, Cm, 1),
    f("cam_ipd", D, Cm, 1),
    f("cam_sensorsize", F, Cm, 2),
    f("cam_intrinsic", F, Cm, 4),
    f("cam_pos", D, Cm, 3),
    f("cam_quat", D, Cm, 4),
    f("cam_poscom0", D, Cm, 3),
    f("cam_pos0", D, Cm, 3),
    f("cam_mat0", D, Cm, 9),
    f("light_mode", I, Li, 1),
    f("light_bodyid", I, Li, 1),
    f("light_targetbodyid", I, Li, 1),
    f("light_type", I, Li, 1),
    f("light_texid", I, Li, 1),
    f("light_castshadow", U, Li, 1),
    f("light_active", U, Li, 1),
    f("light_bulbradius", F, Li, 1),
    f("light_intensity", F, Li, 1),
    f("light_range", F, Li, 1),
    f("light_attenuation", F, Li, 3),
    f("light_cutoff", F, Li, 1),
    f("light_softness", F, Li, 1),
    f("light_exponent", F, Li, 1),
    f("light_ambient", F, Li, 3),
    f("light_diffuse", F, Li, 3),
    f("light_specular", F, Li, 3),
    f("light_pos", D, Li, 3),
    f("light_dir", D, Li, 3),
    f("light_poscom0", D, Li, 3),
    f("light_pos0", D, Li, 3),
    f("light_dir0", D, Li, 3),
    f("mat_texid", I, Mt, 10),
    f("mat_texuniform", U, Mt, 1),
    f("mat_texrepeat", F, Mt, 2),
    f("mat_emission", F, Mt, 1),
    f("mat_specular", F, Mt, 1),
    f("mat_shininess", F, Mt, 1),
    f("mat_reflectance", F, Mt, 1),
    f("mat_metallic", F, Mt, 1),
    f("mat_roughness", F, Mt, 1),
    f("mat_rgba", F, Mt, 4),
    f("tex_type", I, Tx, 1),
    f("tex_colorspace", I, Tx, 1),
    f("tex_height", I, Tx, 1),
    f("tex_width", I, Tx, 1),
    f("tex_nchannel", I, Tx, 1),
    f("tex_adr", S, Tx, 1),
    f("tex_data", U, Td, 1),
    f("tex_pathadr", I, Tx, 1),
    f("mesh_vertadr", I, Me, 1),
    f("mesh_vertnum", I, Me, 1),
    f("mesh_faceadr", I, Me, 1),
    f("mesh_facenum", I, Me, 1),
    f("mesh_bvhadr", I, Me, 1),
    f("mesh_bvhnum", I, Me, 1),
    f("mesh_octadr", I, Me, 1),
    f("mesh_octnum", I, Me, 1),
    f("mesh_normaladr", I, Me, 1),
    f("mesh_normalnum", I, Me, 1),
    f("mesh_texcoordadr", I, Me, 1),
    f("mesh_texcoordnum", I, Me, 1),
    f("mesh_graphadr", I, Me, 1),
    f("mesh_extrema", I, Me, 27),
    f("mesh_vert", F, Mv, 3),
    f("mesh_normal", F, Mn, 3),
    f("mesh_texcoord", F, Tc, 2),
    f("mesh_face", I, Mf, 3),
    f("mesh_facenormal", I, Mf, 3),
    f("mesh_facetexcoord", I, Mf, 3),
    f("mesh_graph", I, Mg, 1),
    f("mesh_scale", D, Me, 3),
    f("mesh_pos", D, Me, 3),
    f("mesh_quat", D, Me, 4),
    f("mesh_pathadr", I, Me, 1),
    f("mesh_polynum", I, Me, 1),
    f("mesh_polyadr", I, Me, 1),
    f("mesh_polynormal", D, My, 3),
    f("mesh_polyvertadr", I, My, 1),
    f("mesh_polyvertnum", I, My, 1),
    f("mesh_polyvert", I, Mz, 1),
    f("mesh_polymapadr", I, Mv, 1),
    f("mesh_polymapnum", I, Mv, 1),
    f("mesh_polymap", I, Mp, 1),
    f("vis.headlight.ambient", F, Fx, 3),
    f("vis.headlight.diffuse", F, Fx, 3),
    f("vis.headlight.specular", F, Fx, 3),
    f("vis.headlight.active", I, Fx, 1),
    f("vis.map.znear", F, Fx, 1),
    f("vis.map.zfar", F, Fx, 1),
    f("stat.extent", D, Fx, 1),
    f("stat.center", D, Fx, 3),
    f("name_bodyadr", I, Nb, 1),
    f("name_jntadr", I, Jt, 1),
    f("name_geomadr", I, Ge, 1),
    f("name_siteadr", I, Si, 1),
    f("name_camadr", I, Cm, 1),
    f("name_lightadr", I, Li, 1),
    f("name_meshadr", I, Me, 1),
    f("name_texadr", I, Tx, 1),
    f("name_matadr", I, Mt, 1),
    f("names", U, Nn, 1),
];
