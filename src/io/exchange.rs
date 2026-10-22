//! I1：数据交互层安全面。
//! 只转调执行层的设备、分配、上传、回读与视图。
//! 不发射内核，也不含`step`与`forward`调度。
//! 物理阶段调度仍归计算物理层。

use crate::diagnostics::{InputError, TransferError};
use crate::io::{DeviceInertialModel, DeviceKinematicModel};
use crate::model::data::DeviceBatch;
use crate::model::{BatchLayout, InertialModelInput, KinematicModelInput};
use crate::runtime::{TransferElement, TransferSession};
use std::ops::Range;

/// 一次设备状态写入的完成回执。
///
/// 执行层安全接口在返回前已完成复制并等待完成；
/// 回执记录会话设备、可选世界与数据代际，供上层核对。
/// 回执不能替代物理结果的有限值与守卫检查。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateReceipt {
    device: usize,
    world: Option<usize>,
    generation: u64,
}

impl StateReceipt {
    pub fn device(self) -> usize {
        self.device
    }

    pub fn world(self) -> Option<usize> {
        self.world
    }

    pub fn generation(self) -> u64 {
        self.generation
    }
}

/// 数据交互层安全面。
///
/// 持有一个执行层传输会话。
/// 上传、回读、模型导入导出与视图全部转调执行层与设备数据安全接口。
/// 不提供内核发射入口，也不组织物理调度。
pub struct DataExchange {
    session: TransferSession,
    generation: u64,
}

impl DataExchange {
    /// 绑定执行层会话；不复制会话状态。
    pub fn new(session: TransferSession) -> Self {
        Self {
            session,
            generation: 0,
        }
    }

    pub fn session(&self) -> &TransferSession {
        &self.session
    }

    pub fn device(&self) -> usize {
        self.session.device()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 分配并上传整批字段；失败不发布半成品。
    pub fn upload<T: TransferElement>(
        &mut self,
        layout: BatchLayout,
        source: &[T],
    ) -> Result<DeviceBatch<T>, TransferError> {
        DeviceBatch::upload(&self.session, layout, source)
    }

    /// 回读整批字段；执行层同步接口返回时已完成复制。
    pub fn read_back<T: TransferElement>(
        &self,
        field: &DeviceBatch<T>,
    ) -> Result<Vec<T>, TransferError> {
        let mut values = crate::runtime::host_staging::<T>(field.layout().total_elements())?;
        field.read_into(&mut values)?;
        Ok(values)
    }

    /// 导入运动学模型；转换仍归数据交互层。
    pub fn import_kinematic(
        &mut self,
        input: &KinematicModelInput,
    ) -> Result<DeviceKinematicModel, TransferError> {
        DeviceKinematicModel::upload(&self.session, input)
    }

    /// 导出运动学模型宿主副本。
    pub fn export_kinematic(
        &self,
        model: &DeviceKinematicModel,
    ) -> Result<KinematicModelInput, TransferError> {
        model.readback()
    }

    /// 导入惯性模型；复用运动学设备模型。
    pub fn import_inertial(
        &mut self,
        input: &InertialModelInput,
    ) -> Result<DeviceInertialModel, TransferError> {
        DeviceInertialModel::upload(&self.session, input)
    }

    /// 导出惯性模型宿主副本。
    pub fn export_inertial(
        &self,
        model: &DeviceInertialModel,
    ) -> Result<InertialModelInput, TransferError> {
        model.readback()
    }

    /// 只读设备视图；生命周期绑定字段借用。
    pub fn world_view<'a, T: TransferElement>(
        &self,
        field: &'a DeviceBatch<T>,
        world: usize,
    ) -> Result<WorldView<'a, T>, TransferError> {
        let range = plan_world_range(field.layout(), world, field.layout().elements_per_world())?;
        Ok(WorldView {
            field,
            world,
            range,
        })
    }

    /// 可写设备视图；借用期间不能创建第二个可写视图。
    pub fn world_view_mut<'a, T: TransferElement>(
        &self,
        field: &'a mut DeviceBatch<T>,
        world: usize,
    ) -> Result<WorldViewMut<'a, T>, TransferError> {
        let range = plan_world_range(field.layout(), world, field.layout().elements_per_world())?;
        Ok(WorldViewMut {
            field,
            world,
            range,
        })
    }

    /// 局部世界写入；返回完成回执。
    pub fn write_world<T: TransferElement>(
        &mut self,
        field: &mut DeviceBatch<T>,
        world: usize,
        source: &[T],
    ) -> Result<StateReceipt, TransferError> {
        plan_world_range(field.layout(), world, source.len())?;
        field.write_world(world, source)?;
        self.receipt(Some(world))
    }

    /// 从另一字段复制一个世界；执行层核对类型与布局。
    pub fn copy_world_from<T: TransferElement>(
        &mut self,
        target: &mut DeviceBatch<T>,
        world: usize,
        source: &DeviceBatch<T>,
        source_world: usize,
    ) -> Result<StateReceipt, TransferError> {
        plan_world_range(target.layout(), world, target.layout().elements_per_world())?;
        plan_world_range(
            source.layout(),
            source_world,
            source.layout().elements_per_world(),
        )?;
        target.copy_world_from(world, source, source_world)?;
        self.receipt(Some(world))
    }

    /// 同一字段内部复制一个世界。
    pub fn copy_world_within<T: TransferElement>(
        &mut self,
        field: &mut DeviceBatch<T>,
        world: usize,
        source_world: usize,
    ) -> Result<StateReceipt, TransferError> {
        plan_world_range(field.layout(), world, field.layout().elements_per_world())?;
        plan_world_range(
            field.layout(),
            source_world,
            field.layout().elements_per_world(),
        )?;
        field.copy_world_within(world, source_world)?;
        self.receipt(Some(world))
    }

    fn receipt(&mut self, world: Option<usize>) -> Result<StateReceipt, TransferError> {
        self.generation = self.generation.checked_add(1).ok_or(InputError::Overflow {
            field: "io_generation",
        })?;
        Ok(StateReceipt {
            device: self.device(),
            world,
            generation: self.generation,
        })
    }
}

