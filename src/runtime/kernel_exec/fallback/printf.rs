//! L.3 退路：内核内格式化打印 → 状态缓冲回写 + 宿主打印。
//!
//! `kernel_printf=false` 时，打印请求写入**宿主侧状态缓冲**（一条记录一行），由宿主读出/
//! 打印；`true` 时走原生 printf 通道（P1-c 参考实现用 `native_log` 表示）。两条路径都不
//! 分配设备内存、不等待队列。

use crate::runtime::kernel_exec::backend::FallbackSet;
use crate::runtime::kernel_exec::queue::Arg;

use super::{FallbackCapability, FallbackTrace, select_native};

/// L.3 退路载体：状态缓冲（宿主侧记录）。
#[derive(Debug, Default)]
pub struct PrintfStatusBuffer {
    records: Vec<String>,
}

impl PrintfStatusBuffer {
    /// 空缓冲。
    pub const fn new() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    /// 全部记录。
    pub fn records(&self) -> &[String] {
        &self.records
    }

    /// 记录条数。
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// 追加一条记录（后端适配层写入状态缓冲）。
    pub fn push(&mut self, record: String) {
        self.records.push(record);
    }

    /// 读走全部记录（宿主打印用）。
    pub fn drain(&mut self) -> Vec<String> {
        std::mem::take(&mut self.records)
    }
}

/// 按顺序把 `{}` 占位符替换成实参文本；多余实参追加在末尾。
///
/// 刻意不做动态格式串解析：避免把内核侧字符串当格式指令执行（格式化注入面）。
pub fn format_record(fmt: &str, args: &[Arg]) -> String {
    let mut out = String::with_capacity(fmt.len() + args.len() * 8);
    let mut remaining = args;
    let mut rest = fmt;
    while let Some(position) = rest.find("{}") {
        out.push_str(&rest[..position]);
        if let Some((arg, tail)) = remaining.split_first() {
            push_arg(&mut out, arg);
            remaining = tail;
        } else {
            out.push_str("{}");
        }
        rest = &rest[position + 2..];
    }
    out.push_str(rest);
    for arg in remaining {
        out.push(' ');
        push_arg(&mut out, arg);
    }
    out
}

fn push_arg(out: &mut String, arg: &Arg) {
    match arg {
        Arg::Buffer { address, offset } => {
            out.push_str(&format!("buffer(0x{:x}+{offset})", address.0));
        }
        Arg::I32(value) => out.push_str(&format!("{value}")),
        Arg::U32(value) => out.push_str(&format!("{value}")),
        Arg::U64(value) => out.push_str(&format!("{value}")),
        Arg::F32(value) => out.push_str(&format!("{value}")),
    }
}

/// L.3 选择：`true` → 原生 printf 通道；`false` → 状态缓冲回写（降级留痕）。
pub fn printf_fallback(
    fallbacks: &FallbackSet,
    status: &mut PrintfStatusBuffer,
    native_log: &mut Vec<String>,
    fmt: &str,
    args: &[Arg],
    trace: &mut FallbackTrace,
) {
    let native = select_native(
        FallbackCapability::KernelPrintf,
        fallbacks,
        "status-buffer",
        trace,
    );
    let record = format_record(fmt, args);
    if native {
        native_log.push(record);
    } else {
        status.records.push(record);
    }
}
