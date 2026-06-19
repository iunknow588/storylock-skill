# Story Assist

## Overview

Use this capability to generate or refine StoryLock story material through the migrated skill layer.

Implemented skill classes:

1. `StoryDraftAssistSkill`
2. `StoryRefineAssistSkill`

Primary implementation source:

1. `assets/migrated/skills/story-assist.js`

## Command Template

```js
import { StoryDraftAssistSkill } from "../assets/migrated/skills/story-assist.js";

const skill = new StoryDraftAssistSkill({
  generator: async ({ objective, audience, tone, constraints }) => ({
    objective,
    audience,
    tone,
    constraints,
    draft: `Draft for ${objective}`,
  }),
});

const result = await skill.run({
  objective: "Explain StoryLock to judges",
  audience: "hackathon reviewer",
  tone: "clear and concrete",
  constraints: ["separate skill layer from core security"],
});
```

## Parameters

| Name | Type | Required | Default | Notes |
| --- | --- | --- | --- | --- |
| `objective` | `string` | yes | none | Must be a non-empty string. |
| `audience` | `string` | no | `"individual founder"` | Target reader or listener. |
| `tone` | `string` | no | `"memorable and concrete"` | Style hint for the generator. |
| `constraints` | `string[]` | no | `[]` | Extra constraints passed through to the generator. |

Input schema:

1. `assets/schemas/story-draft-input.schema.json`

Template:

1. `assets/templates/story-template.json`

## Output Parsing

The return shape is generator-defined. At minimum, agents should expect an object representing the generated draft or refinement result.

Recommended fields to preserve when present:

1. `objective`
2. `audience`
3. `tone`
4. `constraints`
5. `draft` or equivalent generated content field

## Error Handling

| Error Code | Trigger | Fix |
| --- | --- | --- |
| `VALIDATION_ERROR` | `objective` is missing or empty | Supply a non-empty `objective`. |
| `VALIDATION_ERROR` | `generator` is not a function | Inject a valid async or sync generator function. |
| custom generator error | upstream content generation failed | Retry with corrected inputs or inspect the generator implementation. |

## Agent Guidelines

1. Confirm the task is about drafting or refining story material rather than authorization.
2. Ensure `objective` is present and concrete.
3. Add `audience`, `tone`, and `constraints` only when they materially help the output.
4. Preserve the returned structure instead of flattening it into plain text unless the caller asked for plain text.
5. If generation fails, report whether the failure came from input validation or the injected generator.
