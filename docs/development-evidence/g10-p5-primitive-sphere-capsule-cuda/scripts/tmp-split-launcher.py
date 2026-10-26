import io, re

root = r'D:\Rust\mjwarp-rs'
main_path = root + r'\tests\g10_p5_nvrtc_launcher.rs'
support_path = root + r'\tests\support\launcher_compare_support.rs'

lines = io.open(main_path, encoding='utf-8').read().splitlines()
# 1-based boundaries: keep 1..18 (imports), 19 const CASES, 20 blank, skip 21 FOUR_ULP const,
# keep 22..143 (readers/meta/build_meta), move 144..768 (CaseReport..proof test), keep 770..830 (GPU test).
head = lines[:18]
const_cases = lines[18:20]
readers = lines[20:143]
moved = lines[143:768]
gpu_test = lines[769:]

moved_text = '\n'.join(moved)
# expose compare_case / CaseReport / fields / new / print to the parent test module
moved_text = moved_text.replace('struct CaseReport {', 'pub(crate) struct CaseReport {', 1)
struct_start = moved_text.index('pub(crate) struct CaseReport {')
struct_end = moved_text.index('\n}', struct_start)
struct_block = moved_text[struct_start:struct_end]
struct_block_pub = re.sub(r'\n    ([a-z_]+):', r'\n    pub(crate) \1:', struct_block)
moved_text = moved_text[:struct_start] + struct_block_pub + moved_text[struct_end:]
moved_text = moved_text.replace('    fn new(case: &str', '    pub(crate) fn new(case: &str', 1)
moved_text = moved_text.replace('    fn print(&self,', '    pub(crate) fn print(&self,', 1)
moved_text = moved_text.replace('fn compare_case(', 'pub(crate) fn compare_case(', 1)

support_header = '''//! launcher 回归比对的共享实现（行序归一化 + 逐字段检查器）。
//!
//! - 行序不作判据：先按 `worldid` 分组、组内按 G5 `sort_contacts` 键 `dist → pos[0..2] →
//!   geomcollisionid` 稳定排序，再逐字段走原检查器（判据/容差/字段集未变）。
//! - 本模块由 `tests/g10_p5_nvrtc_launcher.rs` 以 `#[path = "support/launcher_compare_support.rs"]`
//!   引入；拆分原因是单文件 500 行上限（normify size gate）。
use std::path::Path;

use mjwarp_rs::runtime::collision_gpu::{CollisionGpuOutput, CollisionKernel};

use super::{cases_root, read_efc_width, read_f32, read_i32};

const FOUR_ULP_AT_ONE: f32 = 4.0 * (1.0 / 8_388_608.0);

'''
io.open(support_path, 'w', encoding='utf-8', newline='\n').write(
    support_header + moved_text + '\n'
)

module_decl = [
    '',
    '#[path = "support/launcher_compare_support.rs"]',
    'mod launcher_compare;',
    '',
    'use launcher_compare::compare_case;',
]
io.open(main_path, 'w', encoding='utf-8', newline='\n').write(
    '\n'.join(head + module_decl + const_cases + readers + gpu_test) + '\n'
)
print('split done; main lines=', len(head + module_decl + const_cases + readers + gpu_test))
