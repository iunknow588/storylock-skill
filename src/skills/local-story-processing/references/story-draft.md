# Story Draft

## Command Template

```js
import { StoryDraftSkill } from "../index.js";

const skill = new StoryDraftSkill();

const result = await skill.run({
  objective: "help the user produce a first private story draft",
  audience: "self",
  tone: "calm",
  constraints: ["keep it concise", "preserve mnemonic cues"],
  source: "approved_local_input",
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `objective` | string | yes | none | Drafting goal. |
| `audience` | string | yes | none | Intended reader, usually `self`. |
| `tone` | string | no | `neutral` | Draft tone hint. |
| `constraints` | string[] | no | `[]` | Draft constraints or reminders. |
| `source` | string | no | `approved_local_input` | Input provenance label. |

## Output Parsing

The skill returns a structured draft result:

```json
{
  "mode": "story_draft",
  "processingLayer": "local_story_processing",
  "source": "approved_local_input",
  "draft": {
    "title": "Story Draft",
    "content": "...",
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

1. `mode`: fixed identifier for the drafting path.
2. `draft`: draft payload to pass to later local processing.
3. `notes`: optional guidance for later refinement.

## Error Handling

| Error | When It Happens | Fix |
| --- | --- | --- |
| `objective must be a non-empty string` | Missing objective | Provide a concrete drafting goal. |
| `audience must be a non-empty string` | Missing audience | Provide `self` or another audience label. |

## Agent Guidelines

1. Use this skill only for local-first drafting.
2. Do not treat it as a challenge or secret-access path.
3. Keep private story material in the local processing layer.
4. Hand protected-object reads and writes back to the local access package.
