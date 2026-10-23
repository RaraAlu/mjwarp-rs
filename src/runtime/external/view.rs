//! A08@P10 零拷贝切片：借用式设备数据视图、所有权与跨流完成事件（宿主侧契约）。
//!
//! 对齐矩阵 A08「上下文、所有权与跨流事件」的宿主侧最小可交付：
//!
//! - **上下文**：`MirrorBuffer` 记录设备序号/上下文/分配身份与完成事件所属流；
//!   视图只在该缓冲的流上、事件完成后构造。
//! - **所有权**：`ZeroCopyView` 只借用 `MirrorBuffer`（Rust 生命周期），不复制、
//!   不持有 `Vec`；`field()`/`elements()` 返回指向原缓冲的切片（指针相等可判定）。
//! - **跨流事件**：`StreamFence` 复刻 `runtime::completion` 的提交/完成契约
//!   （流编号 + 代数 + 完成位）；事件未完成或流/代数不匹配时拒绝构造视图。
//!
//! 纯宿主侧契约与算术：不分配设备内存、不发射内核、不做真实流同步；真实 CUDA
//! 事件与设备指针所有权在零拷贝 GPU 路径具备后另行验收。

use std::error::Error;
use std::fmt;

/// 流完成事件（宿主侧模型；对齐 completion 的 submit/wait 契约）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StreamFence {
    pub stream: u32,
    pub generation: u64,
    completed: bool,
}

impl StreamFence {
    /// 新建待完成事件（对应一次提交）。
    pub fn pending(stream: u32, generation: u64) -> Self {
        Self {
            stream,
            generation,
            completed: false,
        }
    }

    /// 新建已完成事件（对应 wait 成功返回）。
    pub fn completed(stream: u32, generation: u64) -> Self {
        Self {
            stream,
            generation,
            completed: true,
        }
    }

    /// 标记完成；只允许单调从待完成到完成。
    pub fn complete(&mut self) -> Result<(), ZeroCopyError> {
        if self.completed {
            return Err(ZeroCopyError::AlreadyCompleted {
                stream: self.stream,
            });
        }
        self.completed = true;
        Ok(())
    }

    pub fn is_completed(self) -> bool {
        self.completed
    }
}

/// 设备缓冲的宿主镜像（零拷贝视图的所有者）。
#[derive(Clone, Debug, PartialEq)]
pub struct MirrorBuffer {
    pub name: &'static str,
    pub device_ordinal: u32,
    pub context: u64,
    pub allocation: u64,
    pub stream: u32,
    pub generation: u64,
    data: Vec<f32>,
}

impl MirrorBuffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: &'static str,
        device_ordinal: u32,
        context: u64,
        allocation: u64,
        stream: u32,
        data: Vec<f32>,
    ) -> Self {
        Self {
            name,
            device_ordinal,
            context,
            allocation,
            stream,
            generation: 0,
            data,
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn data(&self) -> &[f32] {
        &self.data
    }

    /// 写入后递增代数（对应一次新提交）。
    pub fn write(&mut self, index: usize, value: f32) -> Result<u64, ZeroCopyError> {
        if index >= self.data.len() {
            return Err(ZeroCopyError::OutOfBounds {
                field: "write",
                index,
                limit: self.data.len(),
            });
        }
        self.data[index] = value;
        self.generation += 1;
        Ok(self.generation)
    }
}

/// 字段布局：连续（stride == width）或跨步访问。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FieldLayout {
    pub offset: usize,
    pub width: usize,
    pub stride: usize,
    pub count: usize,
}

impl FieldLayout {
    /// 连续字段。
    pub fn contiguous(offset: usize, width: usize, count: usize) -> Self {
        Self {
            offset,
            width,
            stride: width,
            count,
        }
    }
}

/// 零拷贝错误。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ZeroCopyError {
    EmptyBuffer {
        name: &'static str,
    },
    OutOfBounds {
        field: &'static str,
        index: usize,
        limit: usize,
    },
    EventPending {
        stream: u32,
        generation: u64,
    },
    StreamMismatch {
        buffer_stream: u32,
        event_stream: u32,
    },
    GenerationMismatch {
        buffer_generation: u64,
        event_generation: u64,
    },
    AlreadyCompleted {
        stream: u32,
    },
}

