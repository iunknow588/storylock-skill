# StoryLock Puzzle Plugin Migration

The nine-grid plugin has been split back to a display/input boundary.

## Current state

The plugin owns:

- challenge-cell generation from Story draft JSON
- extracting only `nodeId`, `question`, and `answerOptionsLocalOnly[].text`
- fixed 9-option grid data
- user selection toggle behavior
- answer normalization helper

The plugin no longer owns:

- correct-answer fields
- `isCorrect` parsing
- answer verification
- unlock authorization

The Windows host or another external service should hold the correct answers and verify submitted user selections.

## Host boundary

The host still owns:

- Slint authorization dialog rendering
- package path resolution
- draft loading from the active package
- status text and window lifecycle
- verification and authorization flow

## Suggested next steps

1. Update host adapters to submit only `cell_id` plus selected answers.
2. Keep correct-answer extraction in a verifier module outside this plugin.
3. Add adapter tests around the host verification boundary.
