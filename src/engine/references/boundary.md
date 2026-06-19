# StoryLock Skill Boundary

## Overview

This page defines the responsibility split for the StoryLock skill package.

## What This Skill Does

1. draft and refine story material
2. review question-set strength
3. package login-field authorization results
4. package signature authorization results
5. provide stable demo and invocation surfaces

## What This Skill Does Not Do

1. replace StoryLock core security semantics
2. store secrets as its own persistence layer
3. turn cloud storage or blockchain storage into the skill's security boundary
4. claim that Rust/WASM host replacement is complete

## Required Host Surface

Authorization-oriented capabilities still depend on the host:

1. `createChallenge(identityId, scope)`
2. `submitChallengeAnswers(identityId, challengeId, answers)`
3. `readSecretObject(identityId, sessionId, secretObjectId)`

## Agent Guidelines

1. Treat the skill package as an interface layer.
2. Keep storage security separate from StoryLock authorization.
3. Use this page whenever you need to explain scope or safety limits.
