# Story Refine

## Command Template

```js
import { StoryRefineSkill } from "../index.js";

const skill = new StoryRefineSkill();

const result = await skill.run({
  storyDraft: {
    title: "My Memory",
    content: "Initial draft..."
  },
  goals: ["improve clarity", "preserve private detail"],
  hintStyle: "short mnemonic cues",
  source: "approved_local_input",
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `storyDraft` | object | yes | none | Existing draft object. |
| `goals` | string[] | no | `[]` | Refinement goals. |
| `hintStyle` | string | no | `plain` | Mnemonic or editing style hint. |
| `source` | string | no | `approved_local_input` | Input provenance label. |

## Output Parsing

```json
{
  "mode": "story_refine",
  "processingLayer": "local_story_processing",
  "source": "approved_local_input",
  "refinedDraft": {
    "title": "Updated Title",
    "content": "Updated content...",
    "cues": [],
    "metadata": {}
  },
  "notes": [],
  "boundary": {
    "challengeCreated": false,
    "sessionIssued": false,
    "protectedObjectRead": false,
    "remoteRetentionGranted": false
  }
}
```

Fields:

1. `mode`: fixed identifier for refinement.
2. `refinedDraft`: locally refined story draft.
3. `notes`: optional editing notes.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `storyDraft must be an object` | Missing draft payload | Pass a structured draft object. |
| `storyDraft.content must be a non-empty string` | Empty draft content | Provide draft content before refinement. |

## Agent Guidelines

1. Use this skill after a local draft already exists.
2. Keep refinement local when input contains private story material.
3. Do not bypass the local access layer for protected-object writes.
