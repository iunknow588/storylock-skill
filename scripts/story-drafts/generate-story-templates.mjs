import { mkdirSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = resolve(fileURLToPath(new URL('../../', import.meta.url)));

const draftRoots = [
  join(repoRoot, 'src/host/windows-host/assets/story-drafts'),
  join(repoRoot, 'src/host/linux-host/assets/story-drafts'),
  join(repoRoot, 'src/host/android-host/app/src/main/assets/story-drafts'),
];

const templateRoot = join(repoRoot, 'src/host/windows-host/assets/story-template-directories');

const z = {
  shou: '\u5b88\u682a\u5f85\u5154',
  zhizi: '\u667a\u5b50\u7591\u90bb',
  q: '\u95ee\u9898',
  multi: '\u6b64\u6a21\u677f\u5df2\u6309 24 \u4e2a\u95ee\u9898\u6784\u9020\uff0c\u6240\u6709\u9898\u76ee\u5747\u4e3a\u591a\u9009\u3002',
  readmeUse: '\u5c06 story-template.json \u4f5c\u4e3a StoryLock \u6545\u4e8b\u6a21\u677f\u5bfc\u5165\u6216\u53c2\u8003\u4f7f\u7528\u3002',
};

const groups = ['time', 'place', 'person', 'object', 'event', 'reaction', 'choice', 'result'];

function option(text, isCorrect) {
  return { text, isCorrect };
}

function numberVariants(text) {
  const match = text.match(/\d+/u);
  if (!match) {
    return [];
  }
  const number = Number(match[0]);
  if (!Number.isFinite(number)) {
    return [];
  }
  return [-2, -1, 1, 2]
    .map((delta) => text.replace(match[0], String(number + delta)))
    .filter((item) => item !== text);
}

const simpleVariantMap = new Map([
  ['\u5b8b\u56fd', ['\u536b\u56fd', '\u9c81\u56fd', '\u90d1\u56fd']],
  ['\u4e1c\u5468\u65f6\u671f', ['\u6625\u79cb\u65f6\u671f', '\u6218\u56fd\u65f6\u671f', '\u897f\u5468\u672b\u671f']],
  ['\u6218\u56fd\u65f6\u671f', ['\u4e1c\u5468\u65f6\u671f', '\u6218\u56fd\u4e2d\u671f', '\u6625\u79cb\u672b\u671f']],
  ['\u76db\u590f', ['\u521d\u590f', '\u590f\u672b', '\u6625\u672b']],
  ['\u521d\u79cb', ['\u6df1\u79cb', '\u79cb\u672b', '\u590f\u672b']],
  ['\u9752\u77f3\u6751', ['\u77f3\u6865\u6751', '\u9752\u6eaa\u6751', '\u4e1c\u77f3\u6751']],
  ['\u7530\u95f4', ['\u7530\u8fb9', '\u83dc\u5730', '\u6751\u897f\u7530\u57c2']],
  ['\u8001\u68a8\u6811\u4e0b', ['\u8001\u69d0\u6811\u4e0b', '\u8001\u6851\u6811\u4e0b', '\u6751\u53e3\u6811\u4e0b']],
  ['\u90fd\u57ce\u90ca\u5916', ['\u57ce\u5317\u90ca\u5916', '\u57ce\u5357\u90ca\u5916', '\u57ce\u4e1c\u90ca\u5916']],
  ['\u5468\u8001\u7237\u5bb6', ['\u5b59\u8001\u7237\u5bb6', '\u674e\u5bb6\u5927\u9662', '\u5bcc\u6237\u5bb6\u4e2d']],
  ['\u9662\u5899\u8fb9', ['\u540e\u9662\u5899\u8fb9', '\u5927\u95e8\u8fb9', '\u9662\u95e8\u65c1']],
  ['\u963f\u798f', ['\u963f\u8d35', '\u963f\u987a', '\u5c0f\u798f']],
  ['\u5468\u8001\u7237', ['\u94b1\u8001\u7237', '\u5b59\u5458\u5916', '\u674e\u4e1c\u5bb6']],
  ['\u5218\u4e09', ['\u5f20\u4e09', '\u674e\u56db', '\u738b\u4e8c']],
  ['\u8d75\u9ebb\u5b50', ['\u94b1\u9ebb\u5b50', '\u738b\u7626\u5b50', '\u9648\u5077\u513f']],
  ['\u5468\u6587', ['\u5468\u660e', '\u5468\u5b89', '\u5c0f\u6587']],
  ['\u5154\u5b50\u649e\u6811', ['\u9e1f\u513f\u843d\u7f51', '\u9e7f\u8dd1\u8fc7\u7530', '\u98ce\u5439\u843d\u679c']],
  ['\u72d0\u72f8\u8ffd\u8d76', ['\u72d7\u8ffd\u91ce\u9e21', '\u9e70\u8ffd\u5c0f\u9e1f', '\u4eba\u58f0\u60ca\u52a8']],
  ['\u9662\u5899\u635f\u574f', ['\u95e8\u95e9\u635f\u574f', '\u7a97\u6237\u677e\u52a8', '\u56f4\u680f\u5012\u584c']],
  ['\u5927\u96e8\u51b2\u574f', ['\u96e8\u6c34\u6d78\u574f', '\u5927\u98ce\u5439\u574f', '\u79ef\u6c34\u6ce1\u574f']],
  ['Fantasia', ['Valoria', 'Florentia', 'Aurelia']],
  ['city streets', ['royal avenue', 'market streets', 'capital square']],
  ['the palace', ['the palace gate', 'the council hall', 'the royal hall']],
  ["the emperor's realm", ['the royal realm', 'the capital realm', "the monarch's court"]],
  ['Lorenzo', ['Leonardo', 'Augusto', 'Renato']],
  ['Marco and Pietro', ['Mario and Paolo', 'Marco and Paolo', 'Pietro and Carlo']],
  ['Alberto (old minister)', ['Alberto (senior advisor)', 'Roberto (old minister)', 'Alberto (court elder)']],
  ['Bruno (young officer)', ['Bruno (junior officer)', 'Carlo (young officer)', 'Bruno (court officer)']],
]);

function simpleTextVariants(text) {
  const mapped = simpleVariantMap.get(text) ?? [];
  const numeric = numberVariants(text);
  if (mapped.length || numeric.length) {
    return [...mapped, ...numeric];
  }
  const generic = [
    `${text}\u9644\u8fd1`,
    `${text}\u4e4b\u540e`,
    `${text}\u4e4b\u524d`,
  ];
  const englishGeneric = [
    `${text} nearby`,
    `${text} earlier`,
    `${text} later`,
  ];
  return [...mapped, ...numeric, ...(text.match(/[A-Za-z]/u) ? englishGeneric : generic)];
}

function makeNodes(titlePrefix, questions, editorNotes, negativePool, slotDistractors = [], slotFallbacks = []) {
  return questions.map((item, index) => {
    const correct = item.answers.map((answer) => option(answer, true));
    const distractorPool = [
      ...item.answers.flatMap(simpleTextVariants),
      ...(slotDistractors[index] ?? []),
      ...(slotFallbacks[index] ?? []),
      ...negativePool,
    ];
    const falseOptions = distractorPool
      .filter((candidate) => !item.answers.includes(candidate))
      .filter((candidate, candidateIndex, candidates) => candidates.indexOf(candidate) === candidateIndex)
      .slice(0, Math.max(2, 9 - correct.length))
      .map((answer) => option(answer, false));
    const answerOptionsLocalOnly = [...correct, ...falseOptions].slice(0, 9);
    return {
      nodeId: `question-${String(index + 1).padStart(2, '0')}`,
      title: `${titlePrefix} ${String(index + 1).padStart(2, '0')}`,
      elementId: groups[index % groups.length],
      question: item.question,
      recommendedSelectionMode: 'multi_select',
      recommendedCorrectCount: correct.length,
      candidatePoolSize: answerOptionsLocalOnly.length,
      recallPriority: index < 4 ? 'high' : index < 16 ? 'normal' : 'review',
      verifyPolicy: 'caseInsensitive + trim',
      editorNotes,
      canonicalAnswerLocalOnly: item.answers[0],
      acceptedAnswersLocalOnly: item.answers,
      answerOptionsLocalOnly,
    };
  });
}

const shouQuestions = [
  { question: '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u516c\u5143\u524d332\u5e74', '\u4e1c\u5468\u65f6\u671f', '4\u67085\u65e5', '\u76db\u590f'] },
  { question: '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u65b9\uff1f', answers: ['\u5b8b\u56fd', '\u9752\u77f3\u6751', '\u7530\u95f4', '\u8001\u68a8\u6811\u4e0b'] },
  { question: '\u6545\u4e8b\u4e2d\u6700\u4e3b\u8981\u7684\u4eba\u7269\u662f\u8c01\uff1f', answers: ['\u963f\u798f'] },
  { question: '\u8fd9\u4e2a\u4eba\u7269\u7684\u8eab\u4efd\u6216\u804c\u4e1a\u662f\u4ec0\u4e48\uff1f', answers: ['\u519c\u592b', '\u79cd\u7530\u4eba'] },
  { question: '\u8fd9\u4e2a\u4eba\u7269\u6709\u4ec0\u4e48\u91cd\u8981\u7279\u5f81\u6216\u4e60\u60ef\uff1f', answers: ['\u5531\u6b4c', '\u54fc\u5c0f\u8c03'] },
  { question: '\u6545\u4e8b\u4e2d\u8fd8\u6709\u54ea\u4e9b\u91cd\u8981\u4eba\u7269\u6216\u89d2\u8272\uff1f', answers: ['\u8001\u9ec4\u72d7', '\u9ec4\u72d7'] },
  { question: '\u8fd9\u4e9b\u5e2e\u624b\u6216\u6b21\u8981\u89d2\u8272\u7684\u8eab\u4efd\u662f\u4ec0\u4e48\uff1f', answers: ['\u770b\u95e8\u72d7', '\u966a\u4f34\u8005'] },
  { question: '\u6545\u4e8b\u4e2d\u9020\u6210\u51b2\u7a81\u7684\u4e00\u65b9\u6216\u56e0\u7d20\u662f\u4ec0\u4e48\uff1f', answers: ['\u5218\u4e09', '\u75de\u5b50'] },
  { question: '\u8fd9\u4e00\u65b9\u6216\u56e0\u7d20\u7684\u884c\u4e3a\u7279\u70b9\u662f\u4ec0\u4e48\uff1f', answers: ['\u6e38\u624b\u597d\u95f2', '\u5632\u7b11\u4eba'] },
  { question: '\u4e8b\u4ef6\u7684\u8d77\u56e0\u662f\u4ec0\u4e48\uff1f', answers: ['\u5154\u5b50\u649e\u6811', '\u72d0\u72f8\u8ffd\u8d76'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u708e\u70ed', '\u6674\u6717'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u73af\u5883\u600e\u6837\uff1f', answers: ['\u8749\u9e23\u9635\u9635', '\u5e84\u7a3c\u8302\u76db'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u5927\u559c', '\u9ad8\u5174'] },
  { question: '\u4e8b\u4ef6\u5982\u4f55\u7ee7\u7eed\u53d1\u5c55\uff1f', answers: ['\u653e\u5f03\u8015\u4f5c', '\u6bcf\u65e5\u5b88\u682a'] },
  { question: '\u6545\u4e8b\u4e2d\u6700\u5173\u952e\u7684\u7ed3\u679c\u6216\u573a\u9762\u662f\u4ec0\u4e48\uff1f', answers: ['\u906d\u5230\u5168\u6751\u803b\u7b11', '\u7530\u5730\u8352\u829c'] },
  { question: '\u6545\u4e8b\u7684\u8f6c\u6298\u7531\u4ec0\u4e48\u5f15\u53d1\uff1f', answers: ['\u6bcd\u4eb2\u529d\u544a', '\u5218\u4e09\u5632\u7b11'] },
  { question: '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u6b21\u5e74\u79cb\u5929', '\u4e00\u5e74\u540e'] },
  { question: '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u70b9\uff1f', answers: ['\u7530\u8fb9', '\u8001\u68a8\u6811\u4e0b'] },
  { question: '\u8f6c\u6298\u65f6\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u79cb\u98ce', '\u8427\u745f'] },
  { question: '\u8f6c\u6298\u65f6\u73af\u5883\u600e\u6837\uff1f', answers: ['\u843d\u53f6\u6ee1\u5730', '\u7530\u5730\u8352\u829c'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u6b21\u5e74\u79cb\u5929', '\u6536\u5272\u5b63\u8282'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u65f6\u5730\u70b9\u5728\u54ea\u91cc\uff1f', answers: ['\u8352\u829c\u7684\u7530\u5730', '\u6751\u53e3'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u540e\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u7f9e\u6127', '\u51b3\u5fc3\u52e4\u52b3'] },
  { question: '\u6545\u4e8b\u6700\u540e\u7559\u4e0b\u4e86\u4ec0\u4e48\u542f\u793a\uff1f', answers: ['\u4e0d\u53ef\u4fa5\u5e78', '\u52e4\u52b3\u81f4\u5bcc'] },
];

const zhiziQuestions = [
  { question: '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u516c\u5143\u524d398\u5e74', '\u6218\u56fd\u65f6\u671f', '9\u6708\u521d', '\u521d\u79cb'] },
  { question: '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u65b9\uff1f', answers: ['\u5b8b\u56fd', '\u90fd\u57ce\u90ca\u5916', '\u5468\u8001\u7237\u5bb6', '\u9662\u5899\u8fb9'] },
  { question: '\u6545\u4e8b\u4e2d\u6700\u4e3b\u8981\u7684\u4eba\u7269\u662f\u8c01\uff1f', answers: ['\u5468\u8001\u7237'] },
  { question: '\u8fd9\u4e2a\u4eba\u7269\u7684\u8eab\u4efd\u6216\u804c\u4e1a\u662f\u4ec0\u4e48\uff1f', answers: ['\u5546\u4eba', '\u5bcc\u6237'] },
  { question: '\u8fd9\u4e2a\u4eba\u7269\u6709\u4ec0\u4e48\u91cd\u8981\u7279\u5f81\u6216\u4e60\u60ef\uff1f', answers: ['\u6536\u85cf\u53e4\u73a9', '\u54c1\u8336'] },
  { question: '\u6545\u4e8b\u4e2d\u8fd8\u6709\u54ea\u4e9b\u91cd\u8981\u4eba\u7269\u6216\u89d2\u8272\uff1f', answers: ['\u5468\u6587', '\u513f\u5b50'] },
  { question: '\u8fd9\u4e9b\u5e2e\u624b\u6216\u6b21\u8981\u89d2\u8272\u7684\u8eab\u4efd\u662f\u4ec0\u4e48\uff1f', answers: ['\u8bfb\u4e66\u4eba', '\u5efa\u8bae\u8005'] },
  { question: '\u6545\u4e8b\u4e2d\u9020\u6210\u51b2\u7a81\u7684\u4e00\u65b9\u6216\u56e0\u7d20\u662f\u4ec0\u4e48\uff1f', answers: ['\u8d75\u9ebb\u5b50', '\u60ef\u5077'] },
  { question: '\u8fd9\u4e00\u65b9\u6216\u56e0\u7d20\u7684\u884c\u4e3a\u7279\u70b9\u662f\u4ec0\u4e48\uff1f', answers: ['\u76d7\u8d3c', '\u4e13\u5077\u5bcc\u6237'] },
  { question: '\u4e8b\u4ef6\u7684\u8d77\u56e0\u662f\u4ec0\u4e48\uff1f', answers: ['\u9662\u5899\u635f\u574f', '\u5927\u96e8\u51b2\u574f'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u5927\u96e8', '\u96e8\u505c\u6708\u51fa'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u73af\u5883\u600e\u6837\uff1f', answers: ['\u79ef\u6c34\u904d\u5730', '\u6ce5\u6cde\u4e0d\u582a'] },
  { question: '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u70e6\u8e81', '\u4e0d\u4ee5\u4e3a\u610f'] },
  { question: '\u4e8b\u4ef6\u5982\u4f55\u7ee7\u7eed\u53d1\u5c55\uff1f', answers: ['\u5931\u7a83', '\u79f0\u8d5e\u513f\u5b50', '\u6000\u7591\u90bb\u5c45'] },
  { question: '\u6545\u4e8b\u4e2d\u6700\u5173\u952e\u7684\u7ed3\u679c\u6216\u573a\u9762\u662f\u4ec0\u4e48\uff1f', answers: ['\u771f\u76f8\u5927\u767d', '\u771f\u76d7\u88ab\u6355'] },
  { question: '\u6545\u4e8b\u7684\u8f6c\u6298\u7531\u4ec0\u4e48\u5f15\u53d1\uff1f', answers: ['\u5b98\u5e9c\u7834\u6848', '\u5468\u8001\u7237\u6094\u609f'] },
  { question: '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u4e09\u4e2a\u6708\u540e', '\u51ac\u5929'] },
  { question: '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u70b9\uff1f', answers: ['\u5b98\u5e9c', '\u5468\u8001\u7237\u5bb6'] },
  { question: '\u8f6c\u6298\u65f6\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u5bd2\u98ce\u51db\u51bd', '\u5927\u96ea'] },
  { question: '\u8f6c\u6298\u65f6\u73af\u5883\u600e\u6837\uff1f', answers: ['\u767d\u96ea\u8986\u76d6', '\u5bd2\u51b7'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u51ac\u5929', '\u4e09\u4e2a\u6708\u540e'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u65f6\u5730\u70b9\u5728\u54ea\u91cc\uff1f', answers: ['\u5b59\u4f2f\u5bb6\u95e8\u53e3', '\u5468\u8001\u7237\u5bb6'] },
  { question: '\u6545\u4e8b\u7ed3\u675f\u540e\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u7f9e\u6127', '\u6094\u609f'] },
  { question: '\u6545\u4e8b\u6700\u540e\u7559\u4e0b\u4e86\u4ec0\u4e48\u542f\u793a\uff1f', answers: ['\u4e0d\u53ef\u56e0\u4eba\u5e9f\u8a00', '\u4e0d\u53ef\u56e0\u4eb2\u758f\u8bef\u5224'] },
];

const emperorQuestions = [
  { question: 'When does the story take place?', answers: ['1710', 'early 18th century', 'summer morning', 'bright day'] },
  { question: 'Where does the story take place?', answers: ['Fantasia', 'city streets', 'the palace', "the emperor's realm"] },
  { question: 'Who is the main person in the story?', answers: ['Lorenzo'] },
  { question: 'What is this person\'s role or occupation?', answers: ['Ruler', 'monarch'] },
  { question: 'What important trait or habit does this person have?', answers: ['New clothes', 'splendid attire'] },
  { question: 'Which other important people or roles appear in the story?', answers: ['Alberto (old minister)', 'Bruno (young officer)'] },
  { question: 'What roles or identities do those people have?', answers: ['Minister', 'trusted advisor', 'officer', 'official'] },
  { question: 'What side or factor creates the conflict in the story?', answers: ['Marco and Pietro', 'two swindlers'] },
  { question: 'What behavior or feature describes that side or factor?', answers: ['Swindlers', 'tricksters'] },
  { question: 'What causes the event to begin?', answers: ['The promise of invisible cloth', "emperor's vanity"] },
  { question: 'What is the weather like when the event begins?', answers: ['Sunny', 'bright'] },
  { question: 'What is the setting like when the event begins?', answers: ['Gardens in bloom', 'palace splendor'] },
  { question: 'How does the main person feel when the event begins?', answers: ['Excited', 'proud'] },
  { question: 'How does the event continue to develop?', answers: ['Inspections', 'lying', 'preparation'] },
  { question: 'What is the most important result or scene in the story?', answers: ['The child shouting the truth', "crowd's whisper"] },
  { question: 'What triggers the turning point of the story?', answers: ['The emperor realizing he was naked', 'shame'] },
  { question: 'When does the turning point happen?', answers: ['During the parade', 'later that day'] },
  { question: 'Where does the turning point happen?', answers: ['City streets', 'among the crowd'] },
  { question: 'What is the weather like at the turning point?', answers: ['Sunny', 'warm'] },
  { question: 'What is the setting like at the turning point?', answers: ['Cheering crowds', 'colorful banners'] },
  { question: 'When does the story end?', answers: ['Later that evening', 'after the parade'] },
  { question: 'Where is the story at the end?', answers: ['The palace', "the emperor's chamber"] },
  { question: 'How does the main person feel after the story ends?', answers: ['Foolish', 'humiliated', 'ashamed'] },
  { question: 'What lesson or insight does the story leave behind?', answers: ['Vanity is foolish', 'children speak truth'] },
];

const vagueChineseQuestions = [
  '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f',
  '\u6545\u4e8b\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u65b9\uff1f',
  '\u6545\u4e8b\u4e2d\u6700\u4e3b\u8981\u7684\u4eba\u7269\u662f\u8c01\uff1f',
  '\u8fd9\u4e2a\u4eba\u7269\u7684\u8eab\u4efd\u6216\u804c\u4e1a\u662f\u4ec0\u4e48\uff1f',
  '\u8fd9\u4e2a\u4eba\u7269\u6709\u4ec0\u4e48\u91cd\u8981\u7279\u5f81\u6216\u4e60\u60ef\uff1f',
  '\u6545\u4e8b\u4e2d\u8fd8\u6709\u54ea\u4e9b\u91cd\u8981\u4eba\u7269\u6216\u89d2\u8272\uff1f',
  '\u8fd9\u4e9b\u5e2e\u624b\u6216\u6b21\u8981\u89d2\u8272\u7684\u8eab\u4efd\u662f\u4ec0\u4e48\uff1f',
  '\u6545\u4e8b\u4e2d\u9020\u6210\u51b2\u7a81\u7684\u4e00\u65b9\u6216\u56e0\u7d20\u662f\u4ec0\u4e48\uff1f',
  '\u8fd9\u4e00\u65b9\u6216\u56e0\u7d20\u7684\u884c\u4e3a\u7279\u70b9\u662f\u4ec0\u4e48\uff1f',
  '\u4e8b\u4ef6\u7684\u8d77\u56e0\u662f\u4ec0\u4e48\uff1f',
  '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u5929\u6c14\u5982\u4f55\uff1f',
  '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u73af\u5883\u600e\u6837\uff1f',
  '\u4e8b\u4ef6\u5f00\u59cb\u65f6\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f',
  '\u4e8b\u4ef6\u5982\u4f55\u7ee7\u7eed\u53d1\u5c55\uff1f',
  '\u6545\u4e8b\u4e2d\u6700\u5173\u952e\u7684\u7ed3\u679c\u6216\u573a\u9762\u662f\u4ec0\u4e48\uff1f',
  '\u6545\u4e8b\u7684\u8f6c\u6298\u7531\u4ec0\u4e48\u5f15\u53d1\uff1f',
  '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f',
  '\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u5730\u70b9\uff1f',
  '\u8f6c\u6298\u65f6\u5929\u6c14\u5982\u4f55\uff1f',
  '\u8f6c\u6298\u65f6\u73af\u5883\u600e\u6837\uff1f',
  '\u6545\u4e8b\u7ed3\u675f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f',
  '\u6545\u4e8b\u7ed3\u675f\u65f6\u5730\u70b9\u5728\u54ea\u91cc\uff1f',
  '\u6545\u4e8b\u7ed3\u675f\u540e\u4e3b\u8981\u4eba\u7269\u5fc3\u60c5\u5982\u4f55\uff1f',
  '\u6545\u4e8b\u6700\u540e\u7559\u4e0b\u4e86\u4ec0\u4e48\u542f\u793a\uff1f',
];

const vagueEnglishQuestions = [
  'When does the story take place?',
  'Where does the story take place?',
  'Who is the main person in the story?',
  "What is this person's role or occupation?",
  'What important trait or habit does this person have?',
  'Which other important people or roles appear in the story?',
  'What roles or identities do those people have?',
  'What side or factor creates the conflict in the story?',
  'What behavior or feature describes that side or factor?',
  'What causes the event to begin?',
  'What is the weather like when the event begins?',
  'What is the setting like when the event begins?',
  'How does the main person feel when the event begins?',
  'How does the event continue to develop?',
  'What is the most important result or scene in the story?',
  'What triggers the turning point of the story?',
  'When does the turning point happen?',
  'Where does the turning point happen?',
  'What is the weather like at the turning point?',
  'What is the setting like at the turning point?',
  'When does the story end?',
  'Where is the story at the end?',
  'How does the main person feel after the story ends?',
  'What lesson or insight does the story leave behind?',
];

function withVagueQuestions(questions, vagueQuestions) {
  return questions.map((item, index) => ({
    ...item,
    question: vagueQuestions[index],
  }));
}

const shouDistractors = [
  ['\u516c\u5143\u524d331\u5e74', '\u516c\u5143\u524d333\u5e74', '\u6625\u672b', '4\u67086\u65e5', '\u521d\u590f'],
  ['\u536b\u56fd', '\u77f3\u6865\u6751', '\u83dc\u5730', '\u53e4\u69d0\u6811\u4e0b', '\u6751\u897f\u7530\u57c2'],
  ['\u963f\u8d35', '\u963f\u987a', '\u7530\u5e73', '\u5c0f\u798f', '\u8001\u751f'],
  ['\u4f43\u519c', '\u7267\u4eba', '\u957f\u5de5', '\u79df\u7530\u4eba', '\u6751\u4e2d\u8015\u6237'],
  ['\u5439\u7b1b', '\u8bb2\u7b11\u8bdd', '\u5531\u5c71\u6b4c', '\u6572\u6728\u9c7c', '\u542c\u96e8'],
  ['\u9ed1\u72d7', '\u8001\u7070\u72d7', '\u5c0f\u9ec4\u72d7', '\u6751\u53e3\u571f\u72d7', '\u770b\u7530\u72d7'],
  ['\u730e\u72d7', '\u5b88\u7530\u8005', '\u62a4\u9662\u72d7', '\u5f15\u8def\u8005', '\u62a5\u4fe1\u8005'],
  ['\u738b\u4e8c', '\u5f20\u4e09', '\u674e\u56db', '\u9648\u4e94', '\u8d75\u516d'],
  ['\u559c\u6b22\u8d77\u54c4', '\u8d2a\u73a9\u61d2\u6563', '\u5634\u786c\u7231\u7b11', '\u95f2\u901b\u7231\u8bf4', '\u597d\u770b\u70ed\u95f9'],
  ['\u9e1f\u513f\u843d\u7f51', '\u9e7f\u8dd1\u8fc7\u7530', '\u7f8a\u8e29\u574f\u82d7', '\u72d7\u8ffd\u91ce\u9e21', '\u98ce\u5439\u843d\u679c'],
  ['\u95f7\u70ed', '\u5927\u6674', '\u65e5\u5934\u6bd2', '\u5fae\u98ce', '\u5c11\u4e91'],
  ['\u86d9\u58f0\u8fde\u7eed', '\u9ea6\u82d7\u9752\u7eff', '\u8349\u53f6\u6cdb\u5149', '\u7530\u57c2\u5e72\u786c', '\u6811\u5f71\u6591\u9a73'],
  ['\u60ca\u559c', '\u5f97\u610f', '\u6697\u559c', '\u5174\u594b', '\u5fc3\u52a8'],
  ['\u51cf\u5c11\u8015\u4f5c', '\u653e\u4e0b\u9504\u5934', '\u65e9\u65e9\u56de\u5bb6', '\u53ea\u987e\u7b49\u5f85', '\u51b7\u843d\u7530\u5730'],
  ['\u88ab\u6751\u4eba\u8bae\u8bba', '\u7cae\u98df\u51cf\u6536', '\u6742\u8349\u4e1b\u751f', '\u7530\u57c2\u5d29\u574f', '\u82d7\u6839\u67af\u9ec4'],
  ['\u90bb\u4eba\u63d0\u9192', '\u7236\u4eb2\u53f9\u606f', '\u6751\u957f\u529d\u8bf4', '\u670b\u53cb\u76f4\u8a00', '\u6536\u6210\u5931\u671b'],
  ['\u7b2c\u4e8c\u5e74\u521d\u79cb', '\u4e00\u5e74\u591a\u540e', '\u6b21\u5e74\u516b\u6708', '\u79cb\u6536\u524d', '\u843d\u53f6\u65f6\u8282'],
  ['\u5730\u5934', '\u7530\u57c2\u65c1', '\u8001\u6811\u65c1', '\u6751\u897f\u5730\u8fb9', '\u8352\u7530\u4e2d'],
  ['\u51c9\u98ce', '\u897f\u98ce', '\u79cb\u51c9', '\u9634\u98ce', '\u98ce\u58f0\u7d27'],
  ['\u8349\u6728\u67af\u9ec4', '\u7530\u91cc\u5c11\u82d7', '\u67af\u8349\u904d\u5730', '\u6ce5\u571f\u5e72\u88c2', '\u6811\u53f6\u96f6\u843d'],
  ['\u6b21\u5e74\u516b\u6708', '\u79cb\u672b', '\u6253\u8c37\u65f6\u8282', '\u6536\u6210\u524d\u540e', '\u971c\u964d\u524d'],
  ['\u6751\u8fb9\u8352\u5730', '\u8001\u7530\u57c2', '\u5bb6\u95e8\u53e3', '\u6751\u5934\u5c0f\u8def', '\u7530\u95f4\u5c0f\u9053'],
  ['\u60ed\u6127', '\u61ca\u6094', '\u4e0d\u5b89', '\u6e05\u9192', '\u60f3\u6539\u8fc7'],
  ['\u4e0d\u80fd\u7b49\u5f85\u5de7\u5408', '\u4e0d\u53ef\u8d2a\u56fe\u5076\u7136', '\u8981\u9760\u5b9e\u5e72', '\u4e0d\u80fd\u7a7a\u60f3', '\u52e4\u594b\u624d\u7a33\u59a5'],
];

const zhiziDistractors = [
  ['\u516c\u5143\u524d397\u5e74', '\u516c\u5143\u524d399\u5e74', '\u6218\u56fd\u65e9\u671f', '9\u6708\u4e2d', '\u6df1\u79cb'],
  ['\u9c81\u56fd', '\u57ce\u5317\u90ca\u5916', '\u5bcc\u6237\u5bb6\u4e2d', '\u540e\u9662\u5899\u8fb9', '\u96e8\u540e\u9662\u843d'],
  ['\u94b1\u8001\u7237', '\u5b59\u5458\u5916', '\u5434\u8001\u7237', '\u90d1\u5bcc\u6237', '\u674e\u4e1c\u5bb6'],
  ['\u5730\u4e3b', '\u5e97\u4e3b', '\u8d27\u4e3b', '\u5bb6\u4e1a\u4eba', '\u4e61\u7ec5'],
  ['\u6536\u85cf\u7389\u5668', '\u8d4f\u753b', '\u70f9\u8336', '\u542c\u620f', '\u6574\u7406\u8d26\u672c'],
  ['\u957f\u5b50', '\u4e66\u751f', '\u5bb6\u4eba', '\u5c0f\u513f\u5b50', '\u4eb2\u8fd1\u665a\u8f88'],
  ['\u95e8\u751f', '\u5e74\u8f7b\u5b66\u5b50', '\u529d\u8bf4\u8005', '\u5bb6\u4e2d\u5e2e\u624b', '\u63d0\u9192\u8005'],
  ['\u94b1\u9ebb\u5b50', '\u738b\u7626\u5b50', '\u674e\u9ed1\u5b50', '\u5f20\u62d0\u5b50', '\u9648\u5077\u513f'],
  ['\u5c0f\u5077', '\u591c\u76d7', '\u60ef\u72af', '\u64ac\u95e8\u8005', '\u5077\u8d22\u8005'],
  ['\u95e8\u95e9\u635f\u574f', '\u7a97\u6237\u677e\u52a8', '\u56f4\u680f\u5012\u584c', '\u96e8\u6c34\u6d78\u574f', '\u5bb6\u95e8\u672a\u9501'],
  ['\u66b4\u96e8', '\u96e8\u540e\u653e\u6674', '\u591c\u96e8', '\u9634\u51b7', '\u6e7f\u51c9'],
  ['\u9662\u91cc\u6ce5\u6cde', '\u77f3\u9636\u6e7f\u6ed1', '\u5899\u89d2\u6e17\u6c34', '\u843d\u53f6\u6cbe\u6ce5', '\u5ead\u9662\u6f6e\u6e7f'],
  ['\u4e0d\u8010\u70e6', '\u5acc\u9ebb\u70e6', '\u6f2b\u4e0d\u7ecf\u5fc3', '\u5fc3\u70e6', '\u4e0d\u613f\u52a8\u624b'],
  ['\u5148\u5938\u4eb2\u4eba', '\u6697\u7591\u65c1\u4eba', '\u542c\u4fe1\u4e00\u65b9', '\u5ffd\u89c6\u540c\u8bdd', '\u4ee5\u4eb2\u758f\u4f5c\u5224\u65ad'],
  ['\u6848\u60c5\u6c34\u843d\u77f3\u51fa', '\u76d7\u8d3c\u843d\u7f51', '\u8bef\u4f1a\u88ab\u63ed\u5f00', '\u771f\u51f6\u4f9b\u8ba4', '\u7591\u5fc3\u843d\u7a7a'],
  ['\u5dee\u5f79\u901a\u62a5', '\u6848\u4ef6\u67e5\u660e', '\u4e3b\u4eba\u9053\u6b49', '\u771f\u51f6\u8ba4\u7f6a', '\u8bef\u89e3\u6d88\u6563'],
  ['\u6570\u6708\u540e', '\u5165\u51ac\u540e', '\u5927\u96ea\u524d\u540e', '\u5bd2\u51ac\u65f6\u8282', '\u5e74\u5e95\u524d'],
  ['\u53bf\u8859', '\u5ead\u9662\u91cc', '\u5927\u95e8\u524d', '\u5ba1\u6848\u5904', '\u5bb6\u4e2d\u5802\u524d'],
  ['\u98ce\u96ea\u4ea4\u52a0', '\u5317\u98ce', '\u5bd2\u6c14\u903c\u4eba', '\u96ea\u540e\u5929\u5bd2', '\u9634\u51b7'],
  ['\u9662\u4e2d\u79ef\u96ea', '\u74e6\u4e0a\u767d\u96ea', '\u8857\u9053\u51b0\u51b7', '\u95e8\u524d\u6e05\u51b7', '\u96ea\u75d5\u6e05\u6670'],
  ['\u5e74\u5e95', '\u51ac\u65e5', '\u6570\u6708\u540e', '\u96ea\u505c\u4e4b\u65f6', '\u5bd2\u51ac\u91cc'],
  ['\u90bb\u5c45\u95e8\u524d', '\u5bb6\u95e8\u5916', '\u5ead\u9662\u95e8\u53e3', '\u5df7\u53e3', '\u5802\u5c4b\u524d'],
  ['\u60ed\u6127\u4e0d\u5b89', '\u81ea\u8d23', '\u9192\u609f', '\u6b49\u610f', '\u61ca\u6094'],
  ['\u4e0d\u80fd\u770b\u4eba\u4e0b\u5224\u65ad', '\u8bf4\u8bdd\u5bf9\u9519\u8981\u770b\u5185\u5bb9', '\u4e0d\u8981\u56e0\u4eb2\u8fd1\u504f\u542c', '\u5224\u65ad\u8981\u516c\u5e73', '\u610f\u89c1\u4e0d\u8be5\u56e0\u4eba\u800c\u5e9f'],
];

const emperorDistractors = [
  ['1709', '1711', 'early summer', 'late morning', 'bright afternoon'],
  ['Valoria', 'royal avenue', 'the council hall', 'the palace gate', 'the capital square'],
  ['Leonardo', 'Augusto', 'Matteo', 'Prince Lorenzo', 'King Renato'],
  ['King', 'sovereign', 'court leader', 'royal patron', 'head of court'],
  ['fine robes', 'rare fabrics', 'court fashion', 'ceremonial dress', 'luxury garments'],
  ['old advisor', 'young captain', 'court official', 'senior minister', 'trusted servant'],
  ['advisor', 'court servant', 'captain', 'royal inspector', 'council member'],
  ['two strangers', 'clever tailors', 'traveling merchants', 'false weavers', 'court tricksters'],
  ['con artists', 'deceivers', 'frauds', 'impostors', 'smooth talkers'],
  ['a promise of rare fabric', 'a test of wisdom', 'a flattering offer', 'a magical claim', 'a secret weaving plan'],
  ['clear', 'warm', 'sunlit', 'cloudless', 'pleasant'],
  ['bright halls', 'busy servants', 'decorated rooms', 'open gardens', 'royal bustle'],
  ['eager', 'vain', 'pleased', 'confident', 'curious'],
  ['false inspections', 'silent agreement', 'careful pretending', 'official praise', 'secret preparation'],
  ['a public cry', 'murmuring crowd', 'truth spoken aloud', 'exposed pretense', 'awkward silence'],
  ['realization', 'public embarrassment', 'truth becoming clear', 'loss of pride', 'silent shame'],
  ['at the procession', 'that afternoon', 'during the public walk', 'midday', 'before evening'],
  ['main street', 'public square', 'near the crowd', 'market road', 'city avenue'],
  ['bright', 'warm', 'clear', 'sunny', 'mild'],
  ['watching crowd', 'raised banners', 'busy street', 'public cheers', 'festival colors'],
  ['same evening', 'after the public event', 'before nightfall', 'at dusk', 'later that day'],
  ['private chamber', 'palace room', 'royal hall', 'inner room', 'quiet corridor'],
  ['embarrassed', 'ashamed', 'foolish', 'silent', 'humbled'],
  ['pride misleads', 'truth may be simple', 'honesty matters', 'flattery is dangerous', 'plain speech can reveal truth'],
];


function flattenAnswers(questions) {
  return [...new Set(questions.flatMap((item) => item.answers))];
}

const shouDraft = {
  version: 'storylock-story-draft-v1',
  templateId: 'shouzhudaitu-zh',
  language: 'zh-CN',
  storyTitle: z.shou,
  summary: '\u9752\u5e74\u519c\u592b\u963f\u798f\u5076\u7136\u5f97\u5230\u4e00\u53ea\u649e\u6811\u800c\u6b7b\u7684\u5154\u5b50\uff0c\u4fbf\u4ee5\u4e3a\u597d\u8fd0\u53ef\u4ee5\u53cd\u590d\u964d\u4e34\u3002\u4ed6\u653e\u4e0b\u519c\u5177\uff0c\u65e5\u65e5\u5b88\u5728\u8001\u68a8\u6811\u4e0b\uff0c\u6700\u7ec8\u7530\u5730\u8352\u829c\uff0c\u5728\u6bcd\u4eb2\u548c\u6751\u4eba\u7684\u63d0\u9192\u4e2d\u660e\u767d\u4e86\u52e4\u52b3\u624d\u662f\u771f\u6b63\u7684\u4f9d\u9760\u3002',
  storyPlot: '\u516c\u5143\u524d332\u5e74\u7684\u4e1c\u5468\u65f6\u671f\uff0c\u5b8b\u56fd\u9752\u77f3\u6751\u7684\u7530\u95f4\u70c8\u65e5\u660e\u4eae\u3001\u8749\u9e23\u9635\u9635\u3002\u519c\u592b\u963f\u798f\u5728\u8001\u68a8\u6811\u4e0b\u8015\u4f5c\u65f6\uff0c\u4e00\u53ea\u88ab\u72d0\u72f8\u8ffd\u8d76\u7684\u5154\u5b50\u649e\u6811\u800c\u6b7b\u3002\u963f\u798f\u5927\u559c\uff0c\u4ece\u6b64\u5e26\u7740\u8001\u9ec4\u72d7\u5b88\u5728\u6811\u65c1\uff0c\u4e0d\u518d\u8ba4\u771f\u8015\u4f5c\u3002\u4e00\u5e74\u540e\u7684\u79cb\u5929\uff0c\u843d\u53f6\u6ee1\u5730\uff0c\u7530\u5730\u8352\u829c\uff0c\u5218\u4e09\u7684\u5632\u7b11\u548c\u6bcd\u4eb2\u7684\u529d\u544a\u8ba9\u4ed6\u7f9e\u6127\u9192\u609f\uff0c\u660e\u767d\u4e0d\u80fd\u628a\u751f\u6d3b\u5bc4\u6258\u5728\u4fa5\u5e78\u4e0a\u3002',
  memoryAnchors: ['\u516c\u5143\u524d332\u5e74', '\u5b8b\u56fd', '\u9752\u77f3\u6751', '\u963f\u798f', '\u8001\u68a8\u6811', '\u5154\u5b50\u649e\u6811', '\u8001\u9ec4\u72d7', '\u5218\u4e09', '\u7530\u5730\u8352\u829c', '\u4e0d\u53ef\u4fa5\u5e78'],
  elementGroups: groups,
  nodes: makeNodes(z.q, withVagueQuestions(shouQuestions, vagueChineseQuestions), '\u5b88\u682a\u5f85\u5154\u6a21\u677f\uff1b\u9898\u76ee\u5df2\u53bb\u9664\u6545\u4e8b\u540d\u3001\u4eba\u540d\u548c\u5173\u952e\u79d8\u5bc6\uff0c\u5168\u90e8\u4e3a\u591a\u9009\u3002', flattenAnswers(zhiziQuestions), shouDistractors, zhiziDistractors),
};

const zhiziDraft = {
  version: 'storylock-story-draft-v1',
  templateId: 'zhizi-yilin-zh',
  language: 'zh-CN',
  storyTitle: z.zhizi,
  summary: '\u5bcc\u6237\u5468\u8001\u7237\u5728\u5927\u96e8\u540e\u5ffd\u89c6\u9662\u5899\u635f\u574f\u7684\u9690\u60a3\uff0c\u5bb6\u4e2d\u5931\u7a83\u540e\u5374\u56e0\u4eb2\u758f\u4e0d\u540c\uff0c\u79f0\u8d5e\u513f\u5b50\u6709\u89c1\u8bc6\uff0c\u6000\u7591\u90bb\u5c45\u522b\u6709\u7528\u5fc3\u3002\u771f\u76d7\u88ab\u6355\u540e\uff0c\u4ed6\u624d\u660e\u767d\u5224\u65ad\u662f\u975e\u4e0d\u80fd\u88ab\u5173\u7cfb\u8fdc\u8fd1\u5de6\u53f3\u3002',
  storyPlot: '\u516c\u5143\u524d398\u5e74\u7684\u6218\u56fd\u65f6\u671f\uff0c\u5b8b\u56fd\u90fd\u57ce\u90ca\u5916\u7684\u5468\u8001\u7237\u5bb6\u906d\u9047\u5927\u96e8\uff0c\u9662\u5899\u88ab\u51b2\u574f\u3002\u513f\u5b50\u5468\u6587\u548c\u90bb\u5c45\u5b59\u4f2f\u90fd\u529d\u4ed6\u53ca\u65e9\u4fee\u5899\uff0c\u5468\u8001\u7237\u5374\u4e0d\u4ee5\u4e3a\u610f\u3002\u4e0d\u4e45\u5bb6\u4e2d\u5931\u7a83\uff0c\u4ed6\u8ba4\u4e3a\u513f\u5b50\u806a\u660e\uff0c\u5374\u7591\u5fc3\u90bb\u5c45\u6697\u85cf\u7978\u5fc3\u3002\u4e09\u4e2a\u6708\u540e\uff0c\u5b98\u5e9c\u6293\u5230\u60ef\u5077\u8d75\u9ebb\u5b50\uff0c\u771f\u76f8\u5927\u767d\u3002\u5468\u8001\u7237\u5192\u96ea\u5411\u5b59\u4f2f\u9053\u6b49\uff0c\u660e\u767d\u4e0d\u53ef\u56e0\u4eba\u5e9f\u8a00\uff0c\u4e5f\u4e0d\u53ef\u56e0\u4eb2\u758f\u8bef\u5224\u3002',
  memoryAnchors: ['\u516c\u5143\u524d398\u5e74', '\u5b8b\u56fd', '\u5468\u8001\u7237', '\u9662\u5899\u635f\u574f', '\u5468\u6587', '\u5b59\u4f2f', '\u8d75\u9ebb\u5b50', '\u5b98\u5e9c\u7834\u6848', '\u771f\u76f8\u5927\u767d', '\u4e0d\u53ef\u56e0\u4eba\u5e9f\u8a00'],
  elementGroups: groups,
  nodes: makeNodes(z.q, withVagueQuestions(zhiziQuestions, vagueChineseQuestions), '\u667a\u5b50\u7591\u90bb\u6a21\u677f\uff1b\u9898\u76ee\u5df2\u53bb\u9664\u6545\u4e8b\u540d\u3001\u4eba\u540d\u548c\u5173\u952e\u79d8\u5bc6\uff0c\u5168\u90e8\u4e3a\u591a\u9009\u3002', flattenAnswers(shouQuestions), zhiziDistractors, shouDistractors),
};

const emperorDraft = {
  version: 'storylock-story-draft-v1',
  templateId: 'emperor-new-clothes-en',
  language: 'en-US',
  storyTitle: "The Emperor's New Clothes",
  summary: 'Emperor Lorenzo is deceived by Marco and Pietro, two swindlers who promise magical cloth visible only to the wise. His vanity carries him into a public parade, where a child speaks plainly and exposes the lie.',
  storyPlot: 'In 1710, on a bright summer morning in Fantasia, Emperor Lorenzo fills the palace with talk of splendid new clothes. Marco and Pietro claim they can weave invisible cloth that only worthy people can see. Alberto and Bruno inspect the empty looms and lie out of fear, so the emperor marches through the city streets in a parade wearing nothing. When a child calls out the truth, the crowd begins to whisper, and Lorenzo returns to the palace ashamed, having learned that vanity makes even rulers foolish.',
  memoryAnchors: ['1710', 'Fantasia', 'Emperor Lorenzo', 'new clothes', 'Marco and Pietro', 'Alberto', 'Bruno', 'child', 'parade', 'vanity'],
  elementGroups: groups,
  nodes: makeNodes('Question', withVagueQuestions(emperorQuestions, vagueEnglishQuestions), "Polished English template with vague prompts; all nodes are multi-select.", flattenAnswers([...shouQuestions, ...zhiziQuestions]).slice(0, 80), emperorDistractors, emperorDistractors),
};

const drafts = [shouDraft, zhiziDraft, emperorDraft];
const manifestItems = drafts.map((draft) => ({
  templateId: draft.templateId,
  language: draft.language,
  storyTitle: draft.storyTitle,
  fileName: `${draft.templateId}.json`,
  nodeCount: draft.nodes.length,
}));
const manifest = {
  schemaVersion: 'storylock-story-draft-manifest-v1',
  defaultTemplateId: 'shouzhudaitu-zh',
  items: manifestItems,
};

function writeJson(path, data) {
  writeFileSync(path, `${JSON.stringify(data, null, 2)}\n`, 'utf8');
}

for (const root of draftRoots) {
  mkdirSync(root, { recursive: true });
  writeJson(join(root, 'manifest.json'), manifest);
  for (const draft of drafts) {
    writeJson(join(root, `${draft.templateId}.json`), draft);
  }
}

mkdirSync(templateRoot, { recursive: true });
writeJson(join(templateRoot, 'manifest.json'), {
  schemaVersion: 'storylock-template-directory-manifest-v1',
  description: 'Standalone StoryLock story template directories for user download.',
  items: manifestItems.map((item) => ({
    ...item,
    directoryName: item.templateId,
    templateFileName: 'story-template.json',
  })),
});
writeFileSync(join(templateRoot, 'README.md'), [
  '# StoryLock story template directories',
  '',
  'This folder contains three standalone story template directories for download packaging.',
  '',
  '- shouzhudaitu-zh',
  '- zhizi-yilin-zh',
  '- emperor-new-clothes-en',
  '',
].join('\n'), 'utf8');

for (const draft of drafts) {
  const dir = join(templateRoot, draft.templateId);
  mkdirSync(dir, { recursive: true });
  writeJson(join(dir, 'story-template.json'), draft);
  const titleLine = draft.language === 'zh-CN' ? `# ${draft.storyTitle}` : `# ${draft.storyTitle}`;
  const description = draft.language === 'zh-CN'
    ? [z.multi, z.readmeUse].join('\n\n')
    : 'This template is structured as 24 multi-select StoryLock questions.\n\nImport or use story-template.json as a StoryLock story template.';
  writeFileSync(join(dir, 'README.md'), `${titleLine}\n\n${description}\n`, 'utf8');
}

console.log(JSON.stringify({
  status: 'generated',
  draftRoots,
  templateRoot,
  templates: drafts.map((draft) => draft.templateId),
}, null, 2));
