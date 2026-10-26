#!/usr/bin/env node
// 字节级自查（与 scripts/scope-selfcheck.mjs、scripts/commit-msg-selfcheck.mjs 同级的提交前机械自查）。
//
// 用法：
//   node scripts/byte-selfcheck.mjs --base <rev> [--paths a,b] [--comments-only] [--allow-bom-change] [--repo <path>]
//   node scripts/byte-selfcheck.mjs --self-test
//
// 检查：
// 1. 改动文件必须是严格 UTF-8 可解码（二进制含 NUL 的文件自动跳过）。
// 2. BOM 状态相对 base 提交不得变化（例如 .ps1 原有 UTF-8 BOM 不能被编辑工具悄悄去掉）；
//    确实要改编码时显式给 --allow-bom-change。
// 3. --comments-only：去掉「注释行 + 空行」后的行序列必须与 base 完全一致
//    （注释行 = 修剪后以 # 或 // 开头；用于证明「只改注释」在字节级成立）。
//
// 退出码：0 通过 / 1 违规 / 2 用法或读取错误。
//
// 来源：2026-10-27 T4 债务 B 落盘（c8753bdb）标称「仅注释改动」，实际去掉了
// scripts/test-windows-g01.ps1 的 UTF-8 BOM，由 9d8db76c 修复；Chief 要求把该检查
// 做成脚本。任务：.normify/changes/2026-10-27-byte-selfcheck-bom-encoding.json

import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';

const BOM = Buffer.from([0xef, 0xbb, 0xbf]);
const COMMENT_PREFIXES = ['#', '//'];

function usage() {
  console.error(
    '用法: node scripts/byte-selfcheck.mjs --base <rev> [--paths a,b] [--comments-only] [--allow-bom-change] [--repo <path>]\n' +
      '      node scripts/byte-selfcheck.mjs --self-test',
  );
}

function parseArgs(argv) {
  const out = { base: null, paths: [], commentsOnly: false, allowBomChange: false, repo: process.cwd(), selfTest: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--self-test') out.selfTest = true;
    else if (arg === '--comments-only') out.commentsOnly = true;
    else if (arg === '--allow-bom-change') out.allowBomChange = true;
    else if (arg === '--base') {
      i += 1;
      if (i >= argv.length) throw new Error('--base 需要一个提交');
      out.base = argv[i];
    } else if (arg === '--paths') {
      i += 1;
      if (i >= argv.length) throw new Error('--paths 需要一个逗号分隔列表');
      out.paths = argv[i].split(',').map((p) => p.trim()).filter(Boolean);
    } else if (arg === '--repo') {
      i += 1;
      if (i >= argv.length) throw new Error('--repo 需要一个路径');
      out.repo = argv[i];
    } else if (arg === '--help' || arg === '-h') out.help = true;
    else throw new Error(`未知参数: ${arg}`);
  }
  return out;
}

export function hasBom(buf) {
  return buf.length >= 3 && buf[0] === BOM[0] && buf[1] === BOM[1] && buf[2] === BOM[2];
}

export function isBinary(buf) {
  return buf.includes(0);
}

function decodeUtf8Strict(buf) {
  return new TextDecoder('utf-8', { fatal: true }).decode(buf);
}

function isCommentLine(line) {
  const trimmed = line.trim();
  if (trimmed.length === 0) return true;
  return COMMENT_PREFIXES.some((prefix) => trimmed.startsWith(prefix));
}

export function nonCommentLines(text) {
  return text
    .replace(/^\uFEFF/, '')
    .split(/\r?\n/)
    .filter((line) => !isCommentLine(line));
}

function gitShow(repo, rev, file) {
  try {
    return execFileSync('git', ['show', `${rev}:${file}`], {
      cwd: repo,
      maxBuffer: 128 * 1024 * 1024,
      stdio: ['ignore', 'pipe', 'ignore'],
    });
  } catch {
    return null;
  }
}

function changedPaths(repo, base) {
  const tracked = execFileSync('git', ['diff', '--name-only', base], { cwd: repo, maxBuffer: 16 * 1024 * 1024 })
    .toString('utf8')
    .split(/\r?\n/)
    .filter(Boolean);
  const staged = execFileSync('git', ['diff', '--cached', '--name-only'], { cwd: repo, maxBuffer: 16 * 1024 * 1024 })
    .toString('utf8')
    .split(/\r?\n/)
    .filter(Boolean);
  const untracked = execFileSync('git', ['ls-files', '--others', '--exclude-standard'], { cwd: repo, maxBuffer: 16 * 1024 * 1024 })
    .toString('utf8')
    .split(/\r?\n/)
    .filter(Boolean);
  return [...new Set([...tracked, ...staged, ...untracked])];
}

