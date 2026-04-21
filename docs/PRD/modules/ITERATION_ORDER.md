# PRD Modules Iteration Order

目标是先做基础能力和横向基础设施，再做上层集成与边缘功能。避免一上来就碰 UI、IDE、ACP 这类高耦合表层东西。

## Priority 0, Foundation

1. `global.md`
2. `config.md`
3. `env.md`
4. `id.md`
5. `util.md`
6. `storage.md`
7. `file.md`
8. `project.md`
9. `flag.md`
10. `format.md`

## Priority 1, Core Runtime

11. `session.md`
12. `agent.md`
13. `tool.md`
14. `provider.md`
15. `permission.md`
16. `shell.md`
17. `pty.md`
18. `patch.md`
19. `question.md`
20. `effect.md`

## Priority 2, Integration Layer

21. `plugin.md`
22. `skill.md`
23. `mcp.md`
24. `lsp.md`
25. `server.md`
26. `auth.md`
27. `account.md`
28. `sync.md`
29. `share.md`
30. `bus.md`
31. `control-plane.md`

## Priority 3, Product Surface

32. `cli.md`
33. `installation.md`
34. `git.md`
35. `npm.md`
36. `snapshot.md`
37. `worktree.md`
38. `ide.md`
39. `acp.md`
40. `v2.md`

## Execution Rule

- 严格按顺序，一个接一个迭代执行。
- 当前 iteration 未彻底完成前，不开启下一个 module iteration。
- 完成判断以对应 tasks json 中是否还有非 `done` 任务为准。
- 不再以 `verification-report.md` 是否存在作为完成条件。
