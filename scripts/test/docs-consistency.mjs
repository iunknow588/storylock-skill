import assert from 'node:assert/strict';
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  ChallengeSigningAuthorizationSkill,
  LocalPasswordFillSkill,
  LoginAuthorizationSkill,
  SignatureAuthorizationSkill,
  SigningAuthorizationSkill,
  StoryDraftAssistSkill,
  StoryRefineAssistSkill,
  StrengthReviewSkill,
} from '../../src/storylock-skill-engine/index.js';

const root = fileURLToPath(new URL('../../', import.meta.url));
const docsRoot = join(root, 'docs');
const srcRoot = join(root, 'src');

const historicalContexts = [
  /不再/u,
  /不应/u,
  /不能/u,
  /不把/u,
  /不要/u,
  /避免/u,
  /检查/u,
  /收敛/u,
  /移除/u,
  /删除/u,
  /废弃/u,
  /不再作为当前设计基线/u,
  /不再作为当前主线/u,
  /不再适合当前文档口径/u,
  /当前已废弃/u,
  /历史/u,
  /旧接口/u,
  /old interface/i,
  /no longer part of the current design baseline/i,
  /no longer in current mainline/i,
  /no longer/i,
  /currently deprecated/i,
  /deprecated/i,
  /removed/i,
  /must not/i,
  /does not/i,
  /do not/i,
  /avoid/i,
  /check/i,
  /converged/i,
  /historical/i,
  /archive/i,
];

const checks = [
  {
    pattern: /requestChallengeSign/u,
    allowHistorical: true,
    message: '`requestChallengeSign` must only appear as a deprecated or historical interface',
  },
  {
    pattern: /requestStory(Read|Write)/u,
    allowHistorical: true,
    message: '`requestStoryRead` and `requestStoryWrite` must only appear as deprecated or historical interfaces',
  },
  {
    pattern: /第二层.*(故事读取|故事写回|故事读写)|Layer 2.*(story reading|story writing|read\/write)/iu,
    allowHistorical: true,
    message: 'Layer 2 must not be described as the current story read/write layer',
  },
];

function walk(dir, output = []) {
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const fullPath = join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === 'node_modules' || entry.name === '.git') {
        continue;
      }
      if (relative(root, fullPath).replaceAll('\\', '/') === 'docs/management/back') {
        continue;
      }
      walk(fullPath, output);
    } else if (entry.name.endsWith('.md') || entry.name.endsWith('.txt') || entry.name.endsWith('.json')) {
      output.push(fullPath);
    }
  }
  return output;
}

function lineHasHistoricalContext(lines, index) {
  const start = Math.max(0, index - 3);
  const end = Math.min(lines.length, index + 4);
  const context = lines.slice(start, end).join('\n');
  return historicalContexts.some((pattern) => pattern.test(context));
}

function assertNoStaleCurrentLanguage(files) {
  const violations = [];
  for (const file of files) {
    const text = readFileSync(file, 'utf8');
    const lines = text.split(/\r?\n/u);
    lines.forEach((line, index) => {
      for (const check of checks) {
        if (!check.pattern.test(line)) {
          continue;
        }
        if (check.allowHistorical && lineHasHistoricalContext(lines, index)) {
          continue;
        }
        violations.push({
          file: relative(root, file),
          line: index + 1,
          message: check.message,
          text: line.trim(),
        });
      }
    });
  }

  assert.equal(
    violations.length,
    0,
    violations.map((item) => `${item.file}:${item.line} ${item.message}\n  ${item.text}`).join('\n'),
  );
}

function assertAccessSkillDescription() {
  const skillPath = join(srcRoot, 'storylock-local-story-access-skill', 'SKILL.md');
  const text = readFileSync(skillPath, 'utf8');
  const description = text.match(/^description:\s*(.+)$/mu)?.[1] ?? '';
  assert.ok(description, 'local access SKILL.md must include a description');
  assert.doesNotMatch(description, /read access|write access|protected story object access/i);
  assert.match(description, /object strength policy/i);
  assert.match(description, /grid verification/i);
  assert.match(description, /local authorization/i);
}

function assertWorkspaceReadmeMatchesCurrentLayout() {
  const readmePath = join(root, 'README.md');
  const text = readFileSync(readmePath, 'utf8');
  assert.match(text, /StoryLock Skill Workspace/);
  assert.match(text, /src\/storylock-local-story-processing-skill/);
  assert.match(text, /src\/storylock-local-story-access-skill/);
  assert.match(text, /src\/storylock-remote-gateway-skill/);
  assert.match(text, /src\/storylock-skill-engine/);
  assert.match(text, /Node\.js `>=22\.0\.0`/);
}

function assertEnglishDesignBacklogIsExplicit() {
  const enRoot = join(docsRoot, 'design', 'en');
  const maybeHistorical = [
    'storylock_skill_pharos_alignment_analysis.md',
    'storylock_skill_positioning_and_boundary.md',
    'storylock_skill_positioning_and_boundary_analysis.md',
    'storylock_story_skill_feasibility_analysis.md',
    'storylock_object_access_policy.md',
  ];
  const existing = maybeHistorical.filter((name) => {
    try {
      return statSync(join(enRoot, name)).isFile();
    } catch {
      return false;
    }
  });

  const readme = readFileSync(join(enRoot, 'README.md'), 'utf8');
  for (const fileName of existing) {
    assert.match(
      readme,
      new RegExp(fileName.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')),
      `en/README.md must explicitly classify ${fileName} as current, archive, or historical`,
    );
  }
}

function assertSkillEngineExportsMatchDocs() {
  const exports = {
    ChallengeSigningAuthorizationSkill,
    LocalPasswordFillSkill,
    LoginAuthorizationSkill,
    SignatureAuthorizationSkill,
    SigningAuthorizationSkill,
    StoryDraftAssistSkill,
    StoryRefineAssistSkill,
    StrengthReviewSkill,
  };
  for (const [name, exported] of Object.entries(exports)) {
    assert.equal(typeof exported, 'function', `storylock-skill-engine must export ${name}`);
  }
}

function assertRemoteGatewayMainSurface() {
  const indexPath = join(srcRoot, 'storylock-remote-gateway-skill', 'index.js');
  const text = readFileSync(indexPath, 'utf8');
  const methods = [...text.matchAll(/^\s+async\s+(request[A-Za-z0-9_]+)\(/gmu)].map((match) => match[1]);
  assert.deepEqual(
    methods,
    ['requestSignature', 'requestPasswordFill'],
    'remote gateway must keep requestSignature/requestPasswordFill as the only main request methods',
  );
  assert.doesNotMatch(text, /requestChallengeSign/u, 'remote gateway must not reintroduce requestChallengeSign');
}

const files = [
  ...walk(docsRoot),
  join(root, 'README.md'),
  join(srcRoot, 'storylock-local-story-access-skill', 'SKILL.md'),
  join(srcRoot, 'storylock-local-story-access-skill', 'package.json'),
];

assertNoStaleCurrentLanguage(files);
assertAccessSkillDescription();
assertWorkspaceReadmeMatchesCurrentLayout();
assertEnglishDesignBacklogIsExplicit();
assertSkillEngineExportsMatchDocs();
assertRemoteGatewayMainSurface();

console.log(JSON.stringify({
  status: 'passed',
  filesChecked: files.length,
}, null, 2));
