import { readFile } from 'node:fs/promises';
import { resolve, basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(fileURLToPath(new URL('../../', import.meta.url)));
const draftRoots = [
  resolve(repoRoot, 'src/host/windows-host/assets/story-drafts'),
  resolve(repoRoot, 'src/host/linux-host/assets/story-drafts'),
  resolve(repoRoot, 'src/host/android-host/app/src/main/assets/story-drafts'),
];

function normalizeJson(value) {
  return JSON.stringify(value, null, 2);
}

async function readJson(path) {
  const text = await readFile(path, 'utf8');
  return JSON.parse(text.trimStart().replace(/^\uFEFF/u, ''));
}

function validateDraft(draft, fileName) {
  if (!draft || typeof draft !== 'object') {
    throw new Error(`${fileName}: draft must be an object`);
  }
  for (const key of ['templateId', 'language', 'storyTitle', 'summary', 'storyPlot']) {
    if (!String(draft[key] ?? '').trim()) {
      throw new Error(`${fileName}: ${key} must be non-empty`);
    }
  }
  if (!Array.isArray(draft.memoryAnchors) || draft.memoryAnchors.length < 2) {
    throw new Error(`${fileName}: memoryAnchors must contain at least 2 items`);
  }
  if (!Array.isArray(draft.elementGroups) || draft.elementGroups.length !== 8) {
    throw new Error(`${fileName}: elementGroups must contain exactly 8 items`);
  }
  if (!Array.isArray(draft.nodes) || draft.nodes.length !== 24) {
    throw new Error(`${fileName}: nodes must contain exactly 24 items`);
  }
  const uniqueQuestions = new Set();
  draft.nodes.forEach((node, index) => {
    if (!String(node.nodeId ?? '').trim()) {
      throw new Error(`${fileName}: nodes[${index}].nodeId must be non-empty`);
    }
    if (!String(node.question ?? '').trim()) {
      throw new Error(`${fileName}: nodes[${index}].question must be non-empty`);
    }
    uniqueQuestions.add(String(node.question));
    if (!String(node.canonicalAnswerLocalOnly ?? '').trim()) {
      throw new Error(`${fileName}: nodes[${index}].canonicalAnswerLocalOnly must be non-empty`);
    }
    if (node.recommendedSelectionMode !== 'multi_select') {
      throw new Error(`${fileName}: nodes[${index}].recommendedSelectionMode must be multi_select`);
    }
    if (!Array.isArray(node.acceptedAnswersLocalOnly) || node.acceptedAnswersLocalOnly.length < 1) {
      throw new Error(`${fileName}: nodes[${index}].acceptedAnswersLocalOnly must contain at least 1 item`);
    }
    if (!Array.isArray(node.answerOptionsLocalOnly) || node.answerOptionsLocalOnly.length < 2 || node.answerOptionsLocalOnly.length > 9) {
      throw new Error(`${fileName}: nodes[${index}].answerOptionsLocalOnly must contain 2 to 9 items`);
    }
    const correctCount = node.answerOptionsLocalOnly.filter((item) => item?.isCorrect === true).length;
    if (correctCount < 1) {
      throw new Error(`${fileName}: nodes[${index}].answerOptionsLocalOnly must contain at least 1 correct option`);
    }
    if (Number(node.recommendedCorrectCount) !== correctCount) {
      throw new Error(`${fileName}: nodes[${index}].recommendedCorrectCount must match correct option count`);
    }
  });
  if (uniqueQuestions.size !== draft.nodes.length) {
    throw new Error(`${fileName}: node questions must be unique`);
  }
}

async function loadRoot(root) {
  const manifestPath = join(root, 'manifest.json');
  const manifest = await readJson(manifestPath);
  if (!Array.isArray(manifest.items) || manifest.items.length !== 3) {
    throw new Error(`${manifestPath}: manifest.items must contain exactly 3 items`);
  }
  const files = ['manifest.json', ...manifest.items.map((item) => item.fileName)];
  const payloads = new Map();
  for (const file of files) {
    const fullPath = join(root, file);
    const json = await readJson(fullPath);
    if (file !== 'manifest.json') {
      validateDraft(json, `${basename(root)}/${file}`);
    }
    payloads.set(file, json);
  }
  return payloads;
}

const firstRoot = draftRoots[0];
const baseline = await loadRoot(firstRoot);

for (const root of draftRoots.slice(1)) {
  const current = await loadRoot(root);
  for (const [file, baselineJson] of baseline.entries()) {
    const currentJson = current.get(file);
    if (!currentJson) {
      throw new Error(`${root}: missing ${file}`);
    }
    if (normalizeJson(baselineJson) !== normalizeJson(currentJson)) {
      throw new Error(`story draft asset mismatch: ${dirname(root)}/${file}`);
    }
  }
}

console.log(JSON.stringify({
  status: 'ok',
  checkedRoots: draftRoots,
  files: [...baseline.keys()],
}, null, 2));
