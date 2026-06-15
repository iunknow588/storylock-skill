# StoryLock Skill Boundary

## 1. What this layer is

StoryLock's skill layer is the reusable high-level interface layer above local authorization core semantics.

It is responsible for:

1. story drafting and refinement helpers
2. question-set strength review
3. local password-fill packaging
4. local challenge-signing packaging
5. demo, packaging, and integration-facing invocation surfaces

## 2. What this layer is not

It does not:

1. redefine `6-of-24`, `12-of-24`, or `22-of-24`
2. replace core authorization judgment
3. replace host UI, deployment policy, or credential governance
4. automatically convert "secret readable" into "business workflow fully authorized"

## 3. Current migrated package boundary

For `skill/src/storylock-skill-engine/`, the current boundary is:

1. JS skill-layer is locally runnable and testable
2. Rust/WASM build and dist validation are wired in
3. full Rust-host integration inside this package is still incomplete

That means the package is already beyond a pure document shell, but still not the final fully independent runtime package.

## 4. External phrasing

Safe phrasing:

1. StoryLock packages local authorization results into reusable skill interfaces
2. the migrated package now provides a runnable JS layer plus Rust/WASM build validation
3. the formal security boundary remains in StoryLock core semantics

Avoid saying:

1. the skill layer itself is the security boundary
2. the migrated package has already fully replaced the original runtime stack
3. the current Rust/WASM integration is fully complete
