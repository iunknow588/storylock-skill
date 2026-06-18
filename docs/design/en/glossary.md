# StoryLock Glossary

## Purpose

Unify core terminology across design documents to avoid repeatedly changing how the same concept is described.

| Term | Definition |
| --- | --- |
| Skill | A capability unit with explicit input/output contracts |
| Main Skill | A Skill directly corresponding to a user task |
| Internal Skill | A Skill existing as an underlying building block |
| Orchestration Example | An example object demonstrating how to chain multiple Skills together |
| Host | A runtime providing local sensitive execution boundaries |
| Gateway | A controlled interface for remote Agents to access local capabilities |
| Challenge | A question or challenge flow used for authorization verification |
| Session | A short-term access state established after passing a challenge |
| Secret Object | A secret object protected by StoryLock |
| Story Processing Skill | A local Skill responsible for editing, modifying, and improving story content |
| Local Access Authorization Skill | A local Skill responsible for object strength judgment, grid challenge verification, short-term authorization, and auditing |
| Remote Skill | A remote-facing capability for third-party requests, packaging, orchestration, and delegation |
| Proxy Signature | A third party requests a local completion of a signature, rather than holding the private key itself |
