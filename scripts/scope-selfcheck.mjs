//! 提交前机械自查：把「当前工作树相对 HEAD 的改动路径」与任务 JSON 声明的 `schedule.files` 做集合差。
//!
//! 用法：`node scripts/scope-selfcheck.mjs <task-id> [--staged-only]`
//! 退出码：0 = 全部改动都在声明范围内；1 = 存在越界路径（或任务不存在）；2 = 用法错误。
//!
//! 设计要点（来自 2026-10-26/27 两次 scope 事故）：
//! - 目录 scope `src/x/y` 只匹配 `src/x/y/...`，**不**匹配父文件 `src/x/y.rs`——父文件必须单独写；
//! - 忽略 `.normify/`（任务记录与门禁 runs 天然会变）与 `target/`（构建产物）；
//! - 未跟踪新文件也算改动（`git status --porcelain` 的 `??`），避免「新文件不写进 files」漏检。

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';

const args = process.argv.slice(2);
const taskId = args[0];
const stagedOnly = args.includes('--staged-only');
if (!taskId) {
  console.error('用法: node scripts/scope-selfcheck.mjs <task-id> [--staged-only]');
  process.exit(2);
}

const repo = execFileSync('git', ['rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
const taskPath = `${repo}/.normify/changes/${taskId}.json`;
let task;
try {
  task = JSON.parse(readFileSync(taskPath, 'utf8'));
} catch (error) {
  console.error(`无法读取任务 JSON: ${taskPath} (${error.message})`);
  process.exit(1);
}
const scope = (task.schedule && task.schedule.files) || [];
if (scope.length === 0) {
  console.error(`任务 ${taskId} 未声明 schedule.files`);
  process.exit(1);
}

const ignorePrefixes = ['.normify/', 'target/'];
function ignored(path) {
  return ignorePrefixes.some((prefix) => path.startsWith(prefix));
}

function matchesScope(path) {
  return scope.some((entry) => {
    const normalized = entry.replace(/\\/g, '/');
    if (path === normalized) return true;
    if (normalized.endsWith('/')) return path.startsWith(normalized);
    return path.startsWith(`${normalized}/`);
  });
}

const statusArgs = ['status', '--porcelain'];
if (stagedOnly) statusArgs.push('--untracked-files=no');
const lines = execFileSync('git', statusArgs, { cwd: repo, encoding: 'utf8' })
  .split('\n')
  .filter((line) => line.trim().length > 0);

const changed = new Set();
for (const line of lines) {
  if (stagedOnly) {
    const index = line[0];
    if (index === ' ' || index === '?') continue;
  }
  let path = line.slice(3);
  const arrow = path.indexOf(' -> ');
  if (arrow >= 0) path = path.slice(arrow + 4);
  path = path.replace(/^"|"$/g, '').replace(/\\/g, '/');
  if (ignored(path)) continue;
  changed.add(path);
}

const inside = [];
const outside = [];
for (const path of [...changed].sort()) {
  if (matchesScope(path)) inside.push(path);
  else outside.push(path);
}

console.log(`任务: ${taskId}`);
console.log(`声明范围: ${scope.join(', ')}`);
console.log(`改动路径: ${changed.size}（范围内 ${inside.length} / 越界 ${outside.length}）`);
for (const path of inside) console.log(`  [ok] ${path}`);
for (const path of outside) console.log(`  [越界] ${path}`);

if (outside.length > 0) {
  console.error('\n越界路径存在：先把它们写进任务 files（父文件与目录要分别写）或移出本片，再提交。');
  process.exit(1);
}
console.log('\nOK: 所有改动都在任务声明范围内。');