function checkFile(repo, base, file, options) {
  const abs = path.join(repo, file);
  if (!fs.existsSync(abs)) return { file, status: 'skip-missing' };
  const current = fs.readFileSync(abs);
  if (isBinary(current)) return { file, status: 'skip-binary' };

  const problems = [];
  try {
    decodeUtf8Strict(current.subarray(hasBom(current) ? 3 : 0));
  } catch (error) {
    problems.push(`不是严格 UTF-8（${error.message}）`);
  }

  const baseBuf = base ? gitShow(repo, base, file) : null;
  if (baseBuf && !isBinary(baseBuf)) {
    const baseBom = hasBom(baseBuf);
    const currentBom = hasBom(current);
    if (baseBom !== currentBom && !options.allowBomChange) {
      problems.push(`BOM 状态变化（base=${baseBom ? '有' : '无'} current=${currentBom ? '有' : '无'}）`);
    }
    if (options.commentsOnly) {
      const baseText = decodeUtf8Strict(baseBuf.subarray(baseBom ? 3 : 0));
      const currentText = decodeUtf8Strict(current.subarray(currentBom ? 3 : 0));
      const before = nonCommentLines(baseText);
      const after = nonCommentLines(currentText);
      const same = before.length === after.length && before.every((line, index) => line === after[index]);
      if (!same) {
        const limit = Math.max(before.length, after.length);
        let firstDiff = -1;
        for (let i = 0; i < limit; i += 1) {
          if (before[i] !== after[i]) {
            firstDiff = i;
            break;
          }
        }
        problems.push(`注释外内容变化（非注释行 ${before.length} -> ${after.length}，首个差异在第 ${firstDiff + 1} 个非注释行）`);
      }
    }
  }

  return { file, status: problems.length === 0 ? 'ok' : 'fail', problems };
}

function runCheck(options) {
  const repo = path.resolve(options.repo);
  if (!options.base) {
    console.error('E 缺少 --base <rev>（相对哪个提交做字节级比较）');
    return 2;
  }
  const paths = options.paths.length > 0 ? options.paths : changedPaths(repo, options.base);
  if (paths.length === 0) {
    console.log('OK: 该 base 之后没有改动文件');
    return 0;
  }
  let failures = 0;
  let checked = 0;
  console.log(`base: ${options.base} | 文件: ${paths.length} | comments_only=${options.commentsOnly} | allow_bom_change=${options.allowBomChange}`);
  for (const file of paths) {
    const result = checkFile(repo, options.base, file, options);
    if (result.status === 'skip-missing' || result.status === 'skip-binary') continue;
    checked += 1;
    if (result.status === 'ok') {
      console.log(`  [ok] ${file}`);
    } else {
      failures += 1;
      console.error(`  [FAIL] ${file}`);
      for (const problem of result.problems) console.error(`         - ${problem}`);
    }
  }
  if (failures === 0) {
    console.log(`OK: ${checked} 个文本文件全部通过（UTF-8 / BOM 状态 / 注释外一致）`);
    return 0;
  }
  console.error(`E: ${failures} 个文件违规`);
  return 1;
}

export function selfTest() {
  const cases = [];
  const withBom = '#!/usr/bin/env pwsh\n# 中文注释\nWrite-Output "ok"\n';
  const withoutBom = '#!/usr/bin/env pwsh\n# 中文注释\nWrite-Output "ok"\n';
  const buf = (text, bom) => Buffer.concat([bom ? BOM : Buffer.alloc(0), Buffer.from(text, 'utf8')]);

  // BOM 判定
  cases.push({ name: 'BOM 检测：有 BOM 识别正确', actual: hasBom(buf(withBom, true)), expected: true });
  cases.push({ name: 'BOM 检测：无 BOM 识别正确', actual: hasBom(buf(withoutBom, false)), expected: false });
  // 注释/空行过滤
  const before = 'line1\n# comment\n\nline2\n// note\nline3\n';
  const afterCommentOnly = 'line1\n# comment\n# extra comment\n\nline2\n// note\n// another note\nline3\n';
  cases.push({
    name: 'comments-only：仅新增注释/空行时非注释行不变',
    actual: JSON.stringify(nonCommentLines(before)) === JSON.stringify(nonCommentLines(afterCommentOnly)),
    expected: true,
  });
  const afterCodeChange = 'line1\n# comment\nline2\nline2-extra\n// note\nline3\n';
  cases.push({
    name: 'comments-only：注释外多一行即失败',
    actual: JSON.stringify(nonCommentLines(before)) === JSON.stringify(nonCommentLines(afterCodeChange)),
    expected: false,
  });
  // 非法 UTF-8
  let invalidUtf8Decoded = true;
  try {
    decodeUtf8Strict(Buffer.from([0xff, 0xfe, 0x00]));
  } catch {
    invalidUtf8Decoded = false;
  }
  cases.push({ name: '严格 UTF-8：非法字节被拒绝', actual: invalidUtf8Decoded, expected: false });

  let failed = 0;
  for (const testCase of cases) {
    const ok = testCase.actual === testCase.expected;
    if (!ok) failed += 1;
    console.log(`${ok ? 'PASS' : 'FAIL'} ${testCase.name}`);
  }
  console.log(failed === 0 ? `OK: self-test ${cases.length}/${cases.length} 通过` : `E: self-test ${failed} 条失败`);
  return failed === 0 ? 0 : 1;
}

function main() {
  let options;
  try {
    options = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(`E ${error.message}`);
    usage();
    return 2;
  }
  if (options.help) {
    usage();
    return 0;
  }
  if (options.selfTest) return selfTest();
  return runCheck(options);
}

process.exitCode = main();
