# StoryLock Submission References

This directory contains competition-facing and review-facing reference material for the current StoryLock repo.

## Mainline Code Map

| Current source directory | Role |
| --- | --- |
| `skill/src/skills/local-story-processing/` | Layer 1 story processing and strength review |
| `skill/src/skills/local-story-access/` | Layer 2 local authorization and grid verification |
| `skill/src/skills/remote-gateway/` | Layer 3 remote request wrapping and redaction |
| `skill/src/engine/` | Compatibility and demo package |

## Recommended Reading Order

1. `01-参赛概览.md`
2. `02-技术映射说明.md`
3. `03-演示与调用说明.md`
4. `04-提交材料清单.md`
5. `05-Android宿主实现规范.md`
6. `06-评审讲解与演示说明.md`
7. `07-APK分发与安装说明.md`
8. `08-易安部署与域名说明.md`
9. `09-三层术语与PHAROS定位.md`
10. `10-Android真机闭环检查.md`
11. `11-Linux平台密钥存储与检查说明.md`

## Notes

1. Treat `storylock-skill-guide/` as compatibility reference material.
2. Prefer the simplified runtime directories over legacy `src/storylock-*` path names.
3. Use `node scripts/test/run-selftests.mjs` from `skill/` before demos or submissions.
4. Use `11-Linux平台密钥存储与检查说明.md` when you need the current Linux platform boundary and SecretStore status.
