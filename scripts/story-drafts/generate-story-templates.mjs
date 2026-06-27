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

function makeNodes(titlePrefix, questions, editorNotes, negativePool) {
  return questions.map((item, index) => {
    const correct = item.answers.map((answer) => option(answer, true));
    const falseOptions = negativePool
      .filter((candidate) => !item.answers.includes(candidate))
      .slice(index % 5, index % 5 + Math.max(2, 9 - correct.length))
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
  { question: '\u300a\u5b88\u682a\u5f85\u5154\u300b\u6545\u4e8b\u5177\u4f53\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u516c\u5143\u524d332\u5e74', '\u4e1c\u5468\u65f6\u671f', '4\u67085\u65e5', '\u76db\u590f'] },
  { question: '\u6545\u4e8b\u4e3b\u8981\u53d1\u751f\u5728\u54ea\u4e9b\u5730\u65b9\uff1f', answers: ['\u5b8b\u56fd', '\u9752\u77f3\u6751', '\u7530\u95f4', '\u8001\u68a8\u6811\u4e0b'] },
  { question: '\u6545\u4e8b\u4e2d\u7684\u4e3b\u4eba\u516c\u53eb\u4ec0\u4e48\uff1f', answers: ['\u963f\u798f'] },
  { question: '\u963f\u798f\u7684\u8eab\u4efd\u6216\u804c\u4e1a\u662f\u4ec0\u4e48\uff1f', answers: ['\u519c\u592b', '\u79cd\u7530\u4eba'] },
  { question: '\u963f\u798f\u5e73\u65f6\u559c\u6b22\u505a\u4ec0\u4e48\uff1f', answers: ['\u5531\u6b4c', '\u54fc\u5c0f\u8c03'] },
  { question: '\u8c01\u662f\u963f\u798f\u8eab\u8fb9\u7684\u966a\u4f34\u8005\uff1f', answers: ['\u8001\u9ec4\u72d7', '\u9ec4\u72d7'] },
  { question: '\u8001\u9ec4\u72d7\u5728\u6545\u4e8b\u4e2d\u7684\u8eab\u4efd\u662f\u4ec0\u4e48\uff1f', answers: ['\u770b\u95e8\u72d7', '\u966a\u4f34\u8005'] },
  { question: '\u6545\u4e8b\u4e2d\u5632\u7b11\u963f\u798f\u7684\u4eba\u662f\u8c01\uff1f', answers: ['\u5218\u4e09', '\u75de\u5b50'] },
  { question: '\u5218\u4e09\u662f\u600e\u6837\u7684\u4eba\uff1f', answers: ['\u6e38\u624b\u597d\u95f2', '\u5632\u7b11\u4eba'] },
  { question: '\u963f\u798f\u5f00\u59cb\u5b88\u682a\u7684\u76f4\u63a5\u539f\u56e0\u662f\u4ec0\u4e48\uff1f', answers: ['\u5154\u5b50\u649e\u6811', '\u72d0\u72f8\u8ffd\u8d76'] },
  { question: '\u6545\u4e8b\u5f00\u5934\u7684\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u708e\u70ed', '\u6674\u6717'] },
  { question: '\u6545\u4e8b\u5f00\u5934\u7684\u7530\u95f4\u73af\u5883\u5982\u4f55\uff1f', answers: ['\u8749\u9e23\u9635\u9635', '\u5e84\u7a3c\u8302\u76db'] },
  { question: '\u963f\u798f\u6361\u5230\u5154\u5b50\u540e\u7684\u5fc3\u60c5\u662f\u4ec0\u4e48\uff1f', answers: ['\u5927\u559c', '\u9ad8\u5174'] },
  { question: '\u6545\u4e8b\u53d1\u5c55\u4e2d\u963f\u798f\u505a\u4e86\u4ec0\u4e48\u9519\u8bef\u9009\u62e9\uff1f', answers: ['\u653e\u5f03\u8015\u4f5c', '\u6bcf\u65e5\u5b88\u682a'] },
  { question: '\u6545\u4e8b\u9ad8\u6f6e\u4e2d\u51fa\u73b0\u4e86\u4ec0\u4e48\u540e\u679c\uff1f', answers: ['\u906d\u5230\u5168\u6751\u803b\u7b11', '\u7530\u5730\u8352\u829c'] },
  { question: '\u4ec0\u4e48\u4e8b\u60c5\u4fc3\u4f7f\u963f\u798f\u53cd\u601d\uff1f', answers: ['\u6bcd\u4eb2\u529d\u544a', '\u5218\u4e09\u5632\u7b11'] },
  { question: '\u6545\u4e8b\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u5019\uff1f', answers: ['\u6b21\u5e74\u79cb\u5929', '\u4e00\u5e74\u540e'] },
  { question: '\u6545\u4e8b\u8f6c\u6298\u53d1\u751f\u5728\u54ea\u91cc\uff1f', answers: ['\u7530\u8fb9', '\u8001\u68a8\u6811\u4e0b'] },
  { question: '\u8f6c\u6298\u65f6\u7684\u5929\u6c14\u6709\u4ec0\u4e48\u7279\u70b9\uff1f', answers: ['\u79cb\u98ce', '\u8427\u745f'] },
  { question: '\u8f6c\u6298\u65f6\u7684\u73af\u5883\u5982\u4f55\uff1f', answers: ['\u843d\u53f6\u6ee1\u5730', '\u7530\u5730\u8352\u829c'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u6b21\u5e74\u79cb\u5929', '\u6536\u5272\u5b63\u8282'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u7684\u5730\u70b9\u5728\u54ea\u91cc\uff1f', answers: ['\u8352\u829c\u7684\u7530\u5730', '\u6751\u53e3'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u963f\u798f\u7684\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u7f9e\u6127', '\u51b3\u5fc3\u52e4\u52b3'] },
  { question: '\u8fd9\u4e2a\u6545\u4e8b\u60f3\u8ba9\u4eba\u8bb0\u4f4f\u4ec0\u4e48\u9053\u7406\uff1f', answers: ['\u4e0d\u53ef\u4fa5\u5e78', '\u52e4\u52b3\u81f4\u5bcc'] },
];

const zhiziQuestions = [
  { question: '\u300a\u667a\u5b50\u7591\u90bb\u300b\u6545\u4e8b\u5177\u4f53\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u516c\u5143\u524d398\u5e74', '\u6218\u56fd\u65f6\u671f', '9\u6708\u521d', '\u521d\u79cb'] },
  { question: '\u6545\u4e8b\u4e3b\u8981\u53d1\u751f\u5728\u54ea\u4e9b\u5730\u65b9\uff1f', answers: ['\u5b8b\u56fd', '\u90fd\u57ce\u90ca\u5916', '\u5468\u8001\u7237\u5bb6', '\u9662\u5899\u8fb9'] },
  { question: '\u6545\u4e8b\u4e2d\u7684\u4e3b\u4eba\u516c\u662f\u8c01\uff1f', answers: ['\u5468\u8001\u7237'] },
  { question: '\u5468\u8001\u7237\u7684\u8eab\u4efd\u6216\u804c\u4e1a\u662f\u4ec0\u4e48\uff1f', answers: ['\u5546\u4eba', '\u5bcc\u6237'] },
  { question: '\u5468\u8001\u7237\u5e73\u65f6\u559c\u6b22\u4ec0\u4e48\uff1f', answers: ['\u6536\u85cf\u53e4\u73a9', '\u54c1\u8336'] },
  { question: '\u7ed9\u5468\u8001\u7237\u63d0\u9192\u7684\u4eb2\u8fd1\u4eba\u662f\u8c01\uff1f', answers: ['\u5468\u6587', '\u513f\u5b50'] },
  { question: '\u5468\u6587\u5728\u6545\u4e8b\u4e2d\u7684\u8eab\u4efd\u662f\u4ec0\u4e48\uff1f', answers: ['\u8bfb\u4e66\u4eba', '\u5efa\u8bae\u8005'] },
  { question: '\u771f\u6b63\u7684\u4f5c\u6076\u8005\u662f\u8c01\uff1f', answers: ['\u8d75\u9ebb\u5b50', '\u60ef\u5077'] },
  { question: '\u8d75\u9ebb\u5b50\u5e38\u505a\u4ec0\u4e48\u574f\u4e8b\uff1f', answers: ['\u76d7\u8d3c', '\u4e13\u5077\u5bcc\u6237'] },
  { question: '\u5468\u5bb6\u53ef\u80fd\u5931\u7a83\u7684\u8d77\u56e0\u662f\u4ec0\u4e48\uff1f', answers: ['\u9662\u5899\u635f\u574f', '\u5927\u96e8\u51b2\u574f'] },
  { question: '\u6545\u4e8b\u5f00\u5934\u7684\u5929\u6c14\u5982\u4f55\uff1f', answers: ['\u5927\u96e8', '\u96e8\u505c\u6708\u51fa'] },
  { question: '\u6545\u4e8b\u5f00\u5934\u7684\u73af\u5883\u5982\u4f55\uff1f', answers: ['\u79ef\u6c34\u904d\u5730', '\u6ce5\u6cde\u4e0d\u582a'] },
  { question: '\u542c\u5230\u9662\u5899\u635f\u574f\u540e\u5468\u8001\u7237\u7684\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u70e6\u8e81', '\u4e0d\u4ee5\u4e3a\u610f'] },
  { question: '\u6545\u4e8b\u53d1\u5c55\u4e2d\u5468\u8001\u7237\u600e\u6837\u5224\u65ad\u522b\u4eba\uff1f', answers: ['\u5931\u7a83', '\u79f0\u8d5e\u513f\u5b50', '\u6000\u7591\u90bb\u5c45'] },
  { question: '\u6545\u4e8b\u9ad8\u6f6e\u4e2d\u51fa\u73b0\u4e86\u4ec0\u4e48\u7ed3\u679c\uff1f', answers: ['\u771f\u76f8\u5927\u767d', '\u771f\u76d7\u88ab\u6355'] },
  { question: '\u4ec0\u4e48\u4e8b\u60c5\u4fc3\u4f7f\u5468\u8001\u7237\u9192\u609f\uff1f', answers: ['\u5b98\u5e9c\u7834\u6848', '\u5468\u8001\u7237\u6094\u609f'] },
  { question: '\u6545\u4e8b\u8f6c\u6298\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u5019\uff1f', answers: ['\u4e09\u4e2a\u6708\u540e', '\u51ac\u5929'] },
  { question: '\u6545\u4e8b\u8f6c\u6298\u53d1\u751f\u5728\u54ea\u91cc\uff1f', answers: ['\u5b98\u5e9c', '\u5468\u8001\u7237\u5bb6'] },
  { question: '\u8f6c\u6298\u65f6\u7684\u5929\u6c14\u6709\u4ec0\u4e48\u7279\u70b9\uff1f', answers: ['\u5bd2\u98ce\u51db\u51bd', '\u5927\u96ea'] },
  { question: '\u8f6c\u6298\u65f6\u7684\u73af\u5883\u5982\u4f55\uff1f', answers: ['\u767d\u96ea\u8986\u76d6', '\u5bd2\u51b7'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u53d1\u751f\u5728\u4ec0\u4e48\u65f6\u95f4\uff1f', answers: ['\u51ac\u5929', '\u4e09\u4e2a\u6708\u540e'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u7684\u5730\u70b9\u5728\u54ea\u91cc\uff1f', answers: ['\u5b59\u4f2f\u5bb6\u95e8\u53e3', '\u5468\u8001\u7237\u5bb6'] },
  { question: '\u6545\u4e8b\u7ed3\u5c3e\u5468\u8001\u7237\u7684\u5fc3\u60c5\u5982\u4f55\uff1f', answers: ['\u7f9e\u6127', '\u6094\u609f'] },
  { question: '\u8fd9\u4e2a\u6545\u4e8b\u60f3\u8ba9\u4eba\u8bb0\u4f4f\u4ec0\u4e48\u9053\u7406\uff1f', answers: ['\u4e0d\u53ef\u56e0\u4eba\u5e9f\u8a00', '\u4e0d\u53ef\u56e0\u4eb2\u758f\u8bef\u5224'] },
];

const emperorQuestions = [
  { question: 'When exactly does the story take place?', answers: ['1710', 'early 18th century', 'summer morning', 'bright day'] },
  { question: 'Which places frame the main action?', answers: ['Fantasia', 'city streets', 'the palace', "the emperor's realm"] },
  { question: "What is the protagonist's name?", answers: ['Lorenzo'] },
  { question: 'What role does the protagonist hold?', answers: ['Ruler', 'monarch'] },
  { question: 'What interest makes the protagonist vulnerable?', answers: ['New clothes', 'splendid attire'] },
  { question: "Who are the protagonist's main assistants?", answers: ['Alberto (old minister)', 'Bruno (young officer)'] },
  { question: 'What official roles do the assistants hold?', answers: ['Minister', 'trusted advisor', 'officer', 'official'] },
  { question: 'Who are the villains?', answers: ['Marco and Pietro', 'two swindlers'] },
  { question: 'What kind of people are the villains?', answers: ['Swindlers', 'tricksters'] },
  { question: 'What directly causes the deception to begin?', answers: ['The promise of invisible cloth', "emperor's vanity"] },
  { question: 'What is the weather like when events begin?', answers: ['Sunny', 'bright'] },
  { question: 'What is the palace environment like at the beginning?', answers: ['Gardens in bloom', 'palace splendor'] },
  { question: 'How does Emperor Lorenzo feel at the beginning?', answers: ['Excited', 'proud'] },
  { question: 'How does the deception develop before the parade?', answers: ['Inspections', 'lying', 'preparation'] },
  { question: 'What forms the climax of the story?', answers: ['The child shouting the truth', "crowd's whisper"] },
  { question: 'What event turns pride into shame?', answers: ['The emperor realizing he was naked', 'shame'] },
  { question: 'When does the turning point happen?', answers: ['During the parade', 'later that day'] },
  { question: 'Where does the turning point happen?', answers: ['City streets', 'among the crowd'] },
  { question: 'What is the weather like at the turning point?', answers: ['Sunny', 'warm'] },
  { question: 'What is the public scene like at the turning point?', answers: ['Cheering crowds', 'colorful banners'] },
  { question: 'When does the story end?', answers: ['Later that evening', 'after the parade'] },
  { question: 'Where is the emperor when the story closes?', answers: ['The palace', "the emperor's chamber"] },
  { question: 'How does the emperor feel after the truth is exposed?', answers: ['Foolish', 'humiliated', 'ashamed'] },
  { question: 'What lesson does the emperor finally learn?', answers: ['Vanity is foolish', 'children speak truth'] },
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
  nodes: makeNodes(z.q, shouQuestions, '\u5b88\u682a\u5f85\u5154\u6a21\u677f\uff1b\u9898\u76ee\u5df2\u6309\u6545\u4e8b\u6a21\u677f\u6587\u6863\u6da6\u8272\uff0c\u5168\u90e8\u4e3a\u591a\u9009\u3002', flattenAnswers(zhiziQuestions)),
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
  nodes: makeNodes(z.q, zhiziQuestions, '\u667a\u5b50\u7591\u90bb\u6a21\u677f\uff1b\u9898\u76ee\u5df2\u6309\u6545\u4e8b\u6a21\u677f\u6587\u6863\u6da6\u8272\uff0c\u5168\u90e8\u4e3a\u591a\u9009\u3002', flattenAnswers(shouQuestions)),
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
  nodes: makeNodes('Question', emperorQuestions, "Polished English template for The Emperor's New Clothes; all nodes are multi-select.", flattenAnswers([...shouQuestions, ...zhiziQuestions]).slice(0, 80)),
};

const drafts = [shouDraft, zhiziDraft, emperorDraft];
const manifest = {
  schemaVersion: 'storylock-story-draft-manifest-v1',
  defaultTemplateId: 'shouzhudaitu-zh',
  items: drafts.map((draft) => ({
    templateId: draft.templateId,
    language: draft.language,
    storyTitle: draft.storyTitle,
    fileName: `${draft.templateId}.json`,
  })),
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
  items: manifest.items.map((item) => ({
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
