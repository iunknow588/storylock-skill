---
name: storylock-local-story-processing-skill
description: Local-only StoryLock skill package for story drafting, refinement, and private story processing.
---

# StoryLock Local Story Processing Skill

This package defines the first layer of StoryLock: pure local story processing.

## Capability Index

| User Intent | Capability | Reference File | Key Parameters |
| --- | --- | --- | --- |
| "帮我生成故事草稿" | story draft | `references/story-draft.md` | `objective`, `audience`, `tone`, `constraints`, `source` |
| "帮我润色故事" | story refine | `references/story-refine.md` | `storyDraft`, `goals`, `hintStyle`, `source` |
| "解释本地故事处理边界" | processing boundary | `references/boundary.md` | none |

## Working Rules

1. This package is local-first.
2. It is allowed to process private story input.
3. It does not perform challenge, session, or secret-object access by itself.
4. It must receive approved input from the local story access package when touching protected objects.
5. It must not expose private input to the remote gateway package.
6. Its runtime result must mark `challengeCreated`, `sessionIssued`, `protectedObjectRead`, and `remoteRetentionGranted` as `false`.