impl fmt::Display for ZeroCopyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBuffer { name } => write!(formatter, "空缓冲不可建视图：{name}"),
            Self::OutOfBounds {
                field,
                index,
                limit,
            } => write!(formatter, "访问越界：{field}[{index}]，上限 {limit}"),
            Self::EventPending { stream, generation } => write!(
                formatter,
                "完成事件未完成：stream={stream} generation={generation}"
            ),
            Self::StreamMismatch {
                buffer_stream,
                event_stream,
            } => write!(
                formatter,
                "跨流事件不匹配：buffer stream={buffer_stream}，event stream={event_stream}"
            ),
            Self::GenerationMismatch {
                buffer_generation,
                event_generation,
            } => write!(
                formatter,
                "事件代数不匹配：buffer={buffer_generation}，event={event_generation}"
            ),
            Self::AlreadyCompleted { stream } => {
                write!(formatter, "事件已完成，不能重复完成：stream={stream}")
            }
        }
    }
}

impl Error for ZeroCopyError {}

/// 借用式零拷贝视图：生命周期绑定所有者，不复制数据。
#[derive(Debug)]
pub struct ZeroCopyView<'a> {
    buffer: &'a MirrorBuffer,
    layout: FieldLayout,
}

impl<'a> ZeroCopyView<'a> {
    /// 在“同流 + 已完成 + 同代数”的事件下构造视图。
    pub fn new(
        buffer: &'a MirrorBuffer,
        layout: FieldLayout,
        event: StreamFence,
    ) -> Result<Self, ZeroCopyError> {
        if buffer.is_empty() {
            return Err(ZeroCopyError::EmptyBuffer { name: buffer.name });
        }
        if event.stream != buffer.stream {
            return Err(ZeroCopyError::StreamMismatch {
                buffer_stream: buffer.stream,
                event_stream: event.stream,
            });
        }
        if !event.is_completed() {
            return Err(ZeroCopyError::EventPending {
                stream: event.stream,
                generation: event.generation,
            });
        }
        if event.generation != buffer.generation {
            return Err(ZeroCopyError::GenerationMismatch {
                buffer_generation: buffer.generation,
                event_generation: event.generation,
            });
        }
        if layout.width == 0 || layout.count == 0 {
            return Err(ZeroCopyError::OutOfBounds {
                field: "layout",
                index: 0,
                limit: 0,
            });
        }
        let last = layout
            .offset
            .checked_add((layout.count - 1) * layout.stride + layout.width)
            .ok_or(ZeroCopyError::OutOfBounds {
                field: "layout",
                index: layout.offset,
                limit: buffer.len(),
            })?;
        if last > buffer.len() {
            return Err(ZeroCopyError::OutOfBounds {
                field: "layout",
                index: last,
                limit: buffer.len(),
            });
        }
        Ok(Self { buffer, layout })
    }

    pub fn owner(&self) -> &'a MirrorBuffer {
        self.buffer
    }

    pub fn layout(&self) -> FieldLayout {
        self.layout
    }

    /// 连续字段：返回指向原缓冲的切片（零拷贝）。
    pub fn field(&self) -> Result<&'a [f32], ZeroCopyError> {
        if self.layout.stride != self.layout.width {
            return Err(ZeroCopyError::OutOfBounds {
                field: "field",
                index: 0,
                limit: self.layout.count,
            });
        }
        let end = self.layout.offset + self.layout.count * self.layout.width;
        Ok(&self.buffer.data()[self.layout.offset..end])
    }

    /// 跨步字段逐元素视图：每个元素是指向原缓冲的切片（零拷贝）。
    pub fn elements(&self) -> impl Iterator<Item = &'a [f32]> + '_ {
        let data = self.buffer.data();
        let layout = self.layout;
        (0..layout.count).map(move |element| {
            let start = layout.offset + element * layout.stride;
            &data[start..start + layout.width]
        })
    }

    /// 视图元素个数（测试用；不复制）。
    pub fn count(&self) -> usize {
        self.layout.count
    }
}

/// 显式复制路径（对照用：证明零拷贝视图与复制结果一致）。
pub fn copy_field(buffer: &MirrorBuffer, layout: FieldLayout) -> Vec<f32> {
    let mut output = Vec::with_capacity(layout.count * layout.width);
    for element in 0..layout.count {
        let start = layout.offset + element * layout.stride;
        output.extend_from_slice(&buffer.data()[start..start + layout.width]);
    }
    output
}

#[cfg(test)]
mod tests;
