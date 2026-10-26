#!/usr/bin/env node
// commit message 禁词自查（与 scripts/scope-selfcheck.mjs 同级的提交前机械自查）。
//
// 用法：
//   node scripts/commit-msg-selfcheck.mjs <task-id> <message-file> [--repo <path>]
//   node scripts/commit-msg-selfcheck.mjs --self-test
//
// 规则：
// - 优先读 <repo>/.normify/changes/<task-id>.json 的 checks[kind=forbid_claim].terms；
//   任务无该检查（或文件不存在）时回退内置默认词表，并打印提示。
// - 扫描提交信息全文（标题 + 正文 + trailer），命中即打印「行号 + 命中词」并退出码 1。
// - 零命中打印 OK 并退出码 0。
// - 退出码 2 = 用法/读取错误（不是命中）。
//
// 来源：2026-10-27 M3 提交时 commit message 出现「精度达标」字面词，被 commit-msg hook 的
// forbid_claim 拒绝；Chief 要求把该检查做成脚本，不靠人眼。任务：
// .normify/changes/2026-10-27-commit-msg-forbid-selfcheck.json

import fs from 'node:fs';
import path from 'node:path';

const DEFAULT_TERMS = [
  'G10已实现',
  '完整物理通过',
  '双平台通过',
  '上游完全等价',
  '原生放行',
  '精度达标',
];

function usage() {
  console.error(
    '用法: node scripts/commit-msg-selfcheck.mjs <task-id> <message-file> [--repo <path>]\n' +
      '      node scripts/commit-msg-selfcheck.mjs --self-test',
  );
}

function parseArgs(argv) {
  const out = { taskId: null, messageFile: null, repo: process.cwd(), selfTest: false };
  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--self-test') {
      out.selfTest = true;
    } else if (arg === '--repo') {
      i += 1;
      if (i >= argv.length) throw new Error('--repo 需要一个路径');
      out.repo = argv[i];
    } else if (arg === '--help' || arg === '-h') {
      out.help = true;
    } else if (out.taskId === null) {
      out.taskId = arg;
    } else if (out.messageFile === null) {
      out.messageFile = arg;
    } else {
      throw new Error(`多余参数: ${arg}`);
    }
  }
  return out;
}

/** 从任务 JSON 读 forbid_claim 词表；返回 { terms, source }。 */
export function termsForTask(repo, taskId) {
  const taskFile = path.join(repo, '.normify', 'changes', `${taskId}.json`);
  if (!fs.existsSync(taskFile)) {
    return { terms: DEFAULT_TERMS, source: `默认词表（任务文件不存在: ${taskFile}）` };
  }
  let doc;
  try {
    doc = JSON.parse(fs.readFileSync(taskFile, 'utf8').replace(/^\uFEFF/, ''));
  } catch (error) {
    return { terms: DEFAULT_TERMS, source: `默认词表（任务文件不可解析: ${error.message}）` };
  }
  const checks = Array.isArray(doc.checks) ? doc.checks : [];
  const forbid = checks.filter((check) => check && check.kind === 'forbid_claim');
  const terms = [...new Set(forbid.flatMap((check) => (Array.isArray(check.terms) ? check.terms : [])))];
  if (terms.length === 0) {
    return { terms: DEFAULT_TERMS, source: `默认词表（任务 ${taskId} 无 forbid_claim 检查）` };
  }
  return { terms, source: `任务 ${taskId} 的 forbid_claim 词表（${terms.length} 条）` };
}

/** 扫描一段文本，返回命中行 [{ line, term, text }]。 */
export function scanMessage(text, terms) {
  const lines = text.replace(/^\uFEFF/, '').split(/\r?\n/);
  const hits = [];
  for (let index = 0; index < lines.length; index += 1) {
    for (const term of terms) {
      if (term.length > 0 && lines[index].includes(term)) {
        hits.push({ line: index + 1, term, text: lines[index] });
      }
    }
  }
  return hits;
}

function runCheck(taskId, messageFile, repo) {
  if (!fs.existsSync(messageFile)) {
    console.error(`E 提交信息文件不存在: ${messageFile}`);
    return 2;
  }
  const { terms, source } = termsForTask(repo, taskId);
  const text = fs.readFileSync(messageFile, 'utf8');
  const hits = scanMessage(text, terms);
  console.log(`任务: ${taskId}`);
  console.log(`词表来源: ${source}`);
  console.log(`提交信息: ${messageFile}（${text.split(/\r?\n/).length} 行）`);
  if (hits.length === 0) {
    console.log(`OK: 零命中（${terms.length} 条禁词）`);
    return 0;
  }
  console.error(`E 命中 ${hits.length} 处禁词（commit-msg hook 会拒绝该提交说明）:`);
  for (const hit of hits) {
    console.error(`  行 ${hit.line}: 「${hit.term}」 -> ${hit.text.trim()}`);
  }
  console.error('改写措辞后再提交（如「不宣称精度目标达成」代替字面禁词）。');
  return 1;
}

function selfTest() {
  const cases = [];
  const terms = DEFAULT_TERMS;
  const clean = '功能(碰撞): 示例标题\n\n- 不宣称精度目标已达成\n';
  const dirty = '功能(碰撞): 示例标题\n\n- 本片精度达标，窗口通过\n';
  const crlf = '功能(碰撞): 示例\r\n\r\n- 上游完全等价\r\n';
  const bom = '\uFEFF功能(碰撞): 示例标题\n\n- 无问题\n';

  cases.push({
    name: '干净消息 → 0 命中',
    hits: scanMessage(clean, terms).length,
    expected: 0,
  });
  cases.push({
    name: '含「精度达标」→ 恰好 1 处',
    hits: scanMessage(dirty, terms).length,
    expected: 1,
  });
  cases.push({
    name: 'CRLF 消息含「上游完全等价」→ 恰好 1 处',
    hits: scanMessage(crlf, terms).length,
    expected: 1,
  });
  cases.push({
    name: 'BOM 消息干净 → 0 命中',
    hits: scanMessage(bom, terms).length,
    expected: 0,
  });

  let failed = 0;
  for (const testCase of cases) {
    const ok = testCase.hits === testCase.expected;
    if (!ok) failed += 1;
    console.log(`${ok ? 'PASS' : 'FAIL'} ${testCase.name}（hits=${testCase.hits}, expected=${testCase.expected}）`);
  }
  console.log(failed === 0 ? 'OK: self-test 4/4 通过' : `E: self-test ${failed} 条失败`);
  return failed === 0 ? 0 : 1;
}

function main() {
  let args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (error) {
    console.error(`E ${error.message}`);
    usage();
    return 2;
  }
  if (args.help) {
    usage();
    return 0;
  }
  if (args.selfTest) return selfTest();
  if (!args.taskId || !args.messageFile) {
    usage();
    return 2;
  }
  return runCheck(args.taskId, args.messageFile, args.repo);
}

process.exitCode = main();