/// 只读设备视图。
///
/// 生命周期绑定所属字段：视图不能越过字段存活。
/// 读取经执行层同步接口完成。
///
/// ```compile_fail
/// use mjwarp_rs::io::WorldView;
/// fn widen<'a>(view: WorldView<'a, f32>) -> WorldView<'static, f32> {
///     view
/// }
/// ```
pub struct WorldView<'a, T: TransferElement> {
    field: &'a DeviceBatch<T>,
    world: usize,
    range: Range<usize>,
}

impl<T: TransferElement> WorldView<'_, T> {
    pub fn world(&self) -> usize {
        self.world
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// 读取当前世界；长度不符时拒绝且不写入目标。
    pub fn read_into(&self, target: &mut [T]) -> Result<(), TransferError> {
        self.field.read_world_into(self.world, target)
    }
}

/// 可写设备视图。
///
/// 借用期间字段不能再被写视图或写入接口借用。
/// 视图只提供世界内写入，不提供整批重绑。
///
/// ```compile_fail
/// use mjwarp_rs::io::{DataExchange, DeviceBatch};
/// fn two_views(exchange: &DataExchange, field: &mut DeviceBatch<f32>) {
///     let first = exchange.world_view_mut(field, 0).unwrap();
///     let second = exchange.world_view_mut(field, 1).unwrap();
///     let _ = (first, second);
/// }
/// ```
pub struct WorldViewMut<'a, T: TransferElement> {
    field: &'a mut DeviceBatch<T>,
    world: usize,
    range: Range<usize>,
}

impl<T: TransferElement> WorldViewMut<'_, T> {
    pub fn world(&self) -> usize {
        self.world
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    /// 写入当前世界；长度不符时拒绝且不改写设备。
    pub fn write_from(&mut self, source: &[T]) -> Result<(), TransferError> {
        self.field.write_world(self.world, source)
    }
}

/// 只做宿主校验：世界序号与元素数是否匹配布局。
/// 不访问设备，供数据交互层预检与上层复用。
pub fn plan_world_range(
    layout: BatchLayout,
    world: usize,
    elements: usize,
) -> Result<Range<usize>, InputError> {
    if elements != layout.elements_per_world() {
        return Err(InputError::LengthMismatch {
            field: "world_elements",
            expected: layout.elements_per_world(),
            actual: elements,
        });
    }
    layout.world_elements(world)
}

/// 只做宿主校验：整批元素数是否匹配布局。
pub fn plan_field_elements(layout: BatchLayout, elements: usize) -> Result<(), InputError> {
    if elements != layout.total_elements() {
        return Err(InputError::LengthMismatch {
            field: "field_elements",
            expected: layout.total_elements(),
            actual: elements,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_world_ranges_without_touching_devices() {
        let layout = BatchLayout::new(3, 4, 4).unwrap();
        plan_field_elements(layout, 12).unwrap();
        assert!(matches!(
            plan_field_elements(layout, 11),
            Err(InputError::LengthMismatch {
                field: "field_elements",
                expected: 12,
                actual: 11
            })
        ));
        assert_eq!(plan_world_range(layout, 2, 4).unwrap(), 8..12);
        assert!(matches!(
            plan_world_range(layout, 3, 4),
            Err(InputError::InvalidIndex { field: "world", .. })
        ));
        assert!(matches!(
            plan_world_range(layout, 0, 3),
            Err(InputError::LengthMismatch {
                field: "world_elements",
                expected: 4,
                actual: 3
            })
        ));
    }

    #[test]
    fn empty_worlds_still_plan_an_empty_range() {
        let layout = BatchLayout::new(2, 0, 4).unwrap();
        plan_field_elements(layout, 0).unwrap();
        assert_eq!(plan_world_range(layout, 1, 0).unwrap(), 0..0);
        assert!(plan_world_range(layout, 2, 0).is_err());
    }

    #[test]
    fn receipts_are_ordered_per_exchange() {
        // 只验证回执字段语义；设备路径在GPU定向用例中核对。
        let receipt = StateReceipt {
            device: 0,
            world: Some(2),
            generation: 3,
        };
        assert_eq!(receipt.device(), 0);
        assert_eq!(receipt.world(), Some(2));
        assert_eq!(receipt.generation(), 3);
    }
}
