# storylock-puzzle-plugin

Reusable StoryLock nine-grid puzzle UI core.

This crate is intentionally limited to display data and user-selection state. It does not store, parse, receive, or verify correct answers. The host or an external verification service owns answer checking.

## What is inside

- Story draft to challenge-cell conversion
- Fixed 9-option answer grids, padded with empty strings when needed
- Multi-select toggle behavior for user choices
- Answer text normalization helper

## What stays outside

- Correct answers
- `isCorrect` parsing
- Unlock authorization and answer verification
- Slint dialog rendering
- Package path selection and draft file loading
- Host status text and window lifecycle

## Example

```toml
[dependencies]
storylock-puzzle-plugin = { path = "../../plugins/storylock-puzzle-plugin" }
```

```rust
use storylock_puzzle_plugin::{create_open_challenge_from_draft, toggle_selection};

let cells = create_open_challenge_from_draft(&draft, 2)?;
let mut selections = vec![Vec::<String>::new(); cells.len()];

toggle_selection(&cells, &mut selections, 0, 1);
// Submit `cells[index].cell_id` + `selections[index]` to an external verifier.
```

## Chinese docs

- [概述](./docs/概述.md)
- [接口说明](./docs/接口说明.md)
- [使用指南](./docs/使用指南.md)
- [测试说明](./docs/测试说明.md)
