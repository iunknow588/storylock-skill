import assert from 'node:assert/strict';
import {
  StoryDraftSkill,
  StoryRefineSkill,
  StrengthReviewSkill,
} from '../index.js';

function makeQuestion(index) {
  return {
    nodeId: `q-${index}`,
    validAnswers: [`answer-${index}`],
    distractors: Array.from({ length: 8 }, (_, offset) => `distractor-${index}-${offset}`),
  };
}

const draftSkill = new StoryDraftSkill();
const draft = await draftSkill.run({
  objective: 'Write a local-only memory cue',
  audience: 'self',
  tone: 'plain',
  constraints: ['no remote retention'],
});
assert.equal(draft.mode, 'story_draft');
assert.equal(draft.boundary.challengeCreated, false);
assert.equal(draft.boundary.protectedObjectRead, false);

const refineSkill = new StoryRefineSkill();
const refined = await refineSkill.run({
  storyDraft: draft.draft,
  goals: ['make it shorter'],
  hintStyle: 'mnemonic',
});
assert.equal(refined.mode, 'story_refine');
assert.match(refined.refinedDraft.content, /Goals:/);

const strengthSkill = new StrengthReviewSkill();
const strong = await strengthSkill.run({
  questions: Array.from({ length: 24 }, (_, index) => makeQuestion(index + 1)),
});
assert.equal(strong.mode, 'strength_review');
assert.equal(strong.formalEligibleQuestionCount, 24);
assert.equal(strong.questionSetReady, true);
assert.equal(strong.boundary.sessionIssued, false);

await assert.rejects(
  () => strengthSkill.run({ questions: Array.from({ length: 23 }, (_, index) => makeQuestion(index + 1)) }),
  /questions must contain exactly 24 nodes/,
);

console.log('StoryLock local story processing selftest passed.');
